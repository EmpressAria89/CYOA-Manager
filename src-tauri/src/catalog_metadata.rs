//! Fill absent bibliographic fields from the existing public catalog, never from manual tags.
use std::{collections::BTreeMap,time::Duration};
use serde_json::Value;
use tauri::Manager;
use crate::{commands::LibraryState,models::Project};
const CATALOG_URL:&str="https://sheets.googleapis.com/v4/spreadsheets/1jxBbWB08myhD8YXePPifsWQG3JH2qZtBs9Y5yYcqE7g/values/Beta%20Index?key=AIzaSyBRhMRxRwP23DhvQdjCuk1saB5q2Xnp2kk";
#[derive(Clone,Default,PartialEq)]struct Credit{author:String,fandom:String,description:String}
fn source_key(value:&str)->Option<String>{let mut url=tauri::Url::parse(value).ok()?;if !matches!(url.scheme(),"https"|"http"){return None;}url.set_fragment(None);url.set_query(None);let path=url.path().trim_end_matches('/');let path=path.strip_suffix("/project.json").or_else(||path.strip_suffix("/index.html")).unwrap_or(path).to_string();url.set_path(&format!("{path}/"));Some(url.into())}
fn catalog(payload:&Value)->BTreeMap<String,Option<Credit>>{
 let mut result=BTreeMap::new();let Some(rows)=payload["values"].as_array() else{return result;};let Some(headers)=rows.first().and_then(Value::as_array)else{return result;};
 let columns:BTreeMap<&str,usize>=headers.iter().enumerate().filter_map(|(i,v)|v.as_str().map(|s|(s.trim(),i))).collect();
 for row in rows.iter().skip(1){let cell=|key:&str|columns.get(key).and_then(|&i|row[i].as_str()).unwrap_or("").trim().to_string();let Some(key)=source_key(&cell("Interactive"))else{continue};let credit=Credit{author:cell("Author"),fandom:cell("Universe"),description:cell("Description")};
  if let Some(previous)=result.get(&key){if previous!=&Some(credit.clone()){result.insert(key,None);}}else{result.insert(key,Some(credit));}
 }result
}
fn fill(project:&mut Project,credit:&Credit)->bool{let mut changed=false;
 for (field,value) in [(&mut project.metadata.author,&credit.author),(&mut project.metadata.fandom,&credit.fandom),(&mut project.description,&credit.description)]{if field.trim().is_empty()&&!value.is_empty(){*field=value.clone();changed=true;}}
 changed
}
#[tauri::command]
pub async fn enrich_catalog_metadata(app:tauri::AppHandle)->Result<usize,String>{
 tauri::async_runtime::spawn_blocking(move||{
  let state=app.state::<LibraryState>();let needed=state.lock().map_err(|e|e.to_string())?.projects.iter().any(|p|p.source_url.is_some()&&(p.metadata.author.is_empty()||p.metadata.fandom.is_empty()||p.description.is_empty()));if !needed{return Ok(0);}
  // Optional network enrichment must not prevent local startup when the catalog is unavailable.
  let fetched=(||->Result<Value,reqwest::Error>{reqwest::blocking::Client::builder().timeout(Duration::from_secs(8)).build()?.get(CATALOG_URL).send()?.error_for_status()?.json()})();
  let payload=match fetched{Ok(v)=>v,Err(e)=>{eprintln!("Catalog metadata unavailable: {e}");return Ok(0);}};let credits=catalog(&payload);let mut library=state.lock().map_err(|e|e.to_string())?;let mut changed=0;
  for project in &mut library.projects{let key=project.project_json_url.as_deref().or(project.source_url.as_deref()).and_then(source_key);let Some(Some(credit))=key.and_then(|key|credits.get(&key))else{continue};if fill(project,credit){crate::library::update_project(project)?;changed+=1;}}
  Ok(changed)
 }).await.map_err(|e|e.to_string())?
}
#[cfg(test)]
include!("../../tests/rust/catalog_metadata.rs");
