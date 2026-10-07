//! Personal display names are separate from original project metadata.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path, sync::Mutex};
static LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthorAlias {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct AliasFile {
    version: u32,
    authors: Vec<AuthorAlias>,
}
fn key(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
fn validate(authors: &mut [AuthorAlias]) -> Result<(), String> {
    if authors.len() > 1000 || authors.iter().map(|a| a.aliases.len()).sum::<usize>() > 5000 {
        return Err("Too many author aliases".into());
    }
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for author in authors {
        uuid::Uuid::parse_str(&author.id).map_err(|_| "Invalid author identity")?;
        if !ids.insert(author.id.clone()) {
            return Err("Duplicate author identity".into());
        }
        let mut unique = BTreeSet::new();
        for name in std::iter::once(&mut author.name).chain(author.aliases.iter_mut()) {
            *name = name.split_whitespace().collect::<Vec<_>>().join(" ");
            if name.is_empty() || name.len() > 200 || name.chars().any(char::is_control) {
                return Err("Author names must contain 1–200 characters".into());
            }
            let normalized = key(name);
            if unique.insert(normalized.clone()) && !names.insert(normalized) {
                return Err(format!("{name} already belongs to another author"));
            }
        }
        let canonical = key(&author.name);
        let mut seen = BTreeSet::new();
        author
            .aliases
            .retain(|alias| key(alias) != canonical && seen.insert(key(alias)));
    }
    Ok(())
}
fn write(path: &Path, authors: &[AuthorAlias]) -> Result<(), String> {
    let pending = path.with_extension(format!("{}.pending", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(path.parent().ok_or("Missing alias folder")?)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        let bytes = serde_json::to_vec_pretty(&AliasFile {
            version: 1,
            authors: authors.to_vec(),
        })
        .map_err(|e| e.to_string())?;
        std::fs::write(&pending, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&pending, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(pending);
    }
    result
}
fn read(path: &Path) -> Result<Vec<AuthorAlias>, String> {
    if !path.exists() {
        // The initial mapping was explicitly requested; future names are added by the user.
        let authors = vec![AuthorAlias {
            id: "ac9679c6-f8a3-4fa9-b077-5a2a92f3d522".into(),
            name: "Gaston".into(),
            aliases: vec!["Gaston1231".into()],
        }];
        write(path, &authors)?;
        return Ok(authors);
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut file: AliasFile = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Cannot read author aliases; original file preserved: {e}"))?;
    if file.version != 1 {
        return Err("Unsupported author alias file version".into());
    }
    validate(&mut file.authors)?;
    Ok(file.authors)
}
#[tauri::command(async)]
pub fn get_author_aliases() -> Result<Vec<AuthorAlias>, String> {
    let _lock = LOCK.lock().map_err(|e| e.to_string())?;
    read(&crate::library::data_root_dir().join("save/author-aliases.json"))
}
#[tauri::command(async)]
pub fn save_author_aliases(mut authors: Vec<AuthorAlias>) -> Result<Vec<AuthorAlias>, String> {
    let _lock = LOCK.lock().map_err(|e| e.to_string())?;
    validate(&mut authors)?;
    let root = crate::library::data_root_dir();
    let path = root.join("save/author-aliases.json");
    // Refuse to overwrite a corrupt or newer-format file, and keep the previous valid map.
    if path.exists() {
        let previous = read(&path)?;
        write(
            &root.join("backups/author-aliases.previous.json"),
            &previous,
        )?;
    }
    write(&path, &authors)?;
    Ok(authors)
}
#[cfg(test)]
include!("../../tests/rust/author_aliases.rs");
