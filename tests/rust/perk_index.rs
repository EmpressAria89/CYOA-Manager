#[cfg(test)]
mod tests {
    use super::{extract_perks, strip_utf8_bom};
    use serde_json::json;

    #[test]
    fn strip_utf8_bom_removes_prefix_only_when_present() {
        let payload = b"\xEF\xBB\xBF{\"rows\":[]}";
        assert_eq!(strip_utf8_bom(payload), b"{\"rows\":[]}");
        assert_eq!(strip_utf8_bom(b"{\"rows\":[]}"), b"{\"rows\":[]}");
    }

    #[test]
    fn extract_perks_uses_object_id_as_fallback_title() {
        let document = json!({
            "rows": [
                {
                    "id": "row-1",
                    "title": "Row 1",
                    "objects": [
                        { "id": "empty" },
                        { "id": "perk-1", "title": "Perk 1", "text": "Has text" }
                    ]
                }
            ]
        });

        let perks = extract_perks(&document);
        assert_eq!(perks.len(), 2);
        assert_eq!(perks[0].title, "empty");
        assert_eq!(perks[1].object_id, "perk-1");
    }
}