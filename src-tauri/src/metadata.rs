use crate::{models::{Project, ProjectMetadata}, commands::LibraryState, library};
use serde_json::Value;
use tauri::{Manager, State};
fn text(json: &Value, keys: &[&str]) -> String {
    for parent in [json, &json["metadata"]] {
        for key in keys {
            if let Some(value) = parent[*key].as_str().filter(|v| !v.trim().is_empty()) { return plain(value); }
        }
    }
    String::new()
}
fn plain(value: &str) -> String {
    let regex = regex::Regex::new(r"<[^>]*>").unwrap();
    regex.replace_all(value, " ").split_whitespace().collect::<Vec<_>>().join(" ")
        .replace("&amp;", "&").replace("&nbsp;", " ")
}
pub fn description(json: &Value) -> String {
    let direct = text(json, &["description", "projectDescription", "summary"]);
    if !direct.is_empty() { return direct; }
    json["rows"].as_array().and_then(|r| r.first()).and_then(|r| r["text"].as_str())
        .map(plain).unwrap_or_default().chars().take(1200).collect()
}
pub fn icc2_compatible(json: &Value) -> bool {
    json["pointTypes"].is_array() && json["styling"].is_object()
        && json["rows"].as_array().is_some_and(|rows| rows.iter().all(|r| r["objects"].is_array()
        && r["objects"].as_array().unwrap().iter().all(|c| c["id"].is_string())))
}
pub fn infer(json: &Value, name: &str) -> ProjectMetadata {
    let (author_from_name, title_from_name) = name.split_once(':').unwrap_or(("", name));
    let mut title = text(json, &["title", "name", "projectName"]);
    if title.is_empty() { title = title_from_name.trim().into(); }
    let mut author = text(json, &["author", "creator"]);
    if author.is_empty() { author = author_from_name.trim().into(); }
    let mut fandom = text(json, &["fandom", "universe"]);
    if fandom.is_empty() {
        let lower = title.to_lowercase();
        for (needle, label) in [("naruto", "Naruto"), ("bleach", "Bleach"), ("fire emblem", "Fire Emblem"), ("nasu", "Nasuverse"), ("dxd", "High School DxD"), ("cyberpunk", "Cyberpunk"), ("re:zero", "Re:Zero"), ("re-zero", "Re:Zero"), ("god of war", "God of War"), ("game of thrones", "Game of Thrones"), ("final fantasy", "Final Fantasy")] {
            if lower.contains(needle) { fandom = label.into(); break; }
        }
    }
    let completion = text(json, &["completion", "status", "projectStatus"]);
    let lower = name.to_lowercase();
    let is_mod = json["is_mod"].as_bool().or(json["isMod"].as_bool()).unwrap_or(lower.contains("(mod)") || lower.contains(" mod") || lower.contains("remix"));
    let viewer_check = if icc2_compatible(json) { "ICC2 Plus format check passed" } else if json["rows"].as_array().is_some_and(|r| r.iter().any(|r| r["perks"].is_array())) { "Om1cr0n format" } else { "ICC2 Plus format needs review; legacy viewer available" };
    let (author, inferred_modder)=split_credits(&author);
    let explicit_modder=text(json,&["modder","modAuthor"]);
    ProjectMetadata { title, author, modder: if explicit_modder.is_empty(){inferred_modder}else{explicit_modder}, kind:String::new(),metadata_revision:2,fandom, completion, is_mod, viewer_check: viewer_check.into() }
}
pub fn split_credits(value:&str)->(String,String){
    let re=regex::Regex::new(r"(?i)^(.+?)\s*[-–—]\s*(.+?)\s*\(mod\)\s*$").unwrap();
    if let Some(c)=re.captures(value){return(c[1].trim().into(),c[2].trim().into());}(value.trim().into(),String::new())
}
pub fn normalize_tags(tags:&[String])->Vec<String>{
    let brackets=regex::Regex::new(r"\[([^\]]+)\]").unwrap();let mut result=Vec::new();let mut seen=std::collections::BTreeSet::new();
    for value in tags {let expanded=brackets.replace_all(value,"\n$1\n");for piece in expanded.split(['\n',',',';','|']){let clean=piece.split_whitespace().collect::<Vec<_>>().join(" ");if clean.is_empty(){continue;}
        let key=clean.to_lowercase().replace([' ',':','_','-'],"");let label=match key.as_str(){"oc"=>"Original Character".into(),"fempov"|"femalepov"=>"PoV: Female".into(),"malepov"=>"PoV: Male".into(),"neutralpov"=>"PoV: Neutral".into(),"eventbased"=>"Event Based".into(),"worldpicker"=>"World Picker".into(),"itempicker"=>"Item Picker".into(),"powerpicker"=>"Power Picker".into(),"sfw"=>"SFW".into(),"nsfw"=>"NSFW".into(),"mod"=>"MOD".into(),_=>clean.split(' ').map(|w|{let mut c=w.chars();match c.next(){Some(first)=>first.to_uppercase().collect::<String>()+c.as_str(),None=>String::new()}}).collect::<Vec<_>>().join(" ")};
        if seen.insert(label.to_lowercase()){result.push(label);}
    }}result
}
pub fn normalize_project(project:&mut Project){
    let(author,modder)=split_credits(&project.metadata.author);if !modder.is_empty(){project.metadata.author=author;if project.metadata.modder.is_empty(){project.metadata.modder=modder;}project.metadata.is_mod=true;}
    let redundant=[project.metadata.author.to_lowercase(),project.metadata.modder.to_lowercase(),project.metadata.fandom.to_lowercase()];
    project.tags=normalize_tags(&project.tags).into_iter().filter(|tag|!redundant.contains(&tag.to_lowercase())&&!(project.metadata.is_mod&&tag=="MOD")).collect();
    project.metadata.metadata_revision=2;
}
#[tauri::command]
pub async fn enrich_library_metadata(app: tauri::AppHandle) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<LibraryState>();
        let projects = state.lock().map_err(|e| e.to_string())?.projects.clone();
        let mut changed = 0;
        for project in projects {
            if project.metadata.metadata_revision>=2 && !project.metadata.title.is_empty(){continue;}
            let inferred=if project.metadata.title.is_empty() || project.metadata.viewer_check.is_empty(){
                let Ok(bytes)=std::fs::read(&project.file_path) else{continue};let Ok(json)=serde_json::from_slice::<Value>(&bytes) else{continue};Some((infer(&json,&project.name),description(&json)))
            }else{None};
            let mut lib=state.lock().map_err(|e|e.to_string())?;
            let Some(current)=lib.projects.iter_mut().find(|p|p.id==project.id)else{continue};
            if let Some((inferred,description))=inferred {
                if current.metadata.title.is_empty(){current.metadata.title=inferred.title;}
                if current.metadata.author.is_empty(){current.metadata.author=inferred.author;}
                if current.metadata.modder.is_empty(){current.metadata.modder=inferred.modder;}
                if current.metadata.fandom.is_empty(){current.metadata.fandom=inferred.fandom;}
                current.metadata.viewer_check=inferred.viewer_check;
                if current.description.is_empty(){current.description=description;}
            }
            normalize_project(current);
            library::update_project(current)?;
            changed += 1;
        }
        Ok(changed)
    }).await.map_err(|e| e.to_string())?
}
#[derive(serde::Serialize)]
pub struct DuplicateGroup { pub kind: String, pub projects: Vec<Project> }
#[tauri::command]
pub async fn find_duplicates(state: State<'_, LibraryState>) -> Result<Vec<DuplicateGroup>, String> {
    let projects = state.lock().map_err(|e| e.to_string())?.projects.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut exact: std::collections::BTreeMap<String, Vec<Project>> = Default::default();
        let mut related: std::collections::BTreeMap<String, Vec<Project>> = Default::default();
        for project in projects {
            if let Ok(fingerprint) = crate::builds::fingerprint(std::path::Path::new(&project.file_path)) { exact.entry(fingerprint).or_default().push(project.clone()); }
            let name = project.metadata.title.to_lowercase().replace(" - beforeupdate", "").replace(" - before update", "");
            let identity = project.project_json_url.clone().or(project.source_url.clone()).unwrap_or(name);
            related.entry(identity).or_default().push(project);
        }
        let mut result = Vec::new();
        for projects in exact.into_values().filter(|p| p.len()>1) { result.push(DuplicateGroup { kind:"Identical project files".into(), projects }); }
        for projects in related.into_values().filter(|p| p.len()>1) { result.push(DuplicateGroup { kind:"Related source or title — compare editions".into(), projects }); }
        Ok(result)
    }).await.map_err(|e| e.to_string())?
}
#[cfg(test)]
include!("../../tests/rust/metadata.rs");
