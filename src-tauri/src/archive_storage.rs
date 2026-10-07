//! Lossless archive storage, verified before retiring extracted files.
use std::{collections::BTreeMap,fs,path::Path,sync::Mutex};
use sha2::{Digest,Sha256};
static STORAGE:Mutex<()>=Mutex::new(());
fn hashes(root:&Path)->Result<BTreeMap<String,String>,String>{
 let mut result=BTreeMap::new();
 for entry in walkdir::WalkDir::new(root).follow_links(false){let entry=entry.map_err(|e|e.to_string())?;if entry.file_type().is_symlink(){return Err("Archive contains a symbolic link".into());}if !entry.file_type().is_file(){continue;}
  let name=entry.path().strip_prefix(root).map_err(|e|e.to_string())?.to_string_lossy().replace('\\',"/");let mut input=fs::File::open(entry.path()).map_err(|e|e.to_string())?;let mut hash=Sha256::new();std::io::copy(&mut input,&mut hash).map_err(|e|e.to_string())?;result.insert(name,format!("{:x}",hash.finalize()));
 }Ok(result)
}
fn verify_zip(path:&Path,expected:&BTreeMap<String,String>)->Result<(),String>{
 let mut zip=zip::ZipArchive::new(fs::File::open(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;let mut found=BTreeMap::new();
 for i in 0..zip.len(){let mut entry=zip.by_index(i).map_err(|e|e.to_string())?;if entry.is_dir(){continue;}let name=entry.enclosed_name().ok_or("Invalid archive path")?.to_string_lossy().replace('\\',"/");let mut hash=Sha256::new();std::io::copy(&mut entry,&mut hash).map_err(|e|e.to_string())?;if found.insert(name,format!("{:x}",hash.finalize())).is_some(){return Err("Duplicate archive path".into());}}
 if &found!=expected{return Err("Archive content verification failed; original retained".into());}Ok(())
}
fn pack_unlocked(root:&Path,retire:bool)->Result<bool,String>{
 let files=root.join("files");let zip_path=root.join("files.zip");let manifest=root.join("checksums.json");
 if zip_path.exists(){let expected=serde_json::from_slice(&fs::read(&manifest).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;verify_zip(&zip_path,&expected)?;if retire&&files.exists(){if hashes(&files)?!=expected{return Err("Extracted archive changed; retained for review".into());}fs::remove_dir_all(files).map_err(|e|e.to_string())?;}return Ok(false);}
 let expected=hashes(&files)?;if expected.is_empty(){return Err("Archive has no project files".into());}
 let pending=root.join("files.zip.pending");
 let result=(||{
  let mut writer=zip::ZipWriter::new(fs::File::create(&pending).map_err(|e|e.to_string())?);
  let options=zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).compression_level(Some(6));
  for name in expected.keys(){writer.start_file(name,options).map_err(|e|e.to_string())?;let mut input=fs::File::open(files.join(name)).map_err(|e|e.to_string())?;std::io::copy(&mut input,&mut writer).map_err(|e|e.to_string())?;}
  writer.finish().map_err(|e|e.to_string())?.sync_all().map_err(|e|e.to_string())?;verify_zip(&pending,&expected)?;
  let checksum_pending=root.join("checksums.json.pending");fs::write(&checksum_pending,serde_json::to_vec_pretty(&expected).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
  fs::rename(checksum_pending,manifest).map_err(|e|e.to_string())?;fs::rename(&pending,zip_path).map_err(|e|e.to_string())?;
  if retire{fs::remove_dir_all(files).map_err(|e|e.to_string())?;}Ok(true)
 })();if result.is_err(){let _=fs::remove_file(pending);}result
}
pub fn pack(root:&Path,retire:bool)->Result<bool,String>{let _guard=STORAGE.lock().map_err(|e|e.to_string())?;pack_unlocked(root,retire)}
fn extract_unlocked(project:&Path)->Result<(),String>{
 let files=project.parent().ok_or("Archive files folder missing")?;let root=files.parent().ok_or("Archive folder missing")?;let zip_path=root.join("files.zip");
 if !zip_path.exists(){if project.is_file(){return Ok(());}return Err("Archived project is missing".into());}
 let expected:BTreeMap<String,String>=serde_json::from_slice(&fs::read(root.join("checksums.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
 if files.exists(){if hashes(files)?==expected&&project.is_file(){return Ok(());}return Err("Extracted archive changed; original compressed copy retained".into());}
 verify_zip(&zip_path,&expected)?;let pending=root.join(".extracting");if pending.exists(){fs::remove_dir_all(&pending).map_err(|e|e.to_string())?;}fs::create_dir(&pending).map_err(|e|e.to_string())?;
 let result=(||{let mut zip=zip::ZipArchive::new(fs::File::open(zip_path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
  for i in 0..zip.len(){let mut entry=zip.by_index(i).map_err(|e|e.to_string())?;if entry.is_dir(){continue;}let target=pending.join(entry.enclosed_name().ok_or("Invalid archive path")?);fs::create_dir_all(target.parent().ok_or("Missing asset directory")?).map_err(|e|e.to_string())?;let mut output=fs::File::create(target).map_err(|e|e.to_string())?;std::io::copy(&mut entry,&mut output).map_err(|e|e.to_string())?;}
  if hashes(&pending)?!=expected{return Err("Extracted archive verification failed".into());}if !pending.join(project.file_name().ok_or("Missing project filename")?).is_file(){return Err("Project missing from archive".into());}fs::rename(&pending,files).map_err(|e|e.to_string())?;Ok(())})();if result.is_err(){let _=fs::remove_dir_all(pending);}result
}
pub fn with_files<T>(project:&Path,action:impl FnOnce()->Result<T,String>)->Result<T,String>{let _guard=STORAGE.lock().map_err(|e|e.to_string())?;extract_unlocked(project)?;action()}
#[cfg(test)]
include!("../../tests/rust/archive_storage.rs");
