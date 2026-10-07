#[cfg(test)]mod tests{
 use super::*;
 #[test]fn detects_balance_changes_added_and_removed_choices(){let old=serde_json::json!({"rows":[{"objects":[{"id":"a","title":"A","scores":[1]},{"id":"b"}]}]});let new=serde_json::json!({"rows":[{"objects":[{"id":"a","title":"A","scores":[2]},{"id":"c"}]}]});let d=compare(&old,&new);assert!(d.changed);assert_eq!(d.added.len(),1);assert_eq!(d.removed.len(),1);assert!(d.edited[0].contains("scores"));assert!(!compare(&old,&old).changed);}
}
