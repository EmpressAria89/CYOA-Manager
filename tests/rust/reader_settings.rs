#[cfg(test)]
mod reader_tests {
 use super::*;
 fn change(value:serde_json::Value)->Change{serde_json::from_value(value).unwrap()}
 #[test]
 fn section_overrides_are_pinned_until_explicit_reset(){
  let mut store=Store::default();let mut prefs=crate::preferences::Preferences::default();prefs.cyoa_font="serif".into();
  apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"project","section":"title","font":"monospace","size":125}))).unwrap();
  apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"global","section":"title","font":"sans-serif","size":175}))).unwrap();
  assert_eq!(effective(&store,"a",&prefs,"").parts["title"].font,"monospace");assert_eq!(effective(&store,"a",&prefs,"").parts["title"].size,125);
  assert_eq!(effective(&store,"b",&prefs,"").parts["title"].size,175);
  let saved=serde_json::to_vec(&store).unwrap();store=serde_json::from_slice(&saved).unwrap();
  apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"project","section":"title","reset":true}))).unwrap();assert_eq!(effective(&store,"a",&prefs,"").parts["title"].font,"sans-serif");
 }
 #[test]
 fn fonts_and_layout_are_independent_and_author_mode_is_preserved(){
  let mut store=Store::default();let mut prefs=crate::preferences::Preferences::default();prefs.cyoa_font="serif".into();
  apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"global","section":"body","font":"","size":120}))).unwrap();assert_eq!(prefs.cyoa_font,"serif");assert!(prefs.use_author_fonts);
  apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"project","section":"layout","scale":80,"colorScores":true}))).unwrap();assert_eq!(effective(&store,"a",&prefs,"author").scale,Some(80));assert_eq!(effective(&store,"a",&prefs,"author").parts["body"].font,"");
  assert!(apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"project","section":"layout","scale":2}))).is_err());
 }
 #[test]
 fn section_hiding_is_project_specific_and_survives_unrelated_edits(){
  let mut store=Store::default();let mut prefs=crate::preferences::Preferences::default();
  apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"project","section":"sections","autoHideSections":true,"hiddenRows":["dragon"]}))).unwrap();
  apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"project","section":"title","font":"serif","size":120}))).unwrap();
  store=serde_json::from_slice(&serde_json::to_vec(&store).unwrap()).unwrap();
  assert_eq!(effective(&store,"a",&prefs,"").hidden_rows,Some(vec!["dragon".into()]));
  assert_eq!(effective(&store,"b",&prefs,"").hidden_rows,None);
  apply(&mut store,"a",&mut prefs,change(serde_json::json!({"scope":"project","section":"sections","reset":true}))).unwrap();
  assert_eq!(effective(&store,"a",&prefs,"").hidden_rows,None);
  assert_eq!(effective(&store,"a",&prefs,"").parts["title"].size,120);
 }

}
