#[cfg(test)]
mod font_preference_tests {
    use super::*;
    #[test]
    fn global_and_project_modes_preserve_custom_family() {
        let mut p=Preferences::default();p.cyoa_font="JetBrainsMono NF".into();
        assert_eq!(reader_font(&p,""),p.cyoa_font);
        assert_eq!(reader_font(&p,"author"),"");
        p.use_author_fonts=true;
        assert_eq!(reader_font(&p,""),"");
        assert_eq!(reader_font(&p,"custom"),"JetBrainsMono NF");
        assert_eq!(reader_font(&p,"author"),"");
        let roundtrip:Preferences=serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        assert!(roundtrip.use_author_fonts);assert_eq!(roundtrip.cyoa_font,"JetBrainsMono NF");
        let legacy:Preferences=serde_json::from_str(r#"{"cyoaFont":"serif"}"#).unwrap();
        assert_eq!(reader_font(&legacy,""),"serif");
    }
    #[test]
    fn project_font_setting_roundtrips_without_changing_old_metadata() {
        let old:crate::models::ProjectMetadata=serde_json::from_str("{}").unwrap();
        assert_eq!(old.font_preference,"");
        let selected:crate::models::ProjectMetadata=serde_json::from_str(r#"{"font_preference":"author"}"#).unwrap();
        let saved=serde_json::to_value(selected).unwrap();assert_eq!(saved["font_preference"],"author");
    }
}
