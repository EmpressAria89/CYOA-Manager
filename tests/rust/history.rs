#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retention_keeps_starred_and_open_versions() {
        let root=std::env::temp_dir().join(Uuid::new_v4().to_string());fs::create_dir_all(&root).unwrap();let file=root.join("export.json");fs::write(&file,b"{}").unwrap();
        let project=Project{metadata:Default::default(),build_count:0,id:Uuid::new_v4().to_string(),name:"Test".into(),description:String::new(),cover_image:None,source_url:None,project_json_url:None,file_path:file.to_string_lossy().into_owned(),viewer_preference:None,favorite:false,exclude_from_perk_index:false,date_added:String::new(),tags:vec![],file_missing:false};
        let history=root.join("history");let mut versions=(0..7).map(|_|snapshot_at(&history,&project,"test").unwrap()).collect::<Vec<_>>();versions[0].starred=true;let protected=vec![versions[1].project.file_path.clone()];let star=versions[0].id.clone();let open=versions[1].id.clone();assert_eq!(prune_list(&history,versions,3,&protected).unwrap(),2);assert!(history.join(star).exists());assert!(history.join(open).exists());assert_eq!(fs::read_dir(&history).unwrap().count(),5);fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn archive_is_complete_and_independent() {
        let root = std::env::temp_dir().join(format!("cyoa-history-test-{}", Uuid::new_v4()));
        let source = root.join("source");
        fs::create_dir_all(source.join("images")).unwrap();
        fs::write(source.join("project.json"), b"{\"rows\":[]}").unwrap();
        fs::write(source.join("images/cover.png"), b"old image").unwrap();
        let project = Project { metadata: Default::default(), build_count: 0, id: Uuid::new_v4().to_string(), name: "Naruto".into(), description: String::new(), cover_image: None, source_url: None, project_json_url: None, file_path: source.join("project.json").to_string_lossy().into_owned(), viewer_preference: None, favorite: true, exclude_from_perk_index: false, date_added: String::new(), tags: vec![], file_missing: false };
        let version = snapshot_at(&root.join("versions"), &project, "test").unwrap();
        fs::write(source.join("project.json"), b"new").unwrap();
        fs::write(source.join("images/cover.png"), b"new image").unwrap();
        assert!(!Path::new(&version.project.file_path).exists());
        crate::archive_storage::with_files(Path::new(&version.project.file_path),||Ok(())).unwrap();
        assert_eq!(fs::read(&version.project.file_path).unwrap(), b"{\"rows\":[]}");
        assert_eq!(fs::read(Path::new(&version.project.file_path).parent().unwrap().join("images/cover.png")).unwrap(), b"old image");
        let restored = copy_project(&version.project, &root.join("restored")).unwrap();
        assert_eq!(fs::read(&restored).unwrap(), b"{\"rows\":[]}");
        assert_eq!(fs::read(restored.parent().unwrap().join("images/cover.png")).unwrap(), b"old image");
        let another = snapshot_at(&root.join("versions"), &project, "second").unwrap();
        assert_ne!(version.id, another.id);
        assert!(version.project.favorite);
        assert!(key("../escape").is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn restored_archive_keeps_edition_label_and_independent_identity() {
        let root=std::env::temp_dir().join(format!("archive-restore-{}",Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let source=root.join("source");fs::create_dir_all(&source).unwrap();
        let file=source.join("project.json");fs::write(&file,b"{\"rows\":[]}").unwrap();
        let project=Project{metadata:crate::models::ProjectMetadata{title:"Vanilla edition".into(),..Default::default()},build_count:0,id:Uuid::new_v4().to_string(),name:"Vanilla edition".into(),description:String::new(),cover_image:None,source_url:Some("https://example.com".into()),project_json_url:None,file_path:file.to_string_lossy().into_owned(),viewer_preference:None,favorite:false,exclude_from_perk_index:false,date_added:String::new(),tags:vec![],file_missing:false};
        let version=snapshot_at(&root.join("versions"),&project,"test").unwrap();
        fs::write(&file,b"new active contents").unwrap();
        let restored=restored_copy(&version,&root.join("restored")).unwrap();
        assert_ne!(restored.id,project.id);
        assert_eq!(restored.name,"Vanilla edition");assert_eq!(restored.metadata.title,"Vanilla edition");
        assert_eq!(fs::read(&file).unwrap(),b"new active contents");
        assert_eq!(fs::read(&restored.file_path).unwrap(),b"{\"rows\":[]}");
        assert!(ensure_current_edition(&restored).is_err());assert!(ensure_current_edition(&project).is_ok());
        assert!(Path::new(&version.project.file_path).parent().unwrap().parent().unwrap().join("files.zip").exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn moving_preserves_archive_contents_label_star_and_restoration_receipt() {
        let root=std::env::temp_dir().join(format!("archive-move-{}",Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
        let file=root.join("export.json");fs::write(&file,b"{\"rows\":[]}").unwrap();
        let project=Project{metadata:Default::default(),build_count:0,id:Uuid::new_v4().to_string(),name:"Old label".into(),description:String::new(),cover_image:None,source_url:None,project_json_url:None,file_path:file.to_string_lossy().into_owned(),viewer_preference:None,favorite:false,exclude_from_perk_index:false,date_added:String::new(),tags:vec![],file_missing:false};
        let mut version=snapshot_at(&root.join("source"),&project,"test").unwrap();version.starred=true;
        let source=root.join("source").join(&version.id);write_json(&source.join("version.json"),&version).unwrap();write_json(&source.join("restored-library.json"),&"copy-id").unwrap();
        let original=fs::read(source.join("files.zip")).unwrap();let destination=root.join("target").join(&version.id);let target_id=Uuid::new_v4().to_string();
        let moved=relocate_version_at(&version,&source,&destination,&target_id).unwrap();
        assert!(!source.exists());assert_eq!(moved.project.id,target_id);assert_eq!(moved.project.name,"Old label");assert!(moved.starred);
        assert_eq!(original,fs::read(destination.join("files.zip")).unwrap());assert!(destination.join("restored-library.json").exists());
        crate::archive_storage::with_files(Path::new(&moved.project.file_path),||Ok(())).unwrap();assert_eq!(fs::read(moved.project.file_path).unwrap(),b"{\"rows\":[]}");
        assert!(archive_name("").is_err());assert!(archive_name("bad\nname").is_err());
        fs::remove_dir_all(root).unwrap();
    }

}
