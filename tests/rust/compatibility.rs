#[test]
fn inspect_dialects_without_evaluating_or_modifying_scripts() {
    let input = br#"{"editor_id":"iccremix","rows":[{"image":"ignored artwork","requirementScript":{"script":"[/tab/] == 4"},"objects":[{"requirementScripts":[{"script":""},{"script":"[/points/] > 2"}],"variableModifications":[{"variableId":"tab","value":4}]}]}]}"#;
    let report=inspect(&input[..]).unwrap();
    assert_eq!(report.editor,"iccremix"); assert_eq!(report.scripts,2);
    assert_eq!(report.variable_operations,1); assert!(report.needs_native_viewer());
    let value:serde_json::Value=serde_json::from_slice(input).unwrap();
    assert_eq!(inspect_value(&value).scripts,2);
    assert_eq!(value["rows"][0]["requirementScript"]["script"],"[/tab/] == 4");
}
#[test]
fn empty_defaults_and_text_mentions_do_not_trigger_warning() {
    let input=br#"{"rows":[{"title":"script","text":"requirementScript","requirementScript":{"script":"  "},"variables":[{"id":"vanilla-variable"}],"objects":[]}]}"#;
    assert!(!inspect(&input[..]).unwrap().needs_native_viewer());
    assert!(inspect(&br#"{"editor_id":"ICCREMIX"}"#[..]).unwrap().needs_native_viewer());
    assert!(inspect(&br#"{"rows":[{"requirementScripts":[{"script":"true"}]}]}"#[..]).unwrap().needs_native_viewer());
    assert!(inspect(&br#"{"rows":[]} trailing"#[..]).is_err());
}
