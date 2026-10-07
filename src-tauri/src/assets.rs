use std::{collections::{BTreeMap,BTreeSet},fs,path::{Path,PathBuf,Component}};
// Folder projects own their full tree. Standalone exports own only referenced local files.
pub fn files(project:&Path)->Result<BTreeMap<String,PathBuf>,String>{
 let base=project.parent().ok_or("Missing project folder")?;let mut result=BTreeMap::new();
 if project.file_name().is_some_and(|n|n=="project.json") {
  for item in walkdir::WalkDir::new(base).follow_links(false){let item=item.map_err(|e|e.to_string())?;
   if item.file_type().is_symlink(){return Err("Project contains a symbolic link; original retained".into());}
   if item.file_type().is_file()&&item.path()!=project{result.insert(item.path().strip_prefix(base).map_err(|e|e.to_string())?.to_string_lossy().into_owned(),item.path().into());}
  }
 }else{
  let json:serde_json::Value=serde_json::from_slice(&fs::read(project).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
  let mut strings=BTreeSet::new();fn visit(v:&serde_json::Value,out:&mut BTreeSet<String>){match v{serde_json::Value::String(s)=>{out.insert(s.clone());},serde_json::Value::Array(a)=>for v in a{visit(v,out)},serde_json::Value::Object(o)=>for v in o.values(){visit(v,out)},_=>{}}}visit(&json,&mut strings);
  let re=regex::Regex::new(r#"(?:src|href)\s*=\s*["']([^"']+)["']"#).unwrap();let mut candidates=strings.clone();
  for s in strings{for cap in re.captures_iter(&s){candidates.insert(cap[1].into());}}
  for raw in candidates{if raw.starts_with("data:")||raw.contains("://")||raw.starts_with('#'){continue;}
   let clean=raw.split(['?','#']).next().unwrap_or("");let relative=Path::new(clean);if clean.is_empty(){continue;}
   let candidate=base.join(relative);if !candidate.is_file(){continue;}
   if relative.components().any(|c|matches!(c,Component::ParentDir|Component::RootDir|Component::Prefix(_))){return Err(format!("Local asset outside project folder: {clean}; original retained"));}
   if fs::symlink_metadata(&candidate).map_err(|e|e.to_string())?.file_type().is_symlink(){return Err("Local asset is a symbolic link; original retained".into());}
   let canonical=candidate.canonicalize().map_err(|e|e.to_string())?;let canonical_base=base.canonicalize().map_err(|e|e.to_string())?;if !canonical.starts_with(canonical_base){return Err("Local asset resolves outside project folder; original retained".into());}
   let mut ancestor=candidate.as_path();while ancestor!=base{if fs::symlink_metadata(ancestor).map_err(|e|e.to_string())?.file_type().is_symlink(){return Err("Local asset path contains a symbolic link; original retained".into());}ancestor=ancestor.parent().ok_or("Asset outside folder")?;}
   if candidate!=project{result.insert(clean.into(),candidate);}
  }
 }Ok(result)
}
pub fn manifest(project:&Path)->Result<BTreeMap<String,String>,String>{files(project)?.into_iter().map(|(name,path)|Ok((name,crate::builds::fingerprint(&path)?))).collect()}
pub fn copy(project:&Path,destination:&Path)->Result<(),String>{
 let entries=files(project)?;for(relative,source)in entries{let target=destination.join(relative);fs::create_dir_all(target.parent().ok_or("Asset folder missing")?).map_err(|e|e.to_string())?;fs::copy(&source,&target).map_err(|e|e.to_string())?;if crate::builds::fingerprint(&source)?!=crate::builds::fingerprint(&target)?{return Err("Archive asset verification failed".into());}}Ok(())
}

#[cfg(test)]
include!("../../tests/rust/assets.rs");
