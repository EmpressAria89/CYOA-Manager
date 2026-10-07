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
}
