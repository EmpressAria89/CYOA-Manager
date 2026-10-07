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
}
