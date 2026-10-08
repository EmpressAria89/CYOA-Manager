//! Engine-independent website snapshots. Preserve remote-origin mode for sites requiring live APIs.
use std::{collections::{BTreeMap,VecDeque},fs,io::Read,path::Path,time::Duration};
use serde::{Serialize,Deserialize};
use sha2::{Digest,Sha256};
use tauri::{Manager,Emitter};
use crate::{commands::LibraryState,models::{Project,ProjectMetadata},library};
#[derive(Clone,Serialize,Deserialize)]
pub struct Resource {pub url:String,pub mime:String,pub hash:String}
#[derive(Serialize,Deserialize)]
pub struct Snapshot {pub website_snapshot:bool,pub url:String,pub title:String,pub resources:BTreeMap<String,Resource>,pub unavailable:Vec<String>}
fn asset_key(url:&str)->String{format!("__site/{:x}",Sha256::digest(url.as_bytes()))}
fn web_url(value:&str)->Result<tauri::Url,String>{let u=tauri::Url::parse(value).map_err(|e|e.to_string())?;if !matches!(u.scheme(),"http"|"https")||!u.username().is_empty()||u.password().is_some(){return Err("Enter an HTTP(S) website URL without credentials".into());}Ok(u)}
fn resolve(value:&str,base:&tauri::Url)->Option<String>{let mut u=base.join(value.trim()).ok()?;if !matches!(u.scheme(),"http"|"https"){return None;}u.set_fragment(None);Some(u.into())}
fn rewrite(source:&str,base:&tauri::Url,mime:&str,queue:&mut VecDeque<String>,known:&mut BTreeMap<String,String>)->String{
 let html=mime.contains("html");
 let mut rewritten=source.to_string();
 if html {
  // The mirror has a new origin and modified assets. Publisher policies remain intact in online mode.
  for pattern in [r#"(?is)<base\b[^>]*>"#,r#"(?is)<meta\b[^>]*http-equiv\s*=\s*["']content-security-policy["'][^>]*>"#,r#"(?is)\s+integrity\s*=\s*["'][^"']*["']"#] {
   rewritten=regex::Regex::new(pattern).unwrap().replace_all(&rewritten,"").into_owned();
  }
  let scripts=regex::Regex::new(r"(?is)(<script\b[^>]*>)(.*?)(</script>)").unwrap();
  rewritten=scripts.replace_all(&rewritten,|c:&regex::Captures|format!("{}{}{}",&c[1],rewrite(&c[2],base,"text/javascript",queue,known),&c[3])).into_owned();
 }
 let patterns=if html{vec![r#"(?i)(?:src|poster)\s*=\s*["']([^"']+)["']"#,r#"(?i)<link\b[^>]*href\s*=\s*["']([^"']+)["']"#,r#"(?i)url\(\s*["']?([^\s)'";]+)"#]}else if mime.contains("css"){vec![r#"(?i)url\(\s*["']?([^\s)'";]+)"#,r#"(?i)@import\s*["']([^"']+)["']"#]}else if mime.contains("javascript"){vec![r#"(?:\bfrom\s*|\bimport\s*\(?\s*|\bfetch\s*\(\s*|\bnew URL\s*\(\s*)["']([^"']+)["']"#]}else{vec![]};
 for pattern in patterns{let re=regex::Regex::new(pattern).unwrap();rewritten=re.replace_all(&rewritten,|cap:&regex::Captures|{
  let capture=cap.get(1).unwrap();let raw=capture.as_str();
  // Already rewritten references must not be fetched from the original website.
  if raw.starts_with("/__site/")||raw.starts_with('#'){return cap[0].to_string();}
  let Some(url)=resolve(raw,base)else{return cap[0].to_string()};
  let fragment=base.join(raw).ok().and_then(|u|u.fragment().map(str::to_string));
  if !known.contains_key(&url){known.insert(url.clone(),asset_key(&url));queue.push_back(url.clone());}
  let local=format!("/{}{}",known[&url],fragment.map(|f|format!("#{f}")).unwrap_or_default());
  let whole=cap.get(0).unwrap();let start=capture.start()-whole.start();let end=capture.end()-whole.start();
  format!("{}{}{}",&whole.as_str()[..start],local,&whole.as_str()[end..])
 }).into_owned();}
 if html {
  // Keep navigation usable without recursively downloading an unbounded website.
  let links=regex::Regex::new(r#"(?i)(<a\b[^>]*href\s*=\s*["'])([^"']+)(["'])"#).unwrap();
  rewritten=links.replace_all(&rewritten,|c:&regex::Captures|{
   if c[2].starts_with('#'){return c[0].to_string();}
   match base.join(&c[2]){Ok(url)=>format!("{}{}{}",&c[1],url,&c[3]),Err(_)=>c[0].to_string()}
  }).into_owned();
 }
 rewritten
}
struct Staging(std::path::PathBuf,bool);
impl Drop for Staging{fn drop(&mut self){if !self.1{let _=fs::remove_dir_all(&self.0);}}}
#[derive(Serialize)]
pub struct WebsiteDownload {project:Project,unavailable:Vec<String>}

fn download(url:&str,_title:&str,directory:&Path,max_bytes:u64)->Result<Snapshot,String>{
 let source=web_url(url)?;let client=reqwest::blocking::Client::builder().timeout(Duration::from_secs(25)).user_agent("CYOA-Manager website snapshot").build().map_err(|e|e.to_string())?;
 fs::create_dir_all(directory.join("__site")).map_err(|e|e.to_string())?;let mut queue=VecDeque::from([source.to_string()]);let mut known=BTreeMap::from([(source.to_string(),"index.html".to_string())]);let mut result=Snapshot{website_snapshot:true,url:source.to_string(),title:String::new(),resources:BTreeMap::new(),unavailable:Vec::new()};let mut total=0;
 while let Some(url)=queue.pop_front(){if result.resources.len()+result.unavailable.len()>=512{return Err("Website references more than 512 resources; use its live website mode".into());}
  let fetched=(||->Result<(String,Vec<u8>,tauri::Url),String>{let response=client.get(&url).send().map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?;let final_url=response.url().clone();let mime=response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|s|s.to_str().ok()).unwrap_or("application/octet-stream").to_string();let mut bytes=Vec::new();response.take(max_bytes.saturating_sub(total)+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;Ok((mime,bytes,final_url))})();
  let(mime,bytes,base)=match fetched{Ok(v)=>v,Err(e)=>{if known[&url]=="index.html"{return Err(e);}result.unavailable.push(url);continue;}};
  total+=bytes.len() as u64;if total>max_bytes{return Err("Website snapshot exceeds your download size limit; current library remains unchanged".into());}
  let bytes=if mime.contains("html")||mime.contains("css")||mime.contains("javascript"){rewrite(&String::from_utf8_lossy(&bytes),&base,&mime,&mut queue,&mut known).into_bytes()}else{bytes};let path=known[&url].clone();fs::write(directory.join(&path),&bytes).map_err(|e|e.to_string())?;
  result.resources.insert(path,Resource{url,mime,hash:format!("{:x}",Sha256::digest(&bytes))});
 }
 fs::write(directory.join("project.json"),serde_json::to_vec_pretty(&result).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;Ok(result)
}
pub fn read(path:&Path)->Result<Snapshot,String>{serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())}
#[tauri::command]
pub async fn download_website(app:tauri::AppHandle,url:String,title:String,author:String,fandom:String,description:String,max_size_mb:u64,existing_project_id:Option<String>,force_update:Option<bool>)->Result<WebsiteDownload,String>{
 tauri::async_runtime::spawn_blocking(move||{
  if let Some(id)=&existing_project_id {
   let state=app.state::<LibraryState>();let library=state.lock().map_err(|e|e.to_string())?;
   let project=library.projects.iter().find(|p| &p.id==id).ok_or("Website project not found")?;
   if !force_update.unwrap_or(false){crate::history::ensure_current_edition(project)?;}
  }
  let directory=library::cyoas_dir().join(format!("website-{}",uuid::Uuid::new_v4()));
  let mut staging=Staging(directory.clone(),false);
  let snapshot=match download(&url,&title,&directory,max_size_mb.max(1).saturating_mul(1024*1024)){Ok(v)=>v,Err(e)=>{let _=fs::remove_dir_all(&directory);return Err(e);}};
  let state=app.state::<LibraryState>();let previous=if let Some(id)=existing_project_id{Some(state.lock().map_err(|e|e.to_string())?.projects.iter().find(|p|p.id==id).cloned().ok_or("Website project not found")?)}else{None};
  let mut project=previous.clone().unwrap_or(Project{metadata:ProjectMetadata{title:title.clone(),author,fandom,kind:"website".into(),metadata_revision:2,..Default::default()},build_count:0,id:uuid::Uuid::new_v4().to_string(),name:title,description,cover_image:None,source_url:Some(snapshot.url.clone()),project_json_url:None,file_path:String::new(),viewer_preference:Some("website".into()),favorite:false,exclude_from_perk_index:true,date_added:chrono::Utc::now().to_rfc3339(),tags:vec![],file_missing:false});
  project.metadata.kind="website".into();project.viewer_preference=Some("website".into());project.project_json_url=None;project.source_url=Some(snapshot.url.clone());project.exclude_from_perk_index=true;
  crate::metadata::normalize_project(&mut project);
  project.file_path=directory.join("project.json").to_string_lossy().into_owned();
  if let Some(previous)=&previous {
   let diff=crate::update_diff::compare_files(Path::new(&previous.file_path),Path::new(&project.file_path))?;
   if !diff.changed&&!force_update.unwrap_or(false){crate::history::remove_retired_files(&project,&state)?;let _=app.emit("project-update-result",serde_json::json!({"projectName":previous.name,"diff":diff}));return Ok(WebsiteDownload{project:previous.clone(),unavailable:snapshot.unavailable});}
   let version=crate::history::snapshot(previous,if force_update.unwrap_or(false){"Before forced website update"}else{"Before changed website update"})?;crate::history::pin_sessions(&app,previous,&version)?;
   project.metadata.restored_from_archive=false;
   library::update_project(&project)?;let mut lib=state.lock().map_err(|e|e.to_string())?;if let Some(p)=lib.projects.iter_mut().find(|p|p.id==project.id){*p=project.clone();}drop(lib);
   crate::history::remove_retired_files(previous,&state)?;crate::history::apply_retention(&app,&project.id)?;let _=app.emit("project-update-result",serde_json::json!({"projectName":project.name,"diff":diff}));
  }else{library::insert_project(&project)?;state.lock().map_err(|e|e.to_string())?.projects.push(project.clone());}
  let _=crate::perk_index::remove_project_from_index_if_present(&project.id);
  staging.1=true;
  Ok(WebsiteDownload{project,unavailable:snapshot.unavailable})
 }).await.map_err(|e|e.to_string())?
}
#[cfg(test)]
include!("../../tests/rust/websites.rs");
