//! Fill absent bibliographic fields from the existing public catalog, never from manual tags.
use crate::{commands::LibraryState, models::Project};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::Manager;

const CATALOG_TTL_SECONDS: i64 = 300;
const MAX_SOURCE_BYTES: u64 = 16 * 1024 * 1024;
const SHEET_ID: &str = "1jxBbWB08myhD8YXePPifsWQG3JH2qZtBs9Y5yYcqE7g";

#[derive(Clone, Copy)]
struct CatalogSourceSpec {
    id: &'static str,
    name: &'static str,
    gid: &'static str,
}
const SOURCES: [CatalogSourceSpec; 4] = [
    CatalogSourceSpec {
        id: "beta",
        name: "Beta Index",
        gid: "843963812",
    },
    CatalogSourceSpec {
        id: "projects",
        name: "Project.json",
        gid: "208890884",
    },
    CatalogSourceSpec {
        id: "broken",
        name: "404 Broken Link",
        gid: "574767915",
    },
    CatalogSourceSpec {
        id: "discoveries",
        name: "Unreviewed discoveries",
        gid: "1285606730",
    },
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSourceSnapshot {
    pub id: String,
    pub name: String,
    pub rows: Vec<Vec<String>>,
    pub fetched_at: String,
    pub stale: bool,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSnapshot {
    pub sources: Vec<CatalogSourceSnapshot>,
    pub fetched_at: String,
    pub stale: bool,
    pub warnings: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct StoredSource {
    schema: u8,
    gid: String,
    source: CatalogSourceSnapshot,
}
#[derive(Default)]
struct SnapshotCache {
    root: Option<PathBuf>,
    sources: Vec<CatalogSourceSnapshot>,
    last_attempt: Option<chrono::DateTime<chrono::Utc>>,
    warnings: Vec<String>,
    completed_at: Option<Instant>,
}
static SNAPSHOTS: Mutex<Option<SnapshotCache>> = Mutex::new(None);

fn parse_rows(source: &str) -> Result<Vec<Vec<String>>, String> {
    let json = source
        .split_once("setResponse(")
        .and_then(|(_, tail)| tail.trim().strip_suffix(");"))
        .ok_or("Public catalog response has an unsupported format")?;
    let payload: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if payload["status"] != "ok" {
        return Err("Public catalog returned an error".into());
    }
    let cols = payload["table"]["cols"]
        .as_array()
        .ok_or("Catalog columns missing")?;
    let data = payload["table"]["rows"]
        .as_array()
        .ok_or("Catalog rows missing")?;
    if cols.is_empty() || cols.len() > 128 || data.len() > 25000 {
        return Err("Public catalog dimensions exceed supported limits".into());
    }
    let mut rows = vec![cols
        .iter()
        .map(|c| c["label"].as_str().unwrap_or("").trim().to_string())
        .collect()];
    for row in data {
        let cells = row["c"].as_array().ok_or("Invalid catalog cells")?;
        rows.push(
            (0..cols.len())
                .map(|i| {
                    let Some(cell) = cells.get(i) else {
                        return String::new();
                    };
                    if let Some(value) = cell["f"].as_str().or_else(|| cell["v"].as_str()) {
                        value.to_string()
                    } else if cell["v"].is_null() {
                        String::new()
                    } else {
                        cell["v"].to_string()
                    }
                })
                .collect(),
        );
    }
    if rows.len() < 2 {
        return Err("Public catalog contains no data rows".into());
    }
    Ok(rows)
}

fn validate_source(spec: CatalogSourceSpec, rows: &[Vec<String>]) -> Result<(), String> {
    let headers = rows.first().ok_or("Catalog columns missing")?;
    if rows.len() < 2 || rows.len() > 25001 || headers.is_empty() || headers.len() > 128 {
        return Err("Catalog has no usable rows or exceeds supported limits".into());
    }
    let has = |name: &str| headers.iter().any(|h| h.trim().eq_ignore_ascii_case(name));
    let expected = match spec.id {
        "beta" | "broken" => has("Title") && has("Interactive"),
        "projects" => has("Name") && has("Link"),
        // This tab deliberately has a free-form instruction header, not Beta Index columns.
        "discoveries" => headers.iter().any(|h| !h.trim().is_empty()),
        _ => false,
    };
    if !expected {
        return Err(format!("{} has unexpected columns", spec.name));
    }
    let mut bytes = 0u64;
    for row in rows {
        if row.len() > 128 {
            return Err("Catalog has too many cells".into());
        }
        for cell in row {
            if cell.len() > 128 * 1024 {
                return Err("Catalog cell exceeds supported limit".into());
            }
            bytes += cell.len() as u64;
            if bytes > MAX_SOURCE_BYTES {
                return Err("Catalog exceeds supported size limit".into());
            }
        }
    }
    if !rows
        .iter()
        .skip(1)
        .any(|r| r.iter().any(|c| !c.trim().is_empty()))
    {
        return Err("Catalog contains only empty data rows".into());
    }
    Ok(())
}

fn cache_root() -> Result<PathBuf, String> {
    if let Some(root) = std::env::var_os("CYOA_MANAGER_DATA_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
    {
        return Ok(root.join("save/catalog-cache"));
    }
    if let Some(root) = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
    {
        return Ok(root.join("cyoa-manager/catalog"));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
        .map(|home| home.join(".cache/cyoa-manager/catalog"))
        .ok_or_else(|| "Cannot locate the user's catalog cache directory".into())
}

fn read_source_cache(
    root: &Path,
    spec: CatalogSourceSpec,
) -> Result<Option<CatalogSourceSnapshot>, String> {
    let file = match std::fs::File::open(root.join(format!("{}.json", spec.id))) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("Catalog cache exceeds supported size limit".into());
    }
    let stored: StoredSource = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if stored.schema != 1 || stored.gid != spec.gid || stored.source.id != spec.id {
        return Err("Catalog cache belongs to a different source or format".into());
    }
    validate_source(spec, &stored.source.rows)?;
    chrono::DateTime::parse_from_rfc3339(&stored.source.fetched_at)
        .map_err(|_| "Catalog cache timestamp is invalid")?;
    let mut source = stored.source;
    source.name = spec.name.into();
    Ok(Some(source))
}

fn write_source_cache(
    root: &Path,
    spec: CatalogSourceSpec,
    source: &CatalogSourceSnapshot,
) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&StoredSource {
        schema: 1,
        gid: spec.gid.into(),
        source: source.clone(),
    })
    .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("Serialized catalog cache exceeds supported size limit".into());
    }
    let temporary = root.join(format!(".{}-{}.tmp", spec.id, uuid::Uuid::new_v4()));
    let result = (|| {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary).map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        std::fs::rename(&temporary, root.join(format!("{}.json", spec.id)))
            .map_err(|e| e.to_string())?;
        std::fs::File::open(root)
            .and_then(|file| file.sync_all())
            .map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

fn fresh(source: &CatalogSourceSnapshot, now: chrono::DateTime<chrono::Utc>) -> bool {
    chrono::DateTime::parse_from_rfc3339(&source.fetched_at)
        .map(|time| {
            let age = now.signed_duration_since(time);
            age.num_seconds() >= 0 && age.num_seconds() < CATALOG_TTL_SECONDS
        })
        .unwrap_or(false)
}

fn load_snapshot<F>(
    cache: &mut SnapshotCache,
    root: &Path,
    force: bool,
    now: chrono::DateTime<chrono::Utc>,
    mut fetch: F,
) -> CatalogSnapshot
where
    F: FnMut(&CatalogSourceSpec) -> Result<Vec<Vec<String>>, String>,
{
    let mut loaded_disk = false;
    if cache.root.as_deref() != Some(root) {
        *cache = SnapshotCache {
            root: Some(root.to_path_buf()),
            ..Default::default()
        };
        for spec in SOURCES {
            match read_source_cache(root, spec) {
                Ok(Some(source)) => {
                    cache.sources.push(source);
                    loaded_disk = true;
                }
                Ok(None) => {}
                Err(error) => cache
                    .warnings
                    .push(format!("{} cache ignored: {error}", spec.name)),
            }
        }
        // Offline startup is instant even when the saved snapshot is old. Refresh is explicit.
        if loaded_disk && !force {
            cache.last_attempt = Some(now);
        }
    }
    let recent_attempt = cache
        .last_attempt
        .map(|last| {
            let elapsed = now.signed_duration_since(last).num_seconds();
            elapsed >= 0 && elapsed < CATALOG_TTL_SECONDS
        })
        .unwrap_or(false);
    // Cached searches never wait on network; the UI refreshes stale data separately.
    if force || (cache.sources.is_empty() && !recent_attempt) {
        cache.warnings.clear();
        for spec in SOURCES {
            let fetched = fetch(&spec).and_then(|rows| {
                validate_source(spec, &rows)?;
                Ok(rows)
            });
            match fetched {
                Ok(rows) => {
                    let source = CatalogSourceSnapshot {
                        id: spec.id.into(),
                        name: spec.name.into(),
                        rows,
                        fetched_at: now.to_rfc3339(),
                        stale: false,
                        error: None,
                    };
                    if let Err(error) = write_source_cache(root, spec, &source) {
                        cache.warnings.push(format!(
                            "{} refreshed, but could not save offline cache: {error}",
                            spec.name
                        ));
                    }
                    if let Some(old) = cache.sources.iter_mut().find(|s| s.id == spec.id) {
                        *old = source;
                    } else {
                        cache.sources.push(source);
                    }
                }
                Err(error) => {
                    if let Some(old) = cache.sources.iter_mut().find(|s| s.id == spec.id) {
                        old.stale = true;
                        old.error = Some(error);
                    } else {
                        cache.sources.push(CatalogSourceSnapshot {
                            id: spec.id.into(),
                            name: spec.name.into(),
                            rows: vec![],
                            fetched_at: String::new(),
                            stale: true,
                            error: Some(error),
                        });
                    }
                }
            }
        }
        cache.last_attempt = Some(now);
    }
    let sources: Vec<_> = SOURCES
        .iter()
        .map(|spec| {
            let mut source = cache
                .sources
                .iter()
                .find(|s| s.id == spec.id)
                .cloned()
                .unwrap_or_else(|| CatalogSourceSnapshot {
                    id: spec.id.into(),
                    name: spec.name.into(),
                    rows: vec![],
                    fetched_at: String::new(),
                    stale: true,
                    error: Some("No saved snapshot. Refresh the catalog when online.".into()),
                });
            source.stale = source.error.is_some() || !fresh(&source, now);
            source
        })
        .collect();
    let mut warnings = cache.warnings.clone();
    for source in &sources {
        if let Some(error) = &source.error {
            warnings.push(format!("{}: {error}", source.name));
        }
    }
    let fetched_at = sources
        .iter()
        .filter(|s| !s.fetched_at.is_empty())
        .map(|s| s.fetched_at.as_str())
        .min()
        .unwrap_or("")
        .to_string();
    CatalogSnapshot {
        stale: sources.iter().any(|s| s.stale),
        sources,
        fetched_at,
        warnings,
    }
}

fn fetch_source(
    client: &reqwest::blocking::Client,
    spec: &CatalogSourceSpec,
) -> Result<Vec<Vec<String>>, String> {
    let url = format!(
        "https://docs.google.com/spreadsheets/d/{SHEET_ID}/gviz/tq?tqx=out:json&headers=1&gid={}",
        spec.gid
    );
    let response = client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| e.to_string())?;
    if response
        .content_length()
        .map(|size| size > MAX_SOURCE_BYTES)
        .unwrap_or(false)
    {
        return Err("Catalog response exceeds supported size limit".into());
    }
    let mut bytes = Vec::new();
    response
        .take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err("Catalog response exceeds supported size limit".into());
    }
    parse_rows(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)
}

/// Refreshes coalesce; cached reads can use atomically saved data without waiting.
#[tauri::command(async)]
pub fn get_catalog_snapshot(force: Option<bool>) -> Result<CatalogSnapshot, String> {
    snapshot_from(&SNAPSHOTS, &cache_root()?, force.unwrap_or(false))
}
fn snapshot_from(store: &Mutex<Option<SnapshotCache>>, root: &Path, force: bool) -> Result<CatalogSnapshot, String> {
    let started = Instant::now();
    let mut guard = match store.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::WouldBlock) => {
            if !force {
                let mut disk = SnapshotCache::default();
                let mut saved = load_snapshot(&mut disk, root, false, chrono::Utc::now(), |_| Err("Refresh in progress".into()));
                if saved.sources.iter().any(|source| !source.rows.is_empty()) {
                    saved.warnings.push("Refresh in progress; showing last saved catalog.".into());
                    return Ok(saved);
                }
            }
            store.lock().map_err(|e|e.to_string())?
        }
        Err(error) => return Err(error.to_string()),
    };
    let cache = guard.get_or_insert_with(SnapshotCache::default);
    // A waiting caller shares the refresh that finished after it started, including failures.
    let coalesced = cache
        .completed_at
        .map(|completed| completed >= started)
        .unwrap_or(false);
    let mut client: Option<Result<reqwest::blocking::Client, String>> = None;
    let mut fetched = false;
    let snapshot = load_snapshot(
        cache,
        &root,
        force && !coalesced,
        chrono::Utc::now(),
        |spec| {
            fetched = true;
            let client = client.get_or_insert_with(|| {
                reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(15))
                    .connect_timeout(Duration::from_secs(5))
                    .build()
                    .map_err(|error| error.to_string())
            });
            match client {
                Ok(client) => fetch_source(client, spec),
                Err(error) => Err(error.clone()),
            }
        },
    );
    if fetched {
        cache.completed_at = Some(Instant::now());
    }
    Ok(snapshot)
}

/// Backward-compatible Beta Index rows used by metadata enrichment.
#[tauri::command(async)]
pub fn get_catalog_rows() -> Result<Vec<Vec<String>>, String> {
    let snapshot = get_catalog_snapshot(None)?;
    let beta = snapshot
        .sources
        .into_iter()
        .find(|source| source.id == "beta")
        .ok_or("Beta Index snapshot missing")?;
    if beta.rows.is_empty() {
        return Err(beta
            .error
            .unwrap_or_else(|| "Beta Index is unavailable".into()));
    }
    Ok(beta.rows)
}

#[derive(Clone, Default, PartialEq)]
struct Credit {
    author: String,
    fandom: String,
    description: String,
}
fn source_key(value: &str) -> Option<String> {
    let mut url = tauri::Url::parse(value).ok()?;
    if !matches!(url.scheme(), "https" | "http") {
        return None;
    }
    let path = url.path().trim_end_matches('/');
    let path = path
        .strip_suffix("/project.json")
        .or_else(|| path.strip_suffix("/index.html"))
        .unwrap_or(path)
        .to_string();
    url.set_path(&format!("{path}/"));
    Some(url.into())
}
fn catalog(payload: &Value) -> BTreeMap<String, Option<Credit>> {
    let mut result = BTreeMap::new();
    let Some(rows) = payload["values"].as_array() else {
        return result;
    };
    let Some(headers) = rows.first().and_then(Value::as_array) else {
        return result;
    };
    let columns: BTreeMap<&str, usize> = headers
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.as_str().map(|s| (s.trim(), i)))
        .collect();
    for row in rows.iter().skip(1) {
        let cell = |key: &str| {
            columns
                .get(key)
                .and_then(|&i| row[i].as_str())
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let Some(key) = source_key(&cell("Interactive")) else {
            continue;
        };
        let credit = Credit {
            author: cell("Author"),
            fandom: cell("Universe"),
            description: cell("Description"),
        };
        if let Some(previous) = result.get(&key) {
            if previous != &Some(credit.clone()) {
                result.insert(key, None);
            }
        } else {
            result.insert(key, Some(credit));
        }
    }
    result
}
fn fill(project: &mut Project, credit: &Credit) -> bool {
    let mut changed = false;
    for (field, value) in [
        (&mut project.metadata.author, &credit.author),
        (&mut project.metadata.fandom, &credit.fandom),
        (&mut project.description, &credit.description),
    ] {
        if field.trim().is_empty() && !value.is_empty() {
            *field = value.clone();
            changed = true;
        }
    }
    changed
}
#[tauri::command]
pub async fn enrich_catalog_metadata(app: tauri::AppHandle) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<LibraryState>();
        let needed = state
            .lock()
            .map_err(|e| e.to_string())?
            .projects
            .iter()
            .any(|p| {
                p.source_url.is_some()
                    && (p.metadata.author.is_empty()
                        || p.metadata.fandom.is_empty()
                        || p.description.is_empty())
            });
        if !needed {
            return Ok(0);
        }
        // Optional network enrichment must not prevent local startup when the catalog is unavailable.
        let rows = match get_catalog_rows() {
            Ok(rows) => rows,
            Err(e) => {
                eprintln!("Catalog metadata unavailable: {e}");
                return Ok(0);
            }
        };
        let credits = catalog(&serde_json::json!({"values":rows}));
        let mut library = state.lock().map_err(|e| e.to_string())?;
        let mut changed = 0;
        for project in &mut library.projects {
            let key = project
                .project_json_url
                .as_deref()
                .or(project.source_url.as_deref())
                .and_then(source_key);
            let Some(Some(credit)) = key.and_then(|key| credits.get(&key)) else {
                continue;
            };
            if fill(project, credit) {
                crate::library::update_project(project)?;
                changed += 1;
            }
        }
        Ok(changed)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[cfg(test)]
include!("../../tests/rust/catalog_metadata.rs");
