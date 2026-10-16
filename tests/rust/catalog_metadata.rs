#[cfg(test)]mod tests{use super::*;
 #[test]fn matches_direct_json_but_refuses_conflicting_credits(){assert_eq!(source_key("https://gaston2.neocities.org/Warcraft/project.json"),source_key("https://gaston2.neocities.org/Warcraft/"));let p=serde_json::json!({"values":[["Interactive","Author","Universe"],["https://example.com/a/","Alice","World"],["https://example.com/a/project.json","Bob","World"]]});assert!(catalog(&p).values().next().unwrap().is_none());}
 #[test]fn fills_only_missing_fields_and_preserves_tags(){let mut p:Project=serde_json::from_value(serde_json::json!({"id":"test","name":"Title","description":"My description","cover_image":null,"file_path":"a","viewer_preference":null,"date_added":"","tags":["Gaston"],"fandom":"My fandom"})).unwrap();assert!(fill(&mut p,&Credit{author:"Gaston1231".into(),fandom:"World of Warcraft".into(),description:"Catalog".into()}));assert_eq!(p.metadata.author,"Gaston1231");assert_eq!(p.metadata.fandom,"My fandom");assert_eq!(p.description,"My description");assert_eq!(p.tags,["Gaston"]);}
}

#[cfg(test)]mod public_rows_tests {
 use super::*;
 #[test]fn public_sheet_preserves_headers_blanks_and_display_dates(){
  let source=r#"/*O_o*/
google.visualization.Query.setResponse({"status":"ok","table":{"cols":[{"label":"Title"},{"label":"Interactive"},{"label":"Updated"},{"label":"Description"}],"rows":[{"c":[{"v":"A Story"},{"v":"https://example.com/"},{"v":"Date(2026,9,16)","f":"2026-10-16"},null]},{"c":[{"v":"Other"},{"v":"https://example.com/other/"}]}]}});"#;
  let rows=parse_rows(source).unwrap();assert_eq!(rows[0],["Title","Interactive","Updated","Description"]);assert_eq!(rows[1],["A Story","https://example.com/","2026-10-16",""]);assert_eq!(rows[2].len(),4);
  assert!(parse_rows("not a catalog").is_err());assert!(parse_rows("setResponse({\"status\":\"error\"});").is_err());
 }
}
