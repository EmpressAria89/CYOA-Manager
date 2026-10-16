#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_direct_json_but_refuses_conflicting_credits() {
        assert_eq!(
            source_key("https://gaston2.neocities.org/Warcraft/project.json"),
            source_key("https://gaston2.neocities.org/Warcraft/")
        );
        let p = serde_json::json!({"values":[["Interactive","Author","Universe"],["https://example.com/a/","Alice","World"],["https://example.com/a/project.json","Bob","World"]]});
        assert!(catalog(&p).values().next().unwrap().is_none());
    }
    #[test]
    fn catalog_credits_keep_query_and_route_projects_separate() {
        let payload = serde_json::json!({"values":[
            ["Interactive", "Author", "Universe"],
            ["https://example.com/play/?project=one#/chapter", "Alice", "First"],
            ["https://example.com/play/?project=two#/chapter", "Bob", "Second"],
            ["https://example.com/play/?project=one#/other", "Carol", "Third"]
        ]});
        let credits = catalog(&payload);
        assert_eq!(credits.len(), 3);
        for (url, author) in [
            ("https://example.com/play/?project=one#/chapter", "Alice"),
            ("https://example.com/play/?project=two#/chapter", "Bob"),
            ("https://example.com/play/?project=one#/other", "Carol"),
        ] {
            let credit = credits
                .get(&source_key(url).unwrap())
                .unwrap()
                .as_ref()
                .unwrap();
            assert_eq!(credit.author, author);
        }
        assert!(!credits.contains_key(&source_key("https://example.com/play/").unwrap()));
        assert_eq!(
            source_key("https://example.com/play/project.json?project=one#/chapter"),
            source_key("https://example.com/play/?project=one#/chapter")
        );
    }
    #[test]
    fn fills_only_missing_fields_and_preserves_tags() {
        let mut p:Project=serde_json::from_value(serde_json::json!({"id":"test","name":"Title","description":"My description","cover_image":null,"file_path":"a","viewer_preference":null,"date_added":"","tags":["Gaston"],"fandom":"My fandom"})).unwrap();
        assert!(fill(
            &mut p,
            &Credit {
                author: "Gaston1231".into(),
                fandom: "World of Warcraft".into(),
                description: "Catalog".into()
            }
        ));
        assert_eq!(p.metadata.author, "Gaston1231");
        assert_eq!(p.metadata.fandom, "My fandom");
        assert_eq!(p.description, "My description");
        assert_eq!(p.tags, ["Gaston"]);
    }
}

#[cfg(test)]
mod public_rows_tests {
    use super::*;
    #[test]
    fn public_sheet_preserves_headers_blanks_and_display_dates() {
        let source = r#"/*O_o*/
google.visualization.Query.setResponse({"status":"ok","table":{"cols":[{"label":"Title"},{"label":"Interactive"},{"label":"Updated"},{"label":"Description"}],"rows":[{"c":[{"v":"A Story"},{"v":"https://example.com/"},{"v":"Date(2026,9,16)","f":"2026-10-16"},null]},{"c":[{"v":"Other"},{"v":"https://example.com/other/"}]}]}});"#;
        let rows = parse_rows(source).unwrap();
        assert_eq!(rows[0], ["Title", "Interactive", "Updated", "Description"]);
        assert_eq!(
            rows[1],
            ["A Story", "https://example.com/", "2026-10-16", ""]
        );
        assert_eq!(rows[2].len(), 4);
        assert!(parse_rows("not a catalog").is_err());
        assert!(parse_rows("setResponse({\"status\":\"error\"});").is_err());
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;
    fn temp() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("catalog-snapshot-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }
    fn fixture(spec: &CatalogSourceSpec, value: &str) -> Vec<Vec<String>> {
        let header = if spec.id == "projects" {
            vec!["Name".into(), "Link".into()]
        } else {
            vec!["Title".into(), "Interactive".into()]
        };
        vec![
            header,
            vec![
                value.into(),
                "https://example.com/route?edition=1#play".into(),
            ],
        ]
    }
    #[test]
    fn valid_snapshots_survive_restart_without_network_and_force_refreshes() {
        let path = temp();
        let now = chrono::Utc::now();
        let mut cache = SnapshotCache::default();
        let mut calls = 0;
        let first = load_snapshot(&mut cache, &path, false, now, |spec| {
            calls += 1;
            Ok(fixture(spec, "Original"))
        });
        assert_eq!(calls, 4);
        assert!(!first.stale);
        assert_eq!(first.sources.len(), 4);
        let mut restarted = SnapshotCache::default();
        let offline = load_snapshot(
            &mut restarted,
            &path,
            false,
            now + chrono::Duration::hours(1),
            |_| panic!("Disk snapshot must be usable offline without a request"),
        );
        assert!(offline.stale);
        assert_eq!(offline.sources[0].rows[1][0], "Original");
        let refreshed = load_snapshot(
            &mut restarted,
            &path,
            true,
            now + chrono::Duration::hours(1),
            |spec| Ok(fixture(spec, "New")),
        );
        assert!(!refreshed.stale);
        assert_eq!(refreshed.sources[0].rows[1][0], "New");
        assert_eq!(
            refreshed.sources[0].rows[1][1],
            "https://example.com/route?edition=1#play"
        );
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn expired_memory_snapshot_remains_immediately_searchable_without_network() {
        let path = temp();
        let now = chrono::Utc::now();
        let mut cache = SnapshotCache::default();
        load_snapshot(&mut cache, &path, false, now, |spec| Ok(fixture(spec, "Cached")));
        let result = load_snapshot(&mut cache, &path, false, now + chrono::Duration::hours(1), |_| panic!("Cached searches must not wait for refresh"));
        assert!(result.stale);
        assert_eq!(result.sources[0].rows[1][0], "Cached");
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn cached_read_does_not_wait_for_in_flight_refresh() {
        let path = temp();
        let mut cache = SnapshotCache::default();
        load_snapshot(&mut cache, &path, false, chrono::Utc::now(), |spec| Ok(fixture(spec, "Saved")));
        let store = std::sync::Arc::new(Mutex::new(Some(cache)));
        let held = store.lock().unwrap();
        let worker_store = store.clone(); let worker_path = path.clone();
        let (send, receive) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || { send.send(snapshot_from(&worker_store, &worker_path, false)).unwrap(); });
        let result = receive.recv_timeout(Duration::from_secs(2));
        drop(held); worker.join().unwrap();
        let snapshot = result.expect("Cached read waited on refresh").unwrap();
        assert_eq!(snapshot.sources[0].rows[1][0], "Saved");
        assert!(snapshot.warnings.iter().any(|warning| warning.contains("Refresh in progress")));
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn partial_failure_keeps_last_good_rows_and_original_timestamp() {
        let path = temp();
        let now = chrono::Utc::now();
        let mut cache = SnapshotCache::default();
        let before = load_snapshot(&mut cache, &path, false, now, |spec| {
            Ok(fixture(spec, "Before"))
        });
        let after = load_snapshot(
            &mut cache,
            &path,
            true,
            now + chrono::Duration::minutes(1),
            |spec| {
                if spec.id == "beta" {
                    Err("Offline".into())
                } else {
                    Ok(fixture(spec, "After"))
                }
            },
        );
        assert!(after.stale);
        assert!(!after.warnings.is_empty());
        assert_eq!(after.sources[0].rows, before.sources[0].rows);
        assert_eq!(after.sources[0].fetched_at, before.sources[0].fetched_at);
        assert_eq!(after.sources[0].error.as_deref(), Some("Offline"));
        assert!(!after.sources[1].stale);
        assert_eq!(after.sources[1].rows[1][0], "After");
        let disk = read_source_cache(&path, SOURCES[0]).unwrap().unwrap();
        assert_eq!(disk.rows, before.sources[0].rows);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn invalid_refresh_and_wrong_source_cache_cannot_replace_good_data() {
        let path = temp();
        let now = chrono::Utc::now();
        let mut cache = SnapshotCache::default();
        let before = load_snapshot(&mut cache, &path, false, now, |spec| {
            Ok(fixture(spec, "Good"))
        });
        let after = load_snapshot(
            &mut cache,
            &path,
            true,
            now + chrono::Duration::minutes(1),
            |_| Ok(vec![vec!["unexpected".into()]]),
        );
        assert!(after.stale);
        assert_eq!(after.sources[0].rows, before.sources[0].rows);
        let beta = std::fs::read(path.join("beta.json")).unwrap();
        std::fs::write(path.join("projects.json"), beta).unwrap();
        assert!(read_source_cache(&path, SOURCES[1]).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn failed_fetch_is_throttled_and_serialization_exposes_freshness() {
        let path = temp();
        let now = chrono::Utc::now();
        let mut cache = SnapshotCache::default();
        let failed = load_snapshot(&mut cache, &path, false, now, |_| Err("Unavailable".into()));
        assert!(failed.stale);
        assert!(failed
            .sources
            .iter()
            .all(|s| s.rows.is_empty() && s.error.is_some()));
        let again = load_snapshot(
            &mut cache,
            &path,
            false,
            now + chrono::Duration::seconds(1),
            |_| panic!("Repeated catalog opens must share the previous attempt"),
        );
        assert_eq!(again.sources.len(), 4);
        let json = serde_json::to_value(again).unwrap();
        assert!(json.get("fetchedAt").is_some());
        assert!(json["sources"][0].get("fetchedAt").is_some());
        std::fs::remove_dir_all(path).unwrap();
    }
}
