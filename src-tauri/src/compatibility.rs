//! Read-only dialect inspection. Scripts are counted, never executed or rewritten.
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Serialize;
use std::{fmt, io::Read};

#[derive(Default, Debug, Serialize)]
pub struct Report {
    pub editor: String,
    pub scripts: usize,
    pub variable_operations: usize,
}
impl Report {
    pub fn needs_native_viewer(&self) -> bool {
        self.editor.eq_ignore_ascii_case("iccremix") || self.scripts > 0 || self.variable_operations > 0
    }
}
struct Scan<'a> { report: &'a mut Report, root: bool }
impl<'de> DeserializeSeed<'de> for Scan<'_> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Scan<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("project JSON") }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "editor_id" if self.root => {
                    let value = map.next_value::<serde_json::Value>()?;
                    self.report.editor = value.as_str().unwrap_or_default().to_string();
                }
                "script" => {
                    let value = map.next_value::<serde_json::Value>()?;
                    if value.as_str().is_some_and(|s| !s.trim().is_empty()) { self.report.scripts += 1; }
                }
                "variableModifications" | "batchVariableModifications" => {
                    // Empty default arrays are harmless; preserve only the operation count.
                    map.next_value_seed(Count { count: &mut self.report.variable_operations })?;
                }
                "rows" | "objects" | "addons" | "scores" | "requireds" | "orRequireds" |
                "requirementScript" | "requirementScripts" | "globalRequirements" => {
                    map.next_value_seed(Scan { report: self.report, root: false })?;
                }
                _ => { map.next_value::<IgnoredAny>()?; }
            }
        }
        Ok(())
    }
    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<(), S::Error> {
        while seq.next_element_seed(Scan { report: self.report, root: false })?.is_some() {}
        Ok(())
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> { Ok(()) }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<(), E> { Ok(()) }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<(), E> { Ok(()) }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<(), E> { Ok(()) }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> { Ok(()) }
    fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<(), E> { Ok(()) }
}
struct Count<'a> { count: &'a mut usize }
impl<'de> DeserializeSeed<'de> for Count<'_> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Count<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("variable list") }
    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<(), S::Error> {
        while seq.next_element::<IgnoredAny>()?.is_some() { *self.count += 1; }
        Ok(())
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        while map.next_key::<IgnoredAny>()?.is_some() { map.next_value::<IgnoredAny>()?; *self.count += 1; }
        Ok(())
    }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<(), E> { if value { *self.count += 1; } Ok(()) }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<(), E> { if !value.trim().is_empty() { *self.count += 1; } Ok(()) }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<(), E> { *self.count += 1; Ok(()) }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<(), E> { *self.count += 1; Ok(()) }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<(), E> { *self.count += 1; Ok(()) }
    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> { Ok(()) }
}
pub fn inspect(input: impl Read) -> Result<Report, String> {
    let mut report = Report::default();
    let mut parser = serde_json::Deserializer::from_reader(input);
    Scan { report: &mut report, root: true }.deserialize(&mut parser).map_err(|e|e.to_string())?;
    parser.end().map_err(|e|e.to_string())?;
    Ok(report)
}
pub fn inspect_value(value: &serde_json::Value) -> Report {
    let mut report = Report::default();
    // The value already exists during import; do not serialize embedded images again.
    let _ = Scan { report: &mut report, root: true }.deserialize(value);
    report
}
#[cfg(test)]
include!("../../tests/rust/compatibility.rs");
