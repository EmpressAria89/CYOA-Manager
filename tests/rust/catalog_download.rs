fn fixture(root: &Path, name: &str, id: &str) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, serde_json::json!({"pointTypes":[],"styling":{},"rows":[{"objects":[{"id":id,"title":"Choice","image":"ignored artwork"}]}]}).to_string()).unwrap();
    path
}
fn card(path: &Path) -> Project {
    Project {
        metadata: Default::default(),
        build_count: 0,
        id: uuid::Uuid::new_v4().to_string(),
        name: "Original name".into(),
        description: String::new(),
        cover_image: None,
        source_url: Some("https://example.org/game/?route=1#section".into()),
        project_json_url: None,
        file_path: path.to_string_lossy().into_owned(),
        viewer_preference: None,
        favorite: false,
        exclude_from_perk_index: false,
        date_added: String::new(),
        tags: vec![],
        file_missing: false,
    }
}
#[test]
fn inspect_real_response_content_and_preserve_routes_without_running_scripts() {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = [0; 2048];
        let _ = stream.read(&mut request);
        let body = r#"{"editor_id":"iccremix","rows":[{"objects":[{"id":"a"}],"requirementScript":{"script":"throw new Error('NEVER RUN')"}}]}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
    });
    let source = format!("http://{address}/play?route=one#tab");
    let resolved = resolve(&source, Some("https://example.org/old.zip".into())).unwrap();
    assert_eq!(resolved.plan.mode, "website");
    assert_eq!(resolved.plan.source_url, source);
    assert_eq!(resolved.plan.compatibility.unwrap().scripts, 1);
    assert_eq!(
        resolved.plan.archive_url.as_deref(),
        Some("https://example.org/old.zip")
    );
    let root = resolved.staging.root.clone();
    drop(resolved.staging);
    assert!(!root.exists());
    server.join().unwrap();
}
#[test]
fn vanilla_json_is_supported_but_empty_or_disjoint_projects_are_rejected() {
    let stage = Staging::new().unwrap();
    let a = fixture(&stage.root, "a.json", "same");
    let b = fixture(&stage.root, "b.json", "same");
    let c = fixture(&stage.root, "c.json", "unrelated");
    assert!(!inspect_file(&a).unwrap().needs_native_viewer());
    assert!(ensure_related(&a, &b).is_ok());
    assert!(ensure_related(&a, &c).is_err());
    fs::write(&b, r#"{"rows":[]}"#).unwrap();
    assert!(inspect_file(&b).is_err());
    assert!(copy_bounded(&b"12345"[..], Vec::new(), 4).is_err());
    assert!(http_url("file:///tmp/private").is_err());
    assert!(http_url("https://user:secret@example.org/").is_err());
}
#[test]
fn update_preview_rejects_changed_card_and_edition_and_discards_assets() {
    let staging = Staging::new().unwrap();
    let path = fixture(&staging.root, "candidate.json", "same");
    let old_stage = Staging::new().unwrap();
    let old = fixture(&old_stage.root, "old.json", "same");
    let previous = card(&old);
    let pending_item = Pending {
        created: Instant::now(),
        before: edition_digest(&old).unwrap(),
        candidate: edition_digest(&path).unwrap(),
        previous: previous.clone(),
        path: path.clone(),
        staging,
        source_url: previous.source_url.clone().unwrap(),
        project_url: "https://example.org/game/project.json".into(),
        diff: Default::default(),
    };
    assert!(validate_preview(&pending_item, &previous).is_ok());
    let mut renamed = previous.clone();
    renamed.name = "Renamed".into();
    assert!(validate_preview(&pending_item, &renamed).is_err());
    assert!(ensure_same_source(&previous, "https://example.org/game/?route=2#section").is_err());
    fs::write(&path, "tampered").unwrap();
    assert!(validate_preview(&pending_item, &previous).is_err());
    let root = pending_item.staging.root.clone();
    let token = uuid::Uuid::new_v4().to_string();
    pending()
        .lock()
        .unwrap()
        .insert(token.clone(), pending_item);
    discard_catalog_update(token).unwrap();
    assert!(!root.exists());
    let mut restored = previous.clone();
    restored.metadata.restored_from_archive = true;
    assert!(ensure_same_source(&restored, restored.source_url.as_deref().unwrap()).is_err());
}
#[test]
fn recovery_refuses_multiple_projects_and_remix_before_library_import() {
    let stage = Staging::new().unwrap();
    let zip_path = stage.root.join("test.zip");
    {
        let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());
        for name in ["one.json", "two.json"] {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(br#"{"pointTypes":[],"styling":{},"rows":[{"objects":[{"id":"a"}]}]}"#)
                .unwrap();
        }
        zip.finish().unwrap();
    }
    assert!(extract_recovery(&zip_path, &stage.root.join("prepared"))
        .unwrap_err()
        .contains("exactly one"));
}
#[test]
#[cfg(target_os = "linux")]
fn crash_cleanup_keeps_live_processes_unrelated_and_symlink_paths() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let stage = Staging::new().unwrap();
    assert_eq!(
        fs::metadata(&stage.root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let expired = stage
        .root
        .join(format!("cyoa-catalog-4294967295-{}", uuid::Uuid::new_v4()));
    let alive = stage.root.join(format!(
        "cyoa-catalog-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let linked = stage
        .root
        .join(format!("cyoa-catalog-4294967295-{}", uuid::Uuid::new_v4()));
    let unrelated = stage.root.join("user-files");
    fs::create_dir(&expired).unwrap();
    fs::create_dir(&alive).unwrap();
    fs::create_dir(&unrelated).unwrap();
    symlink(&unrelated, &linked).unwrap();
    cleanup_stale_in(&stage.root);
    assert!(!expired.exists());
    assert!(alive.exists());
    assert!(unrelated.exists());
    assert!(fs::symlink_metadata(&linked)
        .unwrap()
        .file_type()
        .is_symlink());
}
#[test]
fn candidate_asset_digest_changes_when_local_artwork_changes() {
    let stage = Staging::new().unwrap();
    let project = fixture(&stage.root, "project.json", "same");
    let artwork = stage.root.join("cover.png");
    fs::write(&artwork, b"first").unwrap();
    let before = edition_digest(&project).unwrap();
    fs::write(&artwork, b"changed").unwrap();
    assert_ne!(edition_digest(&project).unwrap(), before);
}
#[test]
fn routed_html_does_not_guess_or_follow_a_default_project_link() {
    use std::{net::TcpListener, sync::mpsc};
    for route in ["?variant=selected", "#selected-variant"] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let (stop, finished) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let mut requests = 0;
            loop {
                if finished.try_recv().is_ok() {
                    break;
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(5)))
                            .unwrap();
                        let mut input = [0; 2048];
                        let _ = stream.read(&mut input);
                        requests += 1;
                        let body = if requests == 1 {
                            r#"<html><script src="project.json"></script></html>"#
                        } else {
                            r#"{"pointTypes":[],"styling":{},"rows":[{"objects":[{"id":"wrong-default"}]}]}"#
                        };
                        let mime = if requests == 1 {
                            "text/html"
                        } else {
                            "application/json"
                        };
                        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(2))
                    }
                    Err(e) => panic!("Fixture server failed: {e}"),
                }
            }
            requests
        });
        let source = format!("http://{address}/game/{route}");
        let plan =
            tauri::async_runtime::block_on(inspect_catalog_download(source.clone(), None)).unwrap();
        stop.send(()).unwrap();
        let requests = server.join().unwrap();
        assert_eq!(plan.mode, "website");
        assert_eq!(plan.source_url, source);
        assert!(plan.project_url.is_none());
        assert!(plan.reason.contains("route"));
        assert_eq!(
            requests, 1,
            "Routed HTML must not fetch a guessed/default JSON"
        );
    }
}
#[test]
fn exact_routed_json_remains_downloadable_without_guessing() {
    use std::net::TcpListener;
    for path in ["/project.json", "/extensionless"] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 2048];
            let count = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..count]).contains("?edition=one"));
            let body = r#"{"pointTypes":[],"styling":{},"rows":[{"objects":[{"id":"selected-edition"}]}]}"#;
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        });
        let source = format!("http://{address}{path}?edition=one#selected");
        let plan =
            tauri::async_runtime::block_on(inspect_catalog_download(source.clone(), None)).unwrap();
        assert_eq!(plan.mode, "project");
        assert_eq!(plan.source_url, source);
        assert!(plan.project_url.unwrap().contains("?edition=one"));
        server.join().unwrap();
    }
}
