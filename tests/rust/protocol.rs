#[cfg(test)]
mod reveal_tests {
    use super::*;
    #[test]
    fn reader_override_uses_regular_weight_for_authored_containers_and_headings() {
        let mut session = crate::models::ViewerSession {font_preference:String::new(),
            project_id: "fixture".into(), viewer_id: "icc2-plus".into(),
            cheats_enabled: false, file_path: String::new(), project_name: "Fixture".into(),
            fingerprint: String::new(), theme: "mocha".into(), cyoa_font: "serif".into(),
        };
        let authored = "<head></head><body><div style='font-weight:900'><h1>Title</h1><p>Body</p></div></body>";
        let html = themed_html(authored, &session);
        assert!(html.contains("body *:not(i):not(svg):not([class*=icon]):not(.mdi):not(.material-icons){font-family:'CYOA Reader Regular',sans-serif !important;font-weight:400 !important;font-synthesis:none !important;}"));
        assert!(!html.contains("font-weight:600"));
        session.cyoa_font.clear();
        let html = themed_html(authored, &session);
        assert!(!html.contains("font-weight:400"));
        assert!(html.contains("font-weight:900"));
    }
    #[test]
    fn reveal_changes_only_presentation_gates() {
        let original=include_str!("../../public/viewers/ICC2 Plus/js/app.B6d7tc9y.js");
        let patched=String::from_utf8(bridge_native_bundle("app.B6d7tc9y.js",original.as_bytes().to_vec())).unwrap();
        assert!(patched.contains("setRevealHidden=enabled=>"));
        assert!(patched.contains("(app.managerRevealHidden||v(O))&&e(ne)"));
        assert_eq!(patched.matches("app.managerRevealHidden||app.showAllAddons").count(),8);
        let checker=original.split("function Wo(").nth(1).unwrap().split("function ").next().unwrap();
        assert!(patched.contains(&format!("function Wo({checker}")));
        assert!(patched.contains("t.row.deselectChoices&&!v(p)"));
        for gate in ["ee(J,e=>{v(p)&&v(managerVisible)&&e(Z)})", "ee(X,e=>{v(p)&&v(managerVisible)&&e(K)})", ".concat(v(p)&&v(managerVisible)?\"\":\" hidden\")"] {
            assert!(patched.contains(gate), "navigation gate must remain intact: {gate}");
        }
        assert!(!patched.contains("app.managerRevealHidden||v(p)"));
        assert_eq!(patched.matches("window.__cyoaManagerImages.setSource(").count(),37);
        assert!(!patched.contains("app.preloadExternalImages&&"));
        assert!(patched.contains("style(o,v(m),\"score\",managerScoreKind())"));
        assert!(patched.contains("style(t,v(N),\"title\")"));
        assert!(patched.contains("style(t,v(G),\"description\")"));
        assert!(patched.contains("__cyoaManagerPresentation.card(i)"));
        if let Ok(path)=std::env::var("CYOA_NATIVE_BUNDLE"){std::fs::write(path,&patched).unwrap();}

        assert_eq!(patched.matches("get isEnabled(){return v(S)}").count(),original.matches("get isEnabled(){return v(S)}").count());
    }
    #[test]
    fn unsupported_bundle_does_not_offer_reveal() {
        let patched=bridge_native_bundle("app.B6d7tc9y.js",b"different engine".to_vec());
        assert!(!String::from_utf8(patched).unwrap().contains("setRevealHidden"));
    }
}
