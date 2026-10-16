#[cfg(test)]
mod reader_asset_tests {
    use super::*;
    #[test]
    fn lossless_image_resources_preserve_other_json_and_original() {
        let label=format!("viewer-test-{}",uuid::Uuid::new_v4());
        let source=std::env::temp_dir().join(format!("{label}.json"));
        let original=br#"{"title":"Escaped \\\" title","image":"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a5xkAAAAASUVORK5CYII=","addon":{"image":"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a5xkAAAAASUVORK5CYII="},"url":"data:image/unknown;base64,aGVsbG8=","id":"choice-1"}"#;
        fs::write(&source,original).unwrap();
        let prepared=project(&label,source.to_str().unwrap(),||true).unwrap();
        let value:serde_json::Value=serde_json::from_slice(&prepared).unwrap();
        assert_eq!(value["image"],value["addon"]["image"]);
        let resource=value["image"].as_str().unwrap().strip_prefix("/__manager_images/").unwrap().split('?').next().unwrap();
        assert_eq!(image(&label,resource).unwrap().0,STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a5xkAAAAASUVORK5CYII=").unwrap());
        assert_eq!(value["id"],"choice-1");
        assert_eq!(value["url"],"data:image/unknown;base64,aGVsbG8=");
        assert_eq!(fs::read(&source).unwrap(),original);
        assert_eq!(fs::read_dir(folder(&label).unwrap().join("images")).unwrap().count(),1);
        assert!(image(&label,"../../project.json").is_none());
        assert!(project(&label,source.to_str().unwrap(),||false).is_err());
        retire(&label); assert!(!folder(&label).unwrap().exists()); fs::remove_file(source).unwrap();
    }
    #[test]
    fn escaped_slashes_and_chunk_boundaries_are_preserved() {
        let label=format!("viewer-test-{}",uuid::Uuid::new_v4());
        let cache=folder(&label).unwrap();fs::create_dir_all(cache.join("images")).unwrap();
        let input=br#"{"image":"data:image\/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a5xkAAAAASUVORK5CYII=","text":"a\\b\"c","count":3}"#;
        let mut output=Vec::new();
        externalize(BufReader::with_capacity(3,&input[..]),&mut output,&cache.join("images")).unwrap();
        let result:serde_json::Value=serde_json::from_slice(&output).unwrap();
        assert!(result["image"].as_str().unwrap().starts_with("/__manager_images/"));
        assert_eq!(result["text"],"a\\b\"c");
        assert!(folder("viewer-../private").is_err());retire(&label);
    }
    #[test]
    #[ignore = "Optional read-only check against a large local export"]
    fn large_export_preparation() {
        use std::io::Read;
        let source=std::env::var("CYOA_READER_TEST_PROJECT").expect("Set CYOA_READER_TEST_PROJECT to a local export");
        fn digest(path:&str)->Vec<u8>{let mut file=File::open(path).unwrap();let mut hash=Sha256::new();let mut chunk=[0u8;65536];loop{let n=file.read(&mut chunk).unwrap();if n==0{break;}hash.update(&chunk[..n]);}hash.finalize().to_vec()}
        let before=digest(&source);let bytes=fs::metadata(&source).unwrap().len();
        let label=format!("viewer-test-{}",uuid::Uuid::new_v4());
        let start=std::time::Instant::now();
        let prepared=project(&label,&source,||true).unwrap();
        let value:serde_json::Value=serde_json::from_slice(&prepared).unwrap();
        let count=fs::read_dir(folder(&label).unwrap().join("images")).unwrap().count();
        println!("Reader preparation: original {} bytes, browser JSON {} bytes, {} resources, {:?}, {} rows",bytes,prepared.len(),count,start.elapsed(),value["rows"].as_array().map_or(0,|v|v.len()));
        assert_eq!(before,digest(&source));
        assert!(count>0);assert!((prepared.len() as u64)<bytes);
        retire(&label);
    }
}
