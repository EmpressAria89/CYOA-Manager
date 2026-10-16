//! Fill absent bibliographic fields from the existing public catalog, never from manual tags.
use std::{collections::BTreeMap,time::{Duration,Instant},sync::Mutex};
use serde_json::Value;
use tauri::Manager;
use crate::{commands::LibraryState,models::Project};
// Public Google Visualization data endpoint; no account or API credential required.
const CATALOG_URL:&str="https://docs.google.com/spreadsheets/d/1jxBbWB08myhD8YXePPifsWQG3JH2qZtBs9Y5yYcqE7g/gviz/tq?tqx=out:json&sheet=Beta%20Index&headers=1";
static ROWS:Mutex<Option<(Instant,Vec<Vec<String>>)>>=Mutex::new(None);
fn parse_rows(source:&str)->Result<Vec<Vec<String>>,String>{
 let json=source.split_once("setResponse(").and_then(|(_,tail)|tail.trim().strip_suffix(");")).ok_or("Public catalog response has an unsupported format")?;
 let payload:Value=serde_json::from_str(json).map_err(|e|e.to_string())?;
 if payload["status"]!="ok" {return Err("Public catalog returned an error".into());}
 let cols=payload["table"]["cols"].as_array().ok_or("Catalog columns missing")?;
 let mut rows:Vec<Vec<String>>=vec![cols.iter().map(|c|c["label"].as_str().unwrap_or("").to_string()).collect()];
 for row in payload["table"]["rows"].as_array().ok_or("Catalog rows missing")? {
  let cells=row["c"].as_array().ok_or("Invalid catalog cells")?;
  rows.push((0..cols.len()).map(|i|{
   let Some(cell)=cells.get(i)else{return String::new()};
   if let Some(value)=cell["f"].as_str().or_else(||cell["v"].as_str()){value.to_string()}
   else if cell["v"].is_null(){String::new()}else{cell["v"].to_string()}
  }).collect());
 }
 if rows.len()<2 || !rows[0].iter().any(|h|h=="Interactive"){return Err("Catalog has no usable rows or Interactive column".into());}
 Ok(rows)
}
#[tauri::command(async)]
pub fn get_catalog_rows()->Result<Vec<Vec<String>>,String>{
 // Serialize the optional fetch so opening the catalog during startup shares it.
 let mut cached=ROWS.lock().map_err(|e|e.to_string())?;
 if let Some((created,rows))=&*cached {if created.elapsed()<Duration::from_secs(300){return Ok(rows.clone());}}
 let source=reqwest::blocking::Client::builder().timeout(Duration::from_secs(15)).build().map_err(|e|e.to_string())?.get(CATALOG_URL).send().and_then(reqwest::blocking::Response::error_for_status).map_err(|e|e.to_string())?.text().map_err(|e|e.to_string())?;
 let rows=parse_rows(&source)?;*cached=Some((Instant::now(),rows.clone()));Ok(rows)
}

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
  let rows=match get_catalog_rows(){Ok(rows)=>rows,Err(e)=>{eprintln!("Catalog metadata unavailable: {e}");return Ok(0);}};
  let credits=catalog(&serde_json::json!({"values":rows}));let mut library=state.lock().map_err(|e|e.to_string())?;let mut changed=0;
  for project in &mut library.projects{let key=project.project_json_url.as_deref().or(project.source_url.as_deref()).and_then(source_key);let Some(Some(credit))=key.and_then(|key|credits.get(&key))else{continue};if fill(project,credit){crate::library::update_project(project)?;changed+=1;}}
  Ok(changed)
 }).await.map_err(|e|e.to_string())?
}
#[cfg(test)]
include!("../../tests/rust/catalog_metadata.rs");
