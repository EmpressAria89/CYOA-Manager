#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_builds_preserve_identity_payload_and_version() {
        let root = std::env::temp_dir().join(format!("build-roundtrip-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let project = root.join("project.json"); fs::write(&project, b"old version").unwrap();
        let mut session = ViewerSession { project_id: Uuid::new_v4().to_string(), project_name: "Naruto".into(), viewer_id: "icc-original".into(), cheats_enabled: false, file_path: project.to_string_lossy().into_owned(), fingerprint: fingerprint(&project).unwrap(), theme: "mocha".into(), cyoa_font: "system-ui".into() };
        let store = root.join("builds");
        let saved = handle_at(&store, &session, "POST", br#"{"name":"Hard mode","content":"choice-a,choice-b/ON#2"}"#).unwrap();
        assert_eq!(saved["project_id"], session.project_id);
        assert_eq!(saved["content"], "choice-a,choice-b/ON#2");
        let listed = handle_at(&store, &session, "GET", b"").unwrap();
        assert_eq!(listed["builds"][0], saved);
        assert_eq!(listed["fingerprint"], saved["fingerprint"]);
        fs::write(&project, b"changed version").unwrap();
        session.fingerprint = fingerprint(&project).unwrap();
        let changed = handle_at(&store, &session, "GET", b"").unwrap();
        assert_ne!(changed["fingerprint"], changed["builds"][0]["fingerprint"]);
        let mut wrong_session = session.clone(); wrong_session.project_id = Uuid::new_v4().to_string();
        assert!(handle_at(&store, &wrong_session, "GET", b"").is_err());
        assert!(handle_at(&store, &session, "DELETE", b"").is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn fingerprints_detect_version_changes_and_names_are_validated() {
        let path = std::env::temp_dir().join(format!("build-test-{}", Uuid::new_v4()));
        fs::write(&path, b"old version").unwrap(); let old = fingerprint(&path).unwrap();
        fs::write(&path, b"new version").unwrap(); assert_ne!(old, fingerprint(&path).unwrap());
        assert!(validate(&Draft { name: " ".into(), content: "choices".into(), source: None, legacy: false }).is_err());
        assert!(validate(&Draft { name: "My build".into(), content: " ".into(), source: None, legacy: false }).is_err());
        fs::remove_file(path).unwrap();
    }
}
