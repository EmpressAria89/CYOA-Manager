use serde::{Serialize};
use serde_json::Value;
use std::{collections::BTreeMap,path::Path};
#[derive(Default,Clone,Serialize)]
pub struct UpdateDiff { pub changed: bool, pub added: Vec<String>, pub removed: Vec<String>, pub edited: Vec<String>, pub other_changes: Vec<String> }
fn choices(json:&Value)->BTreeMap<String,Value>{
    let mut result=BTreeMap::new();
    if let Some(rows)=json["rows"].as_array(){for row in rows {if let Some(choices)=row["objects"].as_array().or(row["perks"].as_array()){for choice in choices{if let Some(id)=choice["id"].as_str().or(choice["uid"].as_str()){result.insert(id.into(),choice.clone());}}}}}
    result
}
fn label(id:&str,value:&Value)->String{let title=value["title"].as_str().or(value["name"].as_str()).unwrap_or(id);format!("{} [{}]",title,id)}
pub fn compare(before:&Value,after:&Value)->UpdateDiff{
    let old=choices(before);let new=choices(after);
    let mut report=UpdateDiff{changed:before!=after,..Default::default()};
    for(id,choice)in &new{match old.get(id){None=>report.added.push(label(id,choice)),Some(previous)if previous!=choice=>{
        let fields=choice.as_object().map(|map|map.keys().filter(|key|previous.get(*key)!=choice.get(*key)).cloned().collect::<Vec<_>>()).unwrap_or_default();
        let removed_fields=previous.as_object().map(|map|map.keys().filter(|key|choice.get(*key).is_none()).cloned().collect::<Vec<_>>()).unwrap_or_default();
        report.edited.push(format!("{}: {}",label(id,choice),fields.into_iter().chain(removed_fields).collect::<Vec<_>>().join(", ")));
    },_=>{}}}
    for(id,choice)in &old{if !new.contains_key(id){report.removed.push(label(id,choice));}}
    if let(Some(a),Some(b))=(before.as_object(),after.as_object()){
        for key in a.keys().chain(b.keys()).collect::<std::collections::BTreeSet<_>>(){if key!="rows"&&a.get(key)!=b.get(key){report.other_changes.push(key.clone());}}
    }
    if before["rows"]!=after["rows"] && report.added.is_empty()&&report.removed.is_empty()&&report.edited.is_empty(){report.other_changes.push("Row layout, text or requirements".into());}
    report
}
pub fn compare_files(before:&Path,after:&Path)->Result<UpdateDiff,String>{
    let a=serde_json::from_slice(&std::fs::read(before).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let b=serde_json::from_slice(&std::fs::read(after).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let mut diff=compare(&a,&b);
    let old=crate::assets::manifest(before)?;let new=crate::assets::manifest(after)?;
    for name in old.keys().chain(new.keys()).collect::<std::collections::BTreeSet<_>>() {if old.get(name)!=new.get(name){diff.changed=true;diff.other_changes.push(format!("Local asset: {name}"));}}
    Ok(diff)
}
#[cfg(test)]
include!("../../tests/rust/update_diff.rs");
