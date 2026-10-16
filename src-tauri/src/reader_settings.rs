//! Display preferences only: project exports, build codes and game state are never changed.
use std::{collections::BTreeMap, sync::Mutex};
use serde::{Deserialize, Serialize};
static WRITE: Mutex<()> = Mutex::new(());
#[derive(Clone,Default,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct TextStyle {pub font: String, pub size: u16}
#[derive(Clone,Default,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields,rename_all="camelCase")]
pub struct Display {pub parts:BTreeMap<String,TextStyle>,pub scale:Option<u16>,pub color_scores:Option<bool>,pub auto_hide_sections:Option<bool>,pub hidden_rows:Option<Vec<String>>}
#[derive(Default,Serialize,Deserialize)]
#[serde(default)]
struct Store {global:Display,projects:BTreeMap<String,Display>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields,rename_all="camelCase")]
struct Change {scope:String,section:String,#[serde(default)] reset:bool,font:Option<String>,size:Option<u16>,scale:Option<u16>,color_scores:Option<bool>,auto_hide_sections:Option<bool>,hidden_rows:Option<Vec<String>>}
fn path()->std::path::PathBuf {crate::library::data_root_dir().join("save/reader-settings.json")}
fn read()->Result<Store,String>{match std::fs::read(path()){Ok(b)=>serde_json::from_slice(&b).map_err(|e|format!("Reader settings could not be read; original kept: {e}")),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(Store::default()),Err(e)=>Err(e.to_string())}}
fn effective(store:&Store,id:&str,prefs:&crate::preferences::Preferences,mode:&str)->Display {
 let mut result=store.global.clone();
 result.parts.entry("body".into()).or_default().font=crate::preferences::reader_font(prefs,mode);
 if mode=="author" {for part in result.parts.values_mut(){part.font.clear();}}
 if let Some(local)=store.projects.get(id){result.parts.extend(local.parts.clone());if local.auto_hide_sections.is_some(){result.auto_hide_sections=local.auto_hide_sections;}result.hidden_rows=local.hidden_rows.clone();if local.scale.is_some(){result.scale=local.scale;}if local.color_scores.is_some(){result.color_scores=local.color_scores;}}
 for part in result.parts.values_mut(){if part.size==0{part.size=100;}}
 result
}
fn apply(store:&mut Store,id:&str,prefs:&mut crate::preferences::Preferences,change:Change)->Result<bool,String>{
 if !["global","project"].contains(&change.scope.as_str()) || !["body","title","description","score","layout","sections"].contains(&change.section.as_str()){return Err("Invalid reader setting scope or section".into());}
 if change.font.as_ref().is_some_and(|s|s.len()>200||s.chars().any(char::is_control)) || change.size.is_some_and(|v|!(50..=250).contains(&v)) || change.scale.is_some_and(|v|!(50..=150).contains(&v)){return Err("Font or display size is out of range".into());}
 if change.hidden_rows.as_ref().is_some_and(|ids|ids.len()>10000||ids.iter().any(|id|id.len()>200||id.chars().any(char::is_control))){return Err("Invalid hidden section IDs".into());}
 if change.scope=="global"&&change.hidden_rows.is_some(){return Err("Hidden sections belong to one CYOA".into());}
 let global=change.scope=="global";
 let target=if global{&mut store.global}else{store.projects.entry(id.into()).or_default()};
 if change.section=="sections" {
  if change.reset {target.auto_hide_sections=None;target.hidden_rows=None;}
  else {target.auto_hide_sections=change.auto_hide_sections;if !global{target.hidden_rows=change.hidden_rows;}}
 } else if change.section=="layout" {
  if change.reset {target.scale=None;target.color_scores=None;}
  else {target.scale=change.scale;target.color_scores=change.color_scores;}
 } else if change.reset {target.parts.remove(&change.section);if global&&change.section=="body"{prefs.use_author_fonts=true;}}
 else {
  let mut part=TextStyle{font:change.font.ok_or("Choose a font or author default")?,size:change.size.unwrap_or(100)};
  if global&&change.section=="body" {prefs.use_author_fonts=part.font.is_empty();if !part.font.is_empty(){prefs.cyoa_font=part.font.clone();}part.font.clear();}
  target.parts.insert(change.section,part);
 }
 Ok(global)
}
pub fn handle(session:&crate::models::ViewerSession,method:&str,body:&[u8],app:&tauri::AppHandle)->Result<serde_json::Value,String>{
 let _guard=WRITE.lock().map_err(|e|e.to_string())?;let mut store=read()?;let mut prefs=crate::preferences::load();
 if method=="POST" {
  let change:Change=serde_json::from_slice(body).map_err(|e|e.to_string())?;
  let global=apply(&mut store,&session.project_id,&mut prefs,change)?;
  let file=path();std::fs::create_dir_all(file.parent().unwrap()).map_err(|e|e.to_string())?;
  let pending=file.with_extension(format!("{}.pending",uuid::Uuid::new_v4()));std::fs::write(&pending,serde_json::to_vec_pretty(&store).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
  std::fs::rename(pending,file).map_err(|e|e.to_string())?;
  if global {crate::preferences::set_preferences(prefs.clone(),app.clone())?;use tauri::Emitter;let _=app.emit("manager-reader-defaults-changed",&prefs);}
 } else if method!="GET" {return Err("Unsupported reader settings method".into());}
 let mut global=store.global.clone();let part=global.parts.entry("body".into()).or_default();part.font=crate::preferences::reader_font(&prefs,"");if part.size==0{part.size=100;}
 Ok(serde_json::json!({"theme":session.theme,"global":global,"local":store.projects.get(&session.project_id).cloned().unwrap_or_default(),"effective":effective(&store,&session.project_id,&prefs,&session.font_preference)}))
}
include!("../../tests/rust/reader_settings.rs");
