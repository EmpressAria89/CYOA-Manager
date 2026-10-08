use std::{fs, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use uuid::Uuid;
use crate::{commands::LibraryState, library, models::Project};

#[derive(Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    pub created_at: String,
    pub reason: String,
    pub project: Project,
    #[serde(default)] pub starred: bool,
    #[serde(default)] pub source_project_id: Option<String>,
}

fn key(value: &str) -> Result<&str, String> {
    Uuid::parse_str(value).map_err(|_| "Invalid history ID".to_string())?;
    Ok(value)
}
fn history_dir(project_id: &str) -> Result<PathBuf, String> {
    Ok(library::data_root_dir().join("save/versions").join(key(project_id)?))
}

// Copy only regular files; refuse symlinks rather than silently lose assets or copy outside the project.
fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    for entry in walkdir::WalkDir::new(source).follow_links(false) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_symlink() { return Err("Archive refused a symbolic link; original files are unchanged".into()); }
        let relative = entry.path().strip_prefix(source).map_err(|e| e.to_string())?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(target).map_err(|e| e.to_string())?;
        } else if entry.file_type().is_file() {
            fs::copy(entry.path(), target).map_err(|e| e.to_string())?;
        } else { return Err("Archive contains an unsupported file type".into()); }
    }
    Ok(())
}
fn copy_project(project: &Project, destination: &Path) -> Result<PathBuf, String> {
    let path = Path::new(&project.file_path);
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    let filename = path.file_name().ok_or("Project has no filename")?;
    if filename == "project.json" {
        copy_tree(path.parent().ok_or("Project has no folder")?, destination)?;
    } else {
        if fs::symlink_metadata(path).map_err(|e| e.to_string())?.file_type().is_symlink() {
            return Err("Archive refused a symbolic link".into());
        }
        fs::copy(path, destination.join(filename)).map_err(|e| e.to_string())?;
        crate::assets::copy(path,destination)?;
    }
    Ok(destination.join(filename))
}

pub fn snapshot(project: &Project, reason: &str) -> Result<Version, String> {
    snapshot_at(&history_dir(&project.id)?, project, reason)
}
fn snapshot_at(root: &Path, project: &Project, reason: &str) -> Result<Version, String> {
    let id = Uuid::new_v4().to_string();
    let temporary = root.join(format!(".pending-{id}"));
    let completed = root.join(&id);
    let result = (|| {
        let copied = copy_project(project, &temporary.join("files"))?;
        crate::archive_storage::pack(&temporary,true)?;
        let mut archived = project.clone();
        archived.file_path = completed.join("files").join(copied.file_name().ok_or("Missing filename")?).to_string_lossy().into_owned();
        let version = Version { id, created_at: chrono::Utc::now().to_rfc3339(), reason: reason.into(), project: archived, starred: false, source_project_id: None };
        fs::write(temporary.join("version.json"), serde_json::to_vec_pretty(&version).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        fs::rename(&temporary, &completed).map_err(|e| e.to_string())?;
        Ok(version)
    })();
    if result.is_err() { let _ = fs::remove_dir_all(&temporary); }
    result
}
fn find_project(id: &str, state: &State<LibraryState>) -> Result<Project, String> {
    state.lock().map_err(|e| e.to_string())?.projects.iter().find(|p| p.id == id).cloned().ok_or_else(|| "Project not found".into())
}
pub fn pin_sessions(app: &tauri::AppHandle, source: &Project, version: &Version) -> Result<(), String> {
    let state=app.state::<crate::models::SessionStore>();
    let mut sessions=state.lock().map_err(|e|e.to_string())?;
    if sessions.values().any(|s|s.project_id==source.id && s.file_path==source.file_path){crate::archive_storage::with_files(Path::new(&version.project.file_path),||Ok(()))?;}
    for session in sessions.values_mut().filter(|s|s.project_id==source.id && s.file_path==source.file_path) {
        session.file_path=version.project.file_path.clone();session.project_id=version.project.id.clone();
    }
    Ok(())
}
pub fn apply_retention(app: &tauri::AppHandle, id: &str) -> Result<usize, String> {
    let state=app.state::<crate::models::SessionStore>();
    let protected=state.lock().map_err(|e|e.to_string())?.values().map(|s|s.file_path.clone()).collect::<Vec<_>>();
    prune(id,crate::preferences::load().archive_limit,&protected)
}
pub fn remove_retired_files(source: &Project, state: &State<LibraryState>) -> Result<(), String> {
    let path=Path::new(&source.file_path);
    let managed=path.canonicalize().ok().zip(library::cyoas_dir().canonicalize().ok()).is_some_and(|(p,r)|p.starts_with(r));
    if !managed || !path.exists(){return Ok(());}
    let projects=state.lock().map_err(|e|e.to_string())?.projects.clone();
    if projects.iter().any(|p|p.file_path==source.file_path){return Ok(());}
    let restored_folder=path.parent().and_then(|p|p.file_name()).and_then(|n|n.to_str()).and_then(|n|n.strip_prefix("restored-")).is_some_and(|id|Uuid::parse_str(id).is_ok());
    if path.file_name().is_some_and(|n|n=="project.json") || restored_folder {
        let folder=path.parent().ok_or("Project directory missing")?;
        if folder!=library::cyoas_dir() && !projects.iter().any(|p|Path::new(&p.file_path).starts_with(folder)){
            return fs::remove_dir_all(folder).map_err(|e|e.to_string());
        }
    }
    fs::remove_file(path).map_err(|e|e.to_string())
}
fn remove_archived_source(source: &Project, state: &State<LibraryState>) -> Result<(), String> {
    // Archive copy was verified before this is called. Remove card first, then only manager-owned files.
    library::delete_project(&source.id)?;
    let mut lib = state.lock().map_err(|e| e.to_string())?;
    lib.projects.retain(|p| p.id != source.id);
    drop(lib);
    remove_retired_files(source,state)?;
    if let Err(e) = crate::perk_index::remove_project_from_index_if_present(&source.id) { eprintln!("Index cleanup: {e}"); }
    Ok(())
}
#[tauri::command]
pub async fn archive_version(app: tauri::AppHandle, id: String) -> Result<Version, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<LibraryState>();
        let project = find_project(&id, &state)?;
        let version = snapshot(&project, "Archived from library")?;
        pin_sessions(&app,&project,&version)?;
        remove_archived_source(&project, &state)?;
        apply_retention(&app,&project.id)?;
        Ok(version)
    }).await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn archive_existing_copy(app: tauri::AppHandle, id: String, source_id: String) -> Result<Version, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<LibraryState>();
        let target = find_project(&id, &state)?;
        let source = find_project(&source_id, &state)?;
        if source.id == target.id { return Err("Choose another library copy".into()); }
        let reason = format!("Imported library copy: {}", source.name);
        let mut archived_source = source.clone(); archived_source.id = target.id.clone();
        let mut version = snapshot(&archived_source, &reason)?;
        version.source_project_id = Some(source.id.clone());
        save_version(&version)?;
        crate::builds::copy_associated_builds(&source.id, &target.id, &target.name)?;
        pin_sessions(&app,&source,&version)?;
        remove_archived_source(&source, &state)?;
        apply_retention(&app,&target.id)?;
        Ok(version)
    }).await.map_err(|e| e.to_string())?
}
fn save_version(version: &Version) -> Result<(), String> {
    let file = history_dir(&version.project.id)?.join(key(&version.id)?).join("version.json");
    let temporary = file.with_extension("pending");
    fs::write(&temporary, serde_json::to_vec_pretty(version).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(temporary, file).map_err(|e| e.to_string())
}
pub fn versions_for(id: &str) -> Result<Vec<Version>, String> {
    let root = history_dir(id)?;
    let mut versions = Vec::new();
    if !root.exists() { return Ok(versions); }
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if Uuid::parse_str(&entry.file_name().to_string_lossy()).is_err() { continue; }
        versions.push(serde_json::from_slice::<Version>(&fs::read(entry.path().join("version.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?);
    }
    versions.sort_by(|a,b| b.created_at.cmp(&a.created_at));
    Ok(versions)
}
pub fn all_versions() -> Result<Vec<Version>, String> {
    let root = library::data_root_dir().join("save/versions");
    let mut result = Vec::new();
    if root.exists() {
        for e in fs::read_dir(root).map_err(|e|e.to_string())? {
            let e=e.map_err(|e|e.to_string())?;
            let id=e.file_name().to_string_lossy().into_owned();
            if Uuid::parse_str(&id).is_ok() { result.extend(versions_for(&id)?); }
        }
    }
    result.sort_by(|a,b|b.created_at.cmp(&a.created_at)); Ok(result)
}
#[tauri::command]
pub async fn list_archives() -> Result<Vec<Version>, String> {
    tauri::async_runtime::spawn_blocking(all_versions).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub fn star_version(id: String, version_id: String, starred: bool) -> Result<(), String> {
    let _edit = ARCHIVE_EDITS.lock().map_err(|e|e.to_string())?;
    let mut version = versions_for(&id)?.into_iter().find(|v|v.id==version_id).ok_or("Version not found")?;
    version.starred=starred; save_version(&version)
}
pub fn prune(id: &str, keep: usize, protected: &[String]) -> Result<usize, String> {
    let versions=versions_for(id)?;
    prune_list(&history_dir(id)?,versions,keep,protected)
}
fn prune_list(root:&Path,versions:Vec<Version>,keep:usize,protected:&[String])->Result<usize,String>{
    let mut count=0; let mut removed=0;
    for version in versions {
        if version.starred || protected.iter().any(|path|path==&version.project.file_path) { continue; }
        count += 1;
        if count > keep.clamp(3,5) {
            // Currently open archive copies are protected by active readers' independent paths in the viewer.
            fs::remove_dir_all(root.join(key(&version.id)?)).map_err(|e|e.to_string())?;
            removed += 1;
        }
    }
    Ok(removed)
}
#[tauri::command]
pub async fn reconcile_imported_archives(app: tauri::AppHandle) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state=app.state::<LibraryState>(); let mut removed=0;
        for version in all_versions()? {
            if !version.reason.starts_with("Imported library copy:") { continue; }
            let candidates=state.lock().map_err(|e|e.to_string())?.projects.clone();
            for source in candidates.iter().filter(|p|p.id!=version.project.id && !p.metadata.restored_from_archive && p.name==version.project.name) {
                let matches=crate::archive_storage::with_files(Path::new(&version.project.file_path),||Ok(crate::builds::fingerprint(Path::new(&source.file_path))?==crate::builds::fingerprint(Path::new(&version.project.file_path))? && crate::assets::manifest(Path::new(&source.file_path))?==crate::assets::manifest(Path::new(&version.project.file_path))?))?;
                if !matches{continue;}
                crate::builds::copy_associated_builds(&source.id,&version.project.id,&version.project.name)?;
                pin_sessions(&app,source,&version)?;
                remove_archived_source(source,&state)?; removed+=1;
            }
        }
        Ok(removed)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn list_versions(id: String) -> Result<Vec<Version>, String> {
    let root = history_dir(&id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut result = Vec::new();
        if !root.exists() { return Ok(result); }
        for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if Uuid::parse_str(&entry.file_name().to_string_lossy()).is_err() { continue; }
            let version: Version = serde_json::from_slice(&fs::read(entry.path().join("version.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            result.push(version);
        }
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }).await.map_err(|e| e.to_string())?
}
pub fn ensure_current_edition(project: &Project) -> Result<(), String> {
    if project.metadata.restored_from_archive { return Err("This is a restored archive edition. Re-download the current main card instead.".into()); }
    Ok(())
}
// Restoration creates an independent card; archives and active editions are never replaced.
fn restored_copy(version: &Version, directory: &Path) -> Result<Project, String> {
    let path = crate::archive_storage::with_files(Path::new(&version.project.file_path), || copy_project(&version.project, directory))?;
    let mut project = version.project.clone();
    project.id = Uuid::new_v4().to_string();
    project.file_path = path.to_string_lossy().into_owned();
    project.file_missing = false;
    project.date_added = chrono::Utc::now().to_rfc3339();
    project.build_count = 0;
    project.metadata.restored_from_archive = true;
    Ok(project)
}
#[tauri::command]
pub async fn restore_version(app: tauri::AppHandle, id: String, version_id: String) -> Result<Project, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _edit = ARCHIVE_EDITS.lock().map_err(|e| e.to_string())?;
        let state = app.state::<LibraryState>();
        let root = history_dir(&id)?.join(key(&version_id)?);
        let version = read_version(&root)?;
        if version.project.id != id { return Err("Version belongs to another CYOA".into()); }
        let fingerprint = crate::archive_storage::with_files(Path::new(&version.project.file_path), || crate::builds::fingerprint(Path::new(&version.project.file_path)))?;
        // Retrying the same restore returns its existing, unchanged library copy.
        if let Ok(bytes) = fs::read(root.join("restored-library.json")) {
            let previous: String = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            let existing = state.lock().map_err(|e| e.to_string())?.projects.iter().find(|p| p.id == previous).cloned();
            if let Some(existing) = existing {
                if existing.name == version.project.name && existing.metadata.title == version.project.metadata.title && crate::builds::fingerprint(Path::new(&existing.file_path)).ok().as_deref() == Some(fingerprint.as_str()) {
                    let matches = crate::archive_storage::with_files(Path::new(&version.project.file_path), || Ok(crate::assets::manifest(Path::new(&existing.file_path))? == crate::assets::manifest(Path::new(&version.project.file_path))?))?;
                    if matches { return Ok(existing); }
                }
            }
        }
        let directory = library::cyoas_dir().join(format!("restored-{}", Uuid::new_v4()));
        let result = (|| {
            let mut restored = restored_copy(&version, &directory)?;
            crate::builds::copy_edition_builds(&id, &restored.id, &restored.name, &fingerprint)?;
            restored.build_count = crate::builds::count(&restored.id);
            let mut lib = state.lock().map_err(|e| e.to_string())?;
            library::insert_project(&restored)?;
            lib.projects.push(restored.clone());
            drop(lib);
            // A failed receipt write must not report failure after the card was added.
            if let Err(e) = write_json(&root.join("restored-library.json"), &restored.id) { eprintln!("Restore receipt: {e}"); }
            if let Err(e) = crate::perk_index::sync_index_for_project_if_present(&restored) { eprintln!("Perk index refresh failed: {e}"); }
            Ok(restored)
        })();
        if result.is_err() { let _ = fs::remove_dir_all(directory); }
        result
    }).await.map_err(|e| e.to_string())?
}

static ARCHIVE_EDITS: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn read_version(root: &Path) -> Result<Version, String> {
    serde_json::from_slice(&fs::read(root.join("version.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())
}
fn write_json(file: &Path, value: &impl Serialize) -> Result<(), String> {
    fs::create_dir_all(file.parent().ok_or("Missing metadata directory")?).map_err(|e|e.to_string())?;
    let temporary = file.with_extension(format!("{}.pending", Uuid::new_v4()));
    fs::write(&temporary, serde_json::to_vec_pretty(value).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    fs::rename(temporary, file).map_err(|e|e.to_string())
}
fn archive_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 200 || name.chars().any(char::is_control) { return Err("Enter a name of 1–200 characters".into()); }
    Ok(name.into())
}
#[derive(Serialize, Deserialize)]
struct GroupLabel { name: String }
#[tauri::command]
pub async fn list_archive_groups() -> Result<Vec<Project>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut groups = std::collections::BTreeMap::new();
        for version in all_versions()? {
            let mut project = version.project;
            if groups.contains_key(&project.id) { continue; }
            let label = history_dir(&project.id)?.join("group.json");
            if label.exists() {
                let label: GroupLabel = serde_json::from_slice(&fs::read(label).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                project.name = label.name.clone(); project.metadata.title = label.name;
            }
            groups.insert(project.id.clone(), project);
        }
        Ok(groups.into_values().collect())
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub fn rename_archive_group(id: String, name: String) -> Result<(), String> {
    let _edit = ARCHIVE_EDITS.lock().map_err(|e|e.to_string())?;
    write_json(&history_dir(&id)?.join("group.json"), &GroupLabel { name: archive_name(&name)? })
}
#[tauri::command]
pub fn rename_archive_version(id: String, version_id: String, name: String) -> Result<(), String> {
    let _edit = ARCHIVE_EDITS.lock().map_err(|e|e.to_string())?;
    let mut version = read_version(&history_dir(&id)?.join(key(&version_id)?))?;
    if version.project.id != id { return Err("Version belongs to another CYOA".into()); }
    let name = archive_name(&name)?;
    version.project.name = name.clone(); version.project.metadata.title = name;
    save_version(&version)
}
// Copy/verify/publish before retiring the old location. Interrupted transfers keep a recoverable copy.
fn relocate_version_at(version: &Version, source: &Path, destination: &Path, target_id: &str) -> Result<Version, String> {
    let mut moved = version.clone();
    moved.project.id = target_id.into();
    moved.project.file_path = destination.join("files").join(Path::new(&version.project.file_path).file_name().ok_or("Missing project filename")?).to_string_lossy().into_owned();
    let temporary = destination.with_file_name(format!(".pending-move-{}", version.id));
    if destination.exists() {
        let from: String = serde_json::from_slice(&fs::read(destination.join("transfer-from.json")).map_err(|_| "Destination already contains this edition")?).map_err(|e|e.to_string())?;
        if from != version.project.id || crate::builds::fingerprint(&source.join("files.zip"))? != crate::builds::fingerprint(&destination.join("files.zip"))? { return Err("Destination contains a conflicting edition".into()); }
        moved = read_version(destination)?;
    } else {
        if temporary.exists() { return Err("An interrupted transfer needs review; both original and pending files were retained".into()); }
        let result = (|| {
            copy_tree(source, &temporary)?;
            crate::archive_storage::pack(&temporary, true)?;
            write_json(&temporary.join("version.json"), &moved)?;
            write_json(&temporary.join("transfer-from.json"), &version.project.id)?;
            fs::rename(&temporary, destination).map_err(|e|e.to_string())
        })();
        if result.is_err() { let _ = fs::remove_dir_all(temporary); }
        result?;
    }
    let retired = source.with_file_name(format!(".moved-{}", version.id));
    fs::rename(source, &retired).map_err(|e|e.to_string())?;
    if let Err(e) = fs::remove_dir_all(retired) { eprintln!("Retired transfer cache retained: {e}"); }
    Ok(moved)
}
#[tauri::command]
pub async fn move_archive_version(app: tauri::AppHandle, id: String, version_id: String, target_id: Option<String>, new_group_name: Option<String>) -> Result<Version, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _edit = ARCHIVE_EDITS.lock().map_err(|e|e.to_string())?;
        let target = match target_id { Some(target) => { key(&target)?; target }, None => Uuid::new_v4().to_string() };
        if target == id { return Err("Choose another archive group".into()); }
        let source = history_dir(&id)?.join(key(&version_id)?);
        let destination = history_dir(&target)?.join(&version_id);
        if !source.exists() && destination.exists() {
            let from: String = serde_json::from_slice(&fs::read(destination.join("transfer-from.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            if from == id { return read_version(&destination); }
        }
        let version = read_version(&source)?;
        if version.project.id != id { return Err("Version belongs to another CYOA".into()); }
        let state = app.state::<crate::models::SessionStore>();
        let sessions = state.lock().map_err(|e|e.to_string())?;
        if sessions.values().any(|s|s.file_path==version.project.file_path) { return Err("Close this archived edition's reader before moving it".into()); }
        let fingerprint = crate::archive_storage::with_files(Path::new(&version.project.file_path), || crate::builds::fingerprint(Path::new(&version.project.file_path)))?;
        crate::builds::copy_edition_builds(&id, &target, &version.project.name, &fingerprint)?;
        if let Some(name) = new_group_name { write_json(&history_dir(&target)?.join("group.json"), &GroupLabel { name: archive_name(&name)? })?; }
        let moved = relocate_version_at(&version, &source, &destination, &target)?;
        drop(sessions);
        // Manual organization must never trigger retention deletion in the destination.
        Ok(moved)
    }).await.map_err(|e|e.to_string())?
}

pub fn release_archive_cache(app:&tauri::AppHandle)->Result<(),String>{
 let state=app.state::<crate::models::SessionStore>();let sessions=state.lock().map_err(|e|e.to_string())?;
 for version in all_versions()? {let file=Path::new(&version.project.file_path);let root=file.parent().and_then(Path::parent).ok_or("Archive folder missing")?;
  if root.join("files.zip").exists() && root.join("files").exists() && !sessions.values().any(|s|s.file_path==version.project.file_path){crate::archive_storage::pack(root,true)?;}
 }Ok(())
}
#[tauri::command]
pub async fn optimize_storage(app:tauri::AppHandle)->Result<serde_json::Value,String>{
 tauri::async_runtime::spawn_blocking(move||{
  let changed=crate::commands::compress_library_cover_images(app.state::<LibraryState>())?;
  let state=app.state::<crate::models::SessionStore>();let sessions=state.lock().map_err(|e|e.to_string())?;let mut compressed=0;
  for version in all_versions()?{let file=Path::new(&version.project.file_path);let root=file.parent().and_then(Path::parent).ok_or("Archive folder missing")?;
   let idle=!sessions.values().any(|s|s.file_path==version.project.file_path);
   if crate::archive_storage::pack(root,idle)?{compressed+=1;}
  }
  Ok(serde_json::json!({"thumbnails":changed,"archives":compressed}))
 }).await.map_err(|e|e.to_string())?
}

#[cfg(test)]
include!("../../tests/rust/history.rs");
