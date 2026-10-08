use std::{fs, path::Path};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::{library, models::ViewerSession};

#[derive(Serialize, Deserialize)]
pub struct Build {
    pub id: String,
    pub project_id: String,
    pub project_name: String,
    pub fingerprint: String,
    pub viewer_id: String,
    pub name: String,
    pub saved_at: String,
    pub content: String,
    #[serde(default)] pub source: Option<String>,
}
#[derive(Deserialize)]
pub struct Draft { pub name: String, pub content: String, #[serde(default)] pub source: Option<String>, #[serde(default)] pub legacy: bool }
pub fn fingerprint(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop { let count = file.read(&mut buffer).map_err(|e| e.to_string())?; if count == 0 { break; } digest.update(&buffer[..count]); }
    Ok(format!("{:x}", digest.finalize()))
}
fn validate(draft: &Draft) -> Result<(), String> {
    if draft.name.trim().is_empty() || draft.name.len() > 200 { return Err("Enter a build name (up to 200 characters)".into()); }
    if draft.content.trim().is_empty() || draft.content.len() > 4 * 1024 * 1024 { return Err("Build is empty or exceeds 4 MiB".into()); }
    Ok(())
}
pub fn handle(session: &ViewerSession, method: &str, body: &[u8]) -> Result<serde_json::Value, String> {
    Uuid::parse_str(&session.project_id).map_err(|e| e.to_string())?;
    let root = library::data_root_dir().join("save/builds").join(&session.project_id);
    handle_at(&root, session, method, body)
}
fn handle_at(root: &Path, session: &ViewerSession, method: &str, body: &[u8]) -> Result<serde_json::Value, String> {
    let current = session.fingerprint.clone();
    if method == "POST" {
        let draft: Draft = serde_json::from_slice(body).map_err(|e| e.to_string())?;
        validate(&draft)?;
        if draft.legacy { validate_legacy(session, &draft.content)?; }
        fs::create_dir_all(root).map_err(|e| e.to_string())?;
        if let Some(source)=draft.source.as_ref() {
            for path in fs::read_dir(root).map_err(|e|e.to_string())? {
                let path=path.map_err(|e|e.to_string())?.path();
                if path.extension().and_then(|e|e.to_str())!=Some("json") {continue;}
                let build: Build=serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                if build.source.as_ref()==Some(source) && build.content==draft.content {return serde_json::to_value(build).map_err(|e|e.to_string());}
            }
        }
        let build = Build { id: Uuid::new_v4().to_string(), project_id: session.project_id.clone(), project_name: session.project_name.clone(), fingerprint: if draft.legacy { "legacy-unknown".into() } else {current}, viewer_id: session.viewer_id.clone(), name: draft.name.trim().into(), saved_at: chrono::Utc::now().to_rfc3339(), content: draft.content, source: draft.source };
        let pending = root.join(format!(".{}.pending", build.id));
        fs::write(&pending, serde_json::to_vec_pretty(&build).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        fs::rename(pending, root.join(format!("{}.json", build.id))).map_err(|e| e.to_string())?;
        return serde_json::to_value(build).map_err(|e| e.to_string());
    }
    if method != "GET" { return Err("Unsupported build operation".into()); }
    let mut builds: Vec<Build> = Vec::new();
    if root.exists() {
        for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
            let build: Build = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            if build.project_id != session.project_id { return Err("Build belongs to another CYOA".into()); }
            builds.push(build);
        }
    }
    builds.sort_by(|a,b| b.saved_at.cmp(&a.saved_at));
    Ok(serde_json::json!({"projectId":session.project_id,"projectName":session.project_name,"fingerprint":current,"viewerId":session.viewer_id,"builds":builds}))
}
pub fn count(project_id: &str) -> usize {
    let root=library::data_root_dir().join("save/builds").join(project_id);
    fs::read_dir(root).map(|entries| entries.flatten().filter(|e| e.path().extension().is_some_and(|x|x=="json")).count()).unwrap_or(0)
}
pub fn copy_associated_builds(source_id: &str, target_id: &str, target_name: &str) -> Result<(), String> {
    copy_builds(source_id, target_id, target_name, None)
}
pub fn copy_edition_builds(source_id: &str, target_id: &str, target_name: &str, fingerprint: &str) -> Result<(), String> {
    copy_builds(source_id, target_id, target_name, Some(fingerprint))
}
fn copy_builds(source_id: &str, target_id: &str, target_name: &str, fingerprint: Option<&str>) -> Result<(), String> {
    let root=library::data_root_dir().join("save/builds");
    let source=root.join(source_id); let target=root.join(target_id);
    if !source.exists() {return Ok(());}
    fs::create_dir_all(&target).map_err(|e|e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e|e.to_string())? {
        let path=entry.map_err(|e|e.to_string())?.path();
        if path.extension().and_then(|e|e.to_str())!=Some("json") {continue;}
        let mut build: Build=serde_json::from_slice(&fs::read(&path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        if build.project_id!=source_id {return Err("Build source identity mismatch".into());}
        if fingerprint.is_some_and(|value| value != build.fingerprint) { continue; }
        build.project_id=target_id.into();build.project_name=target_name.into();
        let destination=target.join(path.file_name().ok_or("Build filename missing")?);
        if destination.exists() {continue;}
        fs::write(destination,serde_json::to_vec_pretty(&build).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    }
    Ok(())
}
fn choice_ids(json: &serde_json::Value) -> std::collections::BTreeSet<String> {
    let mut result=std::collections::BTreeSet::new();
    fn visit(value:&serde_json::Value,result:&mut std::collections::BTreeSet<String>) {
        match value {
            serde_json::Value::Object(map)=>{for key in ["id","uid"] {if let Some(id)=map.get(key).and_then(|v|v.as_str()) {result.insert(id.into());}} for v in map.values(){if v.is_object() || v.is_array(){visit(v,result);}}},
            serde_json::Value::Array(items)=>for v in items{visit(v,result)}, _=>{}
        }
    }
    visit(json,&mut result);result
}
fn validate_legacy(session: &ViewerSession, content: &str) -> Result<(), String> {
    let tokens: std::collections::BTreeSet<String>=content.split(',').filter_map(|t| {
        let token=t.trim().split('/').next()?.split('#').next()?.split('[').next()?.trim();
        if token.is_empty(){None}else{Some(token.into())}
    }).collect();
    if tokens.is_empty(){return Err("Legacy save has no identifiable choices; associate it manually".into());}
    let opened:serde_json::Value=serde_json::from_slice(&fs::read(&session.file_path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let opened_ids=choice_ids(&opened);
    if !tokens.iter().all(|id|opened_ids.contains(id)){return Err("Legacy save requires another edition; review its archive before importing".into());}
    let mut projects=library::load_library()?.library.projects;
    projects.extend(crate::history::all_versions()?.into_iter().map(|v|v.project));
    let mut matched=std::collections::BTreeSet::new();
    for project in projects {
        let Ok(bytes)=fs::read(&project.file_path) else {continue};
        let Ok(json)=serde_json::from_slice(&bytes) else {continue};
        let ids=choice_ids(&json);
        if tokens.iter().all(|id|ids.contains(id)){matched.insert(project.id);}
    }
    if matched.len()==1 && matched.contains(&session.project_id){Ok(())}
    else{Err("This legacy save cannot be uniquely matched to this CYOA; review and associate it manually".into())}
}
#[cfg(test)]
include!("../../tests/rust/builds.rs");
