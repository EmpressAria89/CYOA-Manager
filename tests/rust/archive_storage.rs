#[cfg(test)]mod tests{use super::*;
 #[test]fn lossless_roundtrip_and_corruption_keep_original(){let root=std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());let files=root.join("files");fs::create_dir_all(files.join("images")).unwrap();fs::write(files.join("export.json"),br#"{"image":"images/a.png"}"#).unwrap();fs::write(files.join("images/a.png"),[0,255,1,2]).unwrap();let expected=hashes(&files).unwrap();assert!(pack(&root,true).unwrap());assert!(!files.exists());with_files(&files.join("export.json"),||{assert_eq!(hashes(&files).unwrap(),expected);Ok(())}).unwrap();assert!(!pack(&root,true).unwrap());fs::write(root.join("files.zip"),b"corrupt").unwrap();assert!(with_files(&files.join("export.json"),||Ok(())).is_err());assert!(root.join("files.zip").exists());fs::remove_dir_all(root).unwrap();}
 #[test]fn changed_cache_is_never_deleted(){let root=std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());fs::create_dir_all(root.join("files")).unwrap();fs::write(root.join("files/project.json"),b"old").unwrap();pack(&root,false).unwrap();fs::write(root.join("files/project.json"),b"changed").unwrap();assert!(pack(&root,true).is_err());assert_eq!(fs::read(root.join("files/project.json")).unwrap(),b"changed");fs::remove_dir_all(root).unwrap();}
}

#[test]
fn startup_skips_completed_archives_but_restore_still_checks_integrity(){
 let root=std::env::temp_dir().join(format!("cyoa-startup-pack-{}",uuid::Uuid::new_v4()));std::fs::create_dir_all(root.join("files")).unwrap();std::fs::write(root.join("files/project.json"),"fixture").unwrap();
 assert!(pack_if_needed(&root,true).unwrap());assert!(!pack_if_needed(&root,true).unwrap());
 std::fs::write(root.join("files.zip"),"corrupt").unwrap();assert!(!pack_if_needed(&root,true).unwrap());
 assert!(with_files(&root.join("files/project.json"),||Ok(())).is_err());assert!(root.join("files.zip").is_file());std::fs::remove_dir_all(root).unwrap();
}
