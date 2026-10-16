//! Catalog content inspection and reviewed, identity-preserving updates.
//! Network responses and scripts are data only. Inspection never changes the library.
use crate::{commands::LibraryState, models::Project};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};

const MAX_PROJECT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_HTML_BYTES: u64 = 2 * 1024 * 1024;
const PREVIEW_TTL: Duration = Duration::from_secs(15 * 60);
const MAX_PREVIEWS: usize = 3;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadPlan {
    pub mode: String,
    pub source_url: String,
    pub reason: String,
    pub compatibility: Option<crate::compatibility::Report>,
    pub archive_url: Option<String>,
    pub project_url: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePreview {
    pub token: String,
    pub project_name: String,
    pub diff: crate::update_diff::UpdateDiff,
    pub source_url: String,
}
#[derive(Default, Deserialize)]
struct ThinProject {
    #[serde(default)]
    rows: Vec<ThinRow>,
}
#[derive(Default, Deserialize)]
struct ThinRow {
    #[serde(default, alias = "perks")]
    objects: Vec<ThinChoice>,
}
#[derive(Default, Deserialize)]
struct ThinChoice {
    #[serde(default, alias = "uid")]
    id: String,
}
fn choice_ids(path: &Path) -> Result<BTreeSet<String>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let project: ThinProject =
        serde_json::from_reader(std::io::BufReader::new(file)).map_err(|e| e.to_string())?;
    let ids = project
        .rows
        .into_iter()
        .flat_map(|r| r.objects)
        .filter_map(|c| if c.id.is_empty() { None } else { Some(c.id) })
        .collect::<BTreeSet<_>>();
    if ids.is_empty() {
        return Err("No identified ICC choices found; use the original website viewer.".into());
    }
    Ok(ids)
}
struct Staging {
    root: PathBuf,
}
impl Staging {
    fn new() -> Result<Self, String> {
        let root = std::env::temp_dir().join(format!(
            "cyoa-catalog-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&root).map_err(|e| e.to_string())?;
        Ok(Self { root })
    }
}
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
/// Remove only our UID's expired process directories; never follow symlinks.
#[cfg(target_os = "linux")]
fn cleanup_stale_in(root: &Path) {
    use std::os::unix::fs::MetadataExt;
    let Ok(owner) = fs::metadata("/proc/self") else {
        return;
    };
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(rest) = name.strip_prefix("cyoa-catalog-") else {
            continue;
        };
        let Some((pid, uuid)) = rest.split_once('-') else {
            continue;
        };
        let Ok(pid) = pid.parse::<u32>() else {
            continue;
        };
        if uuid::Uuid::parse_str(uuid).is_err()
            || pid == std::process::id()
            || Path::new(&format!("/proc/{pid}")).exists()
        {
            continue;
        }
        if let Ok(metadata) = fs::symlink_metadata(entry.path()) {
            if metadata.is_dir()
                && !metadata.file_type().is_symlink()
                && metadata.uid() == owner.uid()
            {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }
}
#[cfg(target_os = "linux")]
pub(crate) fn cleanup_stale() {
    cleanup_stale_in(&std::env::temp_dir());
}
fn http_url(raw: &str) -> Result<tauri::Url, String> {
    let url = tauri::Url::parse(raw.trim()).map_err(|e| e.to_string())?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Use a public HTTP or HTTPS URL without credentials.".into());
    }
    Ok(url)
}
fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(90))
        .connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| e.to_string())
}
fn copy_bounded(mut input: impl Read, mut output: impl Write, limit: u64) -> Result<u64, String> {
    let count = std::io::copy(&mut input.by_ref().take(limit + 1), &mut output)
        .map_err(|e| e.to_string())?;
    if count > limit {
        return Err(format!(
            "Response exceeds the {} MiB inspection limit",
            limit / 1024 / 1024
        ));
    }
    Ok(count)
}
fn fetch(
    client: &reqwest::blocking::Client,
    url: &tauri::Url,
    path: &Path,
    limit: u64,
) -> Result<tauri::Url, String> {
    let response = client
        .get(url.clone())
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?;
    let final_url = http_url(response.url().as_str())?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    // Landing HTML must not consume the JSON artwork budget. Direct JSON and
    // extensionless JSON MIME responses may use the explicit project bound.
    let is_json = final_url.path().to_ascii_lowercase().ends_with(".json")
        || content_type.contains("application/json")
        || content_type.contains("+json");
    let limit = if limit == MAX_PROJECT_BYTES && !is_json {
        MAX_HTML_BYTES
    } else {
        limit
    };
    if response.content_length().is_some_and(|n| n > limit) {
        return Err("Response exceeds the inspection size limit".into());
    }
    copy_bounded(
        response,
        fs::File::create(path).map_err(|e| e.to_string())?,
        limit,
    )?;
    Ok(final_url)
}
#[derive(Deserialize)]
struct SupportedShape {
    #[serde(rename = "pointTypes")]
    point_types: Option<Vec<serde::de::IgnoredAny>>,
    styling: Option<BTreeMap<String, serde::de::IgnoredAny>>,
    rows: Vec<SupportedRow>,
}
#[derive(Deserialize)]
struct SupportedRow {
    objects: Option<Vec<ThinChoice>>,
}
fn inspect_file(path: &Path) -> Result<crate::compatibility::Report, String> {
    choice_ids(path)?;
    let report = crate::compatibility::inspect(std::io::BufReader::new(
        fs::File::open(path).map_err(|e| e.to_string())?,
    ))?;
    if !report.needs_native_viewer() {
        let shape: SupportedShape = serde_json::from_reader(std::io::BufReader::new(
            fs::File::open(path).map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?;
        if shape.point_types.is_none()
            || shape.styling.is_none()
            || shape.rows.iter().any(|r| r.objects.is_none())
        {
            return Err(
                "Project format does not match bundled ICC2. Use its original website viewer."
                    .into(),
            );
        }
    }
    Ok(report)
}

struct Resolved {
    plan: DownloadPlan,
    staging: Staging,
    raw: PathBuf,
}
fn resolve(website: &str, archive: Option<String>) -> Result<Resolved, String> {
    let url = http_url(website)?;
    let archive = archive
        .filter(|s| !s.trim().is_empty())
        .map(|s| http_url(&s).map(|u| u.to_string()))
        .transpose()?;
    let staging = Staging::new()?;
    let raw = staging.root.join("input.json");
    let client = client()?;
    let direct = url.path().to_lowercase().ends_with(".json");
    let mut html = None;
    let mut source_ok = false;
    let mut last_error = String::new();
    // Inspect the selected link itself first; extensionless JSON is supported.
    if let Ok(final_url) = fetch(&client, &url, &raw, MAX_PROJECT_BYTES) {
        source_ok = true;
        if let Ok(report) = inspect_file(&raw) {
            let native = report.needs_native_viewer();
            return Ok(Resolved { plan: DownloadPlan { mode: if native { "website" } else { "project" }.into(), source_url: website.into(), reason: if native { "ICC Remix/scripts require the original website viewer; no script conversion was attempted." } else { "Project content contains identified ICC choices and no known native-only scripts." }.into(), compatibility: Some(report), archive_url: archive, project_url: Some(final_url.to_string()) }, staging, raw });
        }
        if fs::metadata(&raw)
            .map(|m| m.len() <= MAX_HTML_BYTES)
            .unwrap_or(false)
        {
            html = fs::read_to_string(&raw).ok().map(|text| (final_url, text));
        }
    }
    // A route can select a different CYOA inside one HTML application. Static
    // links/default filenames cannot prove correspondence to that route. The
    // exact selected URL was inspected above, so direct JSON remains allowed.
    let routed = |candidate: &tauri::Url| {
        candidate.query().is_some_and(|s| !s.is_empty())
            || candidate.fragment().is_some_and(|s| !s.is_empty())
    };
    if routed(&url)
        || html
            .as_ref()
            .is_some_and(|(final_url, _)| routed(final_url))
    {
        return Ok(Resolved {
            plan: DownloadPlan {
                mode: if source_ok && !direct { "website" } else { "online" }.into(),
                source_url: website.into(),
                reason: "This URL selects a query or fragment route. Its matching project cannot be proven from static HTML; use the original website viewer or play online. No default project was substituted.".into(),
                compatibility: None, archive_url: archive, project_url: None,
            }, staging, raw,
        });
    }
    if !direct {
        let base = html.as_ref().map(|(u, _)| u).unwrap_or(&url);
        let mut candidates = Vec::new();
        if let Some((_, text)) = &html {
            let links = regex::Regex::new(
                r#"["']([^"'<>\s]*project[^"'<>\s]*\.json(?:\?[^"'<>\s]*)?)["']"#,
            )
            .map_err(|e| e.to_string())?;
            for found in links.captures_iter(text).take(3) {
                if let Ok(candidate) = base.join(&found[1]) {
                    if matches!(candidate.scheme(), "http" | "https") {
                        candidates.push(candidate);
                    }
                }
            }
        }
        let mut folder = base.clone();
        let path = folder.path();
        if !path.ends_with('/') && !path.rsplit('/').next().unwrap_or_default().contains('.') {
            folder.set_path(&format!("{path}/"));
        }
        if let Ok(candidate) = folder.join("project.json") {
            candidates.push(candidate);
        }
        let mut seen = BTreeSet::new();
        for candidate in candidates
            .into_iter()
            .filter(|u| seen.insert(u.to_string()))
            .take(4)
        {
            match fetch(&client, &candidate, &raw, MAX_PROJECT_BYTES)
                .and_then(|u| inspect_file(&raw).map(|r| (u, r)))
            {
                Ok((project_url, report)) => {
                    let native = report.needs_native_viewer();
                    return Ok(Resolved { plan: DownloadPlan { mode: if native { "website" } else { "project" }.into(), source_url: website.into(), reason: if native { "Native Remix/script features detected. Save the original website or play online." } else { "Inspected actual project content; bundled ICC viewer is available." }.into(), compatibility: Some(report), archive_url: archive, project_url: Some(project_url.to_string()) }, staging, raw });
                }
                Err(error) => last_error = error,
            }
        }
    }
    Ok(Resolved { plan: DownloadPlan { mode: if source_ok && !direct { "website" } else { "online" }.into(), source_url: website.into(), reason: format!("No supported project could be verified without running website scripts. Save the original website if possible, or play online. {last_error}"), compatibility: None, archive_url: archive, project_url: None }, staging, raw })
}
#[tauri::command]
pub async fn inspect_catalog_download(
    website_url: String,
    zip_url: Option<String>,
) -> Result<DownloadPlan, String> {
    tauri::async_runtime::spawn_blocking(move || resolve(&website_url, zip_url).map(|r| r.plan))
        .await
        .map_err(|e| e.to_string())?
}

pub(crate) fn edition_digest(path: &Path) -> Result<String, String> {
    let mut hash = Sha256::new();
    hash.update(crate::builds::fingerprint(path)?.as_bytes());
    hash.update(serde_json::to_vec(&crate::assets::manifest(path)?).map_err(|e| e.to_string())?);
    Ok(format!("{:x}", hash.finalize()))
}
struct Pending {
    created: Instant,
    staging: Staging,
    path: PathBuf,
    previous: Project,
    before: String,
    candidate: String,
    source_url: String,
    project_url: String,
    diff: crate::update_diff::UpdateDiff,
}
fn pending() -> &'static Mutex<BTreeMap<String, Pending>> {
    static PENDING: OnceLock<Mutex<BTreeMap<String, Pending>>> = OnceLock::new();
    PENDING.get_or_init(|| {
        // One bounded cleanup worker, not one sleeping thread per preview.
        std::thread::spawn(|| loop {
            std::thread::sleep(Duration::from_secs(30));
            if let Ok(mut entries) = pending().lock() {
                expire(&mut entries);
            }
        });
        Mutex::new(BTreeMap::new())
    })
}
fn expire(entries: &mut BTreeMap<String, Pending>) {
    entries.retain(|_, p| p.created.elapsed() < PREVIEW_TTL);
}
fn ensure_same_source(previous: &Project, url: &str) -> Result<(), String> {
    let selected = http_url(url)?;
    // Deliberately retain query/fragment routes: sharing a hostname is not identity.
    if ![
        previous.source_url.as_deref(),
        previous.project_json_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter_map(|s| http_url(s).ok())
    .any(|u| u == selected)
    {
        return Err(
            "The selected link is not this card's saved source. Import it separately instead."
                .into(),
        );
    }
    crate::history::ensure_current_edition(previous)
}
fn ensure_related(before: &Path, after: &Path) -> Result<(), String> {
    let old = choice_ids(before)?;
    let new = choice_ids(after)?;
    let shared = old.intersection(&new).count();
    if shared == 0 || shared.saturating_mul(4) < old.len().min(new.len()) {
        return Err(
            "Too few choice IDs match the current edition. Import separately; no update was applied."
                .into(),
        );
    }
    Ok(())
}
fn prepare(resolved: &Resolved) -> Result<PathBuf, String> {
    if resolved.plan.mode != "project" {
        return Err(resolved.plan.reason.clone());
    }
    let base = http_url(
        resolved
            .plan
            .project_url
            .as_deref()
            .ok_or("Missing project URL")?,
    )?;
    crate::commands::prepare_catalog_candidate(
        &resolved.raw,
        &base,
        &resolved.staging.root.join("prepared"),
    )
}
#[tauri::command]
pub async fn preview_catalog_update(
    app: tauri::AppHandle,
    project_id: String,
    website_url: String,
) -> Result<UpdatePreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<LibraryState>();
        let previous = state
            .lock()
            .map_err(|e| e.to_string())?
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .cloned()
            .ok_or("Project not found")?;
        ensure_same_source(&previous, &website_url)?;
        let before = edition_digest(Path::new(&previous.file_path))?;
        let resolved = resolve(&website_url, None)?;
        let path = prepare(&resolved)?;
        ensure_related(Path::new(&previous.file_path), &path)?;
        let diff = crate::update_diff::compare_files(Path::new(&previous.file_path), &path)?;
        if before != edition_digest(Path::new(&previous.file_path))? {
            return Err("Current edition changed during preview; try again.".into());
        }
        let candidate = edition_digest(&path)?;
        let token = uuid::Uuid::new_v4().to_string();
        let result = UpdatePreview {
            token: token.clone(),
            project_name: previous.metadata.title.clone(),
            diff: diff.clone(),
            source_url: website_url.clone(),
        };
        let mut entries = pending().lock().map_err(|e| e.to_string())?;
        expire(&mut entries);
        if entries.len() >= MAX_PREVIEWS {
            return Err("Close another update preview before starting a new one.".into());
        }
        entries.insert(
            token,
            Pending {
                created: Instant::now(),
                staging: resolved.staging,
                path,
                previous,
                before,
                candidate,
                source_url: website_url,
                project_url: resolved.plan.project_url.ok_or("Missing project URL")?,
                diff,
            },
        );
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn discard_catalog_update(token: String) -> Result<(), String> {
    let mut entries = pending().lock().map_err(|e| e.to_string())?;
    expire(&mut entries);
    entries.remove(&token);
    Ok(())
}
fn publish_candidate(staging: &Staging, path: &Path) -> Result<(PathBuf, PathBuf), String> {
    let prepared = staging.root.join("prepared");
    let relative = path
        .strip_prefix(&prepared)
        .map_err(|_| "Candidate outside staging")?;
    let target = crate::library::cyoas_dir().join(format!("catalog-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(crate::library::cyoas_dir()).map_err(|e| e.to_string())?;
    // Temp and user data can be on different filesystems. Copy and hash-check before committing.
    fs::create_dir(&target).map_err(|e| e.to_string())?;
    let result = (|| {
        for entry in walkdir::WalkDir::new(&prepared).follow_links(false) {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().is_symlink() {
                return Err("Candidate contains a symbolic link".into());
            }
            let dest = target.join(
                entry
                    .path()
                    .strip_prefix(&prepared)
                    .map_err(|e| e.to_string())?,
            );
            if entry.file_type().is_dir() {
                fs::create_dir_all(dest).map_err(|e| e.to_string())?;
            } else if entry.file_type().is_file() {
                fs::copy(entry.path(), dest).map_err(|e| e.to_string())?;
            } else {
                return Err("Unsupported candidate file type".into());
            }
        }
        let copied = target.join(relative);
        if edition_digest(&copied)? != edition_digest(path)? {
            return Err("Candidate copy verification failed".into());
        }
        Ok((target.clone(), copied))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&target);
    }
    result
}
fn validate_preview(pending: &Pending, current: &Project) -> Result<(), String> {
    ensure_same_source(&pending.previous, &pending.source_url)?;
    if serde_json::to_vec(current).map_err(|e| e.to_string())?
        != serde_json::to_vec(&pending.previous).map_err(|e| e.to_string())?
        || edition_digest(Path::new(&pending.previous.file_path))? != pending.before
        || edition_digest(&pending.path)? != pending.candidate
    {
        return Err("An edition or card changed after preview. Inspect again.".into());
    }
    Ok(())
}
#[tauri::command]
pub async fn apply_catalog_update(app: tauri::AppHandle, token: String) -> Result<Project, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let pending = {
            let mut entries = pending().lock().map_err(|e| e.to_string())?;
            expire(&mut entries);
            entries
                .remove(&token)
                .ok_or("Update preview expired; inspect again.")?
        };
        let state = app.state::<LibraryState>();
        {
            let library = state.lock().map_err(|e| e.to_string())?;
            let current = library
                .projects
                .iter()
                .find(|p| p.id == pending.previous.id)
                .ok_or("Project no longer exists")?;
            validate_preview(&pending, current)?;
        }
        let (target, copied) = publish_candidate(&pending.staging, &pending.path)?;
        let result = (|| {
            let archive =
                crate::history::snapshot(&pending.previous, "Before reviewed catalog update")?;
            crate::history::star_version(pending.previous.id.clone(), archive.id.clone(), true)?;
            crate::history::pin_sessions(&app, &pending.previous, &archive)?;
            crate::commands::replace_project_checked(
                &pending.previous.id,
                copied.to_string_lossy().into_owned(),
                &state,
                Some(pending.source_url.clone()),
                Some(pending.project_url.clone()),
                &pending.previous.name,
                Some((&pending.previous, &pending.before)),
            )
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(target);
        } else if let Err(error) = crate::history::remove_retired_files(&pending.previous, &state) {
            eprintln!("Reviewed update committed; old-file cleanup deferred: {error}");
        }
        if let Ok(project) = &result {
            let _ = app.emit(
                "project-update-result",
                serde_json::json!({"projectName":project.name,"diff":pending.diff}),
            );
        }
        result
    })
    .await
    .map_err(|e| e.to_string())?
}

fn extract_recovery(path: &Path, root: &Path) -> Result<PathBuf, String> {
    let mut archive = zip::ZipArchive::new(fs::File::open(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if archive.len() > 10_000 {
        return Err("Archive has too many entries".into());
    }
    let mut total = 0u64;
    let mut projects = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let relative = entry
            .enclosed_name()
            .ok_or("Archive path escapes its folder")?
            .to_path_buf();
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err("Archive symbolic links are not supported".into());
        }
        let out = root.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        total = total
            .checked_add(entry.size())
            .ok_or("Archive size overflow")?;
        if total > MAX_PROJECT_BYTES {
            return Err("Expanded archive exceeds 512 MiB".into());
        }
        fs::create_dir_all(out.parent().ok_or("Missing archive folder")?)
            .map_err(|e| e.to_string())?;
        let size = entry.size();
        copy_bounded(
            &mut entry,
            fs::File::create(&out).map_err(|e| e.to_string())?,
            size,
        )?;
        if out.extension().is_some_and(|s| s == "json") && inspect_file(&out).is_ok() {
            projects.push(out);
        }
    }
    if projects.len() != 1 {
        return Err(
            "Archive must contain exactly one identified project. Review it manually instead."
                .into(),
        );
    }
    let project = projects.pop().unwrap();
    if inspect_file(&project)?.needs_native_viewer() {
        return Err(
            "Archive uses native Remix/scripts. Use its original website viewer instead.".into(),
        );
    }
    Ok(project)
}
#[tauri::command]
pub async fn import_catalog_download(
    app: tauri::AppHandle,
    website_url: String,
    project_name: String,
    max_size_mb: u64,
    archive_url: Option<String>,
    allow_archive: Option<bool>,
) -> Result<Project, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let recovery = allow_archive.unwrap_or(false);
        let source = if recovery && website_url.trim().is_empty() { archive_url.as_deref().ok_or("Recovery needs an archive URL")? } else { &website_url };
        let website = http_url(source)?;
        let (staging, path, project_url) = if recovery {
            let archive_url = archive_url.clone().ok_or("Explicit recovery needs an archive URL")?;
            let url = http_url(&archive_url)?;
            let staging = Staging::new()?;
            let raw = staging.root.join("archive.zip");
            fetch(&client()?, &url, &raw, 64 * 1024 * 1024)?;
            let prepared = staging.root.join("prepared");
            let path = extract_recovery(&raw, &prepared)?;
            (staging, path, None)
        } else {
            let resolved = resolve(&website_url, None)?;
            let path = prepare(&resolved)?;
            (resolved.staging, path, resolved.plan.project_url)
        };
        let total = walkdir::WalkDir::new(staging.root.join("prepared")).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file()).try_fold(0u64, |total, entry| entry.metadata().map(|m| total.saturating_add(m.len())).map_err(|e| e.to_string()))?;
        if max_size_mb > 0 && total > max_size_mb.saturating_mul(1024 * 1024) { return Err("Prepared download exceeds your size limit. Change the limit before importing.".into()); }
        let (target, copied) = publish_candidate(&staging, &path)?;
        let state = app.state::<LibraryState>();
        let result = crate::commands::add_catalog_project_configured(copied.to_string_lossy().into_owned(), &state, Some(website.to_string()), project_url, |project| {
            if !project_name.trim().is_empty() {
                project.name = if recovery { format!("{} — archive recovery", project_name.trim()) } else { project_name.trim().to_string() };
                project.metadata.title = project.name.clone();
            }
            if recovery {
                project.metadata.restored_from_archive = true;
                project.tags.push("Archive recovery — age unverified".into());
                project.description = format!("Recovered from {}. Edition age is unverified; this is a separate library card.", archive_url.unwrap_or_default());
            }
        });
        if result.is_err() { let _ = fs::remove_dir_all(target); }
        result
    }).await.map_err(|e| e.to_string())?
}
#[cfg(test)]
include!("../../tests/rust/catalog_download.rs");
