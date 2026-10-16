//! Lossless, disposable reader resources. Never rewrite the library export.
use std::{fs::{self, File}, io::{BufRead, BufReader, BufWriter, Write}, path::PathBuf, sync::Mutex};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

// Serialize preparation against retirement, including windows closed during loading.
static PREPARATION: Mutex<()> = Mutex::new(());

fn folder(label: &str) -> Result<PathBuf, String> {
    if !label.starts_with("viewer-") || !label.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
        return Err("Invalid reader cache label".into());
    }
    Ok(std::env::temp_dir().join(format!("cyoa-manager-readers-{}", std::process::id())).join(label))
}

/// Scan JSON string tokens with bounded memory (one image, not the whole export).
fn externalize(input: impl BufRead, mut output: impl Write, images: &std::path::Path) -> Result<(), String> {
    let mut input = input;
    let mut quoted = false;
    let mut escaped = false;
    let mut token = Vec::new();
    loop {
        let buffer = input.fill_buf().map_err(|e| e.to_string())?;
        if buffer.is_empty() { break; }
        for &byte in buffer {
            if !quoted {
                if byte == b'"' { quoted = true; token.push(byte); }
                else { output.write_all(&[byte]).map_err(|e|e.to_string())?; }
            } else {
                token.push(byte);
                if byte == b'"' && !escaped {
                    let replacement = image_token(&token, images)?;
                    output.write_all(replacement.as_deref().unwrap_or(&token)).map_err(|e|e.to_string())?;
                    token.clear(); quoted = false;
                }
                escaped = if escaped { false } else { byte == b'\\' };
            }
        }
        let used = buffer.len(); input.consume(used);
    }
    if quoted { return Err("Unterminated project JSON string".into()); }
    output.flush().map_err(|e|e.to_string())
}

fn image_token(token: &[u8], images: &std::path::Path) -> Result<Option<Vec<u8>>, String> {
    if !token.starts_with(b"\"data:image") { return Ok(None); }
    let url: String = serde_json::from_slice(token).map_err(|e|e.to_string())?;
    let Some((header, encoded)) = url.split_once(',') else { return Ok(None); };
    let extension = match header {
        "data:image/png;base64" => "png", "data:image/jpeg;base64" | "data:image/jpg;base64" => "jpg",
        "data:image/webp;base64" => "webp", "data:image/avif;base64" => "avif",
        "data:image/gif;base64" => "gif", "data:image/svg+xml;base64" => "svg",
        _ => return Ok(None),
    };
    let Ok(bytes) = STANDARD.decode(encoded) else { return Ok(None); };
    // Reserve exact intrinsic space before the image is fetched. Without this,
    // zero-height placeholders can make the entire document intersect at once.
    let dimensions = image::ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format()
        .ok().and_then(|reader|reader.into_dimensions().ok());
    let Some((width,height))=dimensions.filter(|&(w,h)|w>0 && h>0) else {return Ok(None);};
    let name = format!("{:x}.{extension}", Sha256::digest(&bytes));
    let path = images.join(&name);
    if !path.exists() { fs::write(path, bytes).map_err(|e|e.to_string())?; }
    Ok(Some(serde_json::to_vec(&format!("/__manager_images/{name}?w={width}&h={height}")).unwrap()))
}

pub fn project(label: &str, source: &str, active: impl FnOnce() -> bool) -> Result<Vec<u8>, String> {
    let _lock = PREPARATION.lock().map_err(|e|e.to_string())?;
    if !active() { return Err("Reader closed while loading".into()); }
    let cache = folder(label)?;
    let prepared = cache.join("project.json");
    if !prepared.is_file() {
        fs::create_dir_all(cache.join("images")).map_err(|e|e.to_string())?;
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(cache.parent().unwrap(), fs::Permissions::from_mode(0o700)).map_err(|e|e.to_string())?;
            fs::set_permissions(&cache, fs::Permissions::from_mode(0o700)).map_err(|e|e.to_string())?;
        }
        let build = (|| {
            let input = BufReader::new(File::open(source).map_err(|e|e.to_string())?);
            let output = BufWriter::new(File::create(cache.join("project.tmp")).map_err(|e|e.to_string())?);
            externalize(input, output, &cache.join("images"))?;
            fs::rename(cache.join("project.tmp"), &prepared).map_err(|e|e.to_string())
        })();
        if let Err(error) = build { let _ = fs::remove_dir_all(cache); return Err(error); }
    }
    fs::read(prepared).map_err(|e|e.to_string())
}

pub fn image(label: &str, name: &str) -> Option<(Vec<u8>, &'static str)> {
    let (hash, extension) = name.split_once('.')?;
    if hash.len()!=64 || !hash.bytes().all(|c|c.is_ascii_hexdigit()) { return None; }
    let mime = match extension { "png"=>"image/png", "jpg"=>"image/jpeg", "webp"=>"image/webp", "avif"=>"image/avif", "gif"=>"image/gif", "svg"=>"image/svg+xml", _=>return None };
    Some((fs::read(folder(label).ok()?.join("images").join(name)).ok()?, mime))
}

pub fn retire(label: &str) {
    if let Ok(_lock) = PREPARATION.lock() { if let Ok(path) = folder(label) {
        let parent=path.parent().unwrap().to_path_buf();
        let _ = fs::remove_dir_all(path); let _ = fs::remove_dir(parent);
    } }
}

/// A previous crash may leave disposable files. Never touch a live process cache.
#[cfg(target_os="linux")]
pub fn cleanup_stale() {
    use std::os::unix::fs::MetadataExt;
    let Ok(owner)=fs::metadata("/proc/self") else {return;};
    let Ok(entries)=fs::read_dir(std::env::temp_dir()) else {return;};
    for entry in entries.flatten() {
        let name=entry.file_name();let name=name.to_string_lossy();
        let Some(pid)=name.strip_prefix("cyoa-manager-readers-").and_then(|s|s.parse::<u32>().ok()) else {continue;};
        if pid==std::process::id() || std::path::Path::new(&format!("/proc/{pid}")).exists() {continue;}
        if let Ok(metadata)=fs::symlink_metadata(entry.path()) {
            if metadata.is_dir() && metadata.uid()==owner.uid() {let _=fs::remove_dir_all(entry.path());}
        }
    }
}

include!("../../tests/rust/reader_assets.rs");
