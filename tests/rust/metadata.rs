#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn normalizes_brackets_and_separates_mod_credits(){assert_eq!(split_credits("Gaston- jojonosa(mod)"),("Gaston".into(),"jojonosa".into()));assert_eq!(split_credits("Mary-Jane"),("Mary-Jane".into(),String::new()));let tags=normalize_tags(&["[OC] [FemPOV] [Event Based]".into(),"world picker, World Picker".into()]);assert_eq!(tags,["Original Character","PoV: Female","Event Based","World Picker"]);}
    #[test] fn infer_separates_metadata_and_does_not_invent_completion() {
        let j=serde_json::json!({"pointTypes":[],"styling":{},"rows":[{"objects":[{"id":"a"}],"text":"<p>About this CYOA</p>"}]});
        let m=infer(&j,"Gaston - jojonosa(mod): Naruto");
        assert_eq!(m.title,"Naruto"); assert_eq!(m.fandom,"Naruto"); assert!(m.is_mod);
        assert!(m.completion.is_empty()); assert!(icc2_compatible(&j));
        assert_eq!(description(&j),"About this CYOA");
    }
    #[test] fn refresh_fills_fandom_without_replacing_manual_metadata_or_archive_identity(){
        let mut p=crate::models::Project{metadata:crate::models::ProjectMetadata{title:"Gaston's Dragon Age CYOA".into(),author:"My credit".into(),restored_from_archive:true,..Default::default()},build_count:0,id:"id".into(),name:"Dragon Age".into(),description:"My notes".into(),cover_image:None,source_url:None,project_json_url:None,file_path:"file".into(),viewer_preference:None,favorite:false,exclude_from_perk_index:false,date_added:"today".into(),tags:vec![],file_missing:false};
        let j=serde_json::json!({"author":"Source author","description":"Source description","rows":[{"objects":[{"id":"a"}]}]});
        fill_missing(&mut p,&j);assert_eq!(p.metadata.fandom,"Dragon Age");assert_eq!(p.metadata.author,"My credit");assert_eq!(p.description,"My notes");assert!(p.metadata.restored_from_archive);
        p.metadata.fandom="Custom universe".into();fill_missing(&mut p,&serde_json::json!({"metadata":{"fandom":"Updated source universe"}}));assert_eq!(p.metadata.fandom,"Custom universe");
        p.metadata.fandom.clear();fill_missing(&mut p,&serde_json::json!({"metadata":{"fandom":"Updated source universe"}}));assert_eq!(p.metadata.fandom,"Updated source universe");
        p.name="Writer: Unknown".into();p.metadata.title="Custom display title".into();p.metadata.author.clear();fill_missing(&mut p,&serde_json::json!({}));assert_eq!(p.metadata.author,"Writer");
    }
    #[test] fn forced_updates_cannot_replace_a_game_with_an_empty_source_stub(){assert!(validate_update(&serde_json::json!({"rows":[]})).is_err());assert!(validate_update(&serde_json::json!({"rows":[{"objects":[]}]})).is_err());assert!(validate_update(&serde_json::json!({"rows":[{"objects":[{"id":"a"}]}]})).is_ok());}
}
