use std::path::{Component, Path, PathBuf};

use tauri::http::{Request, Response};
use tauri::{Manager,Emitter};

use crate::commands::{slugify, viewers_base_dir};
use crate::models::SessionStore;

const VIEWER_OVERLAY_SCRIPT_PATH: &str = "__cyoa_manager_viewer_overlay.js";
const VIEWER_OVERLAY_SCRIPT: &str = include_str!("viewer_overlay.js");
const VIEWER_OVERLAY_TEMPLATE_PATH: &str = "__cyoa_manager_viewer_overlay.html";
const VIEWER_OVERLAY_TEMPLATE: &str = include_str!("viewer_overlay.html");

/// Entry point called from `register_uri_scheme_protocol` in lib.rs.
///
/// URL format: `cyoaview://<session-id>/<path>`
///
/// The session-id is a UUID generated when a viewer window is opened.
/// It maps to (project_id, viewer_id) via the SessionStore state.
///
/// All paths, including root-relative ones like `/js/app.js`, resolve
/// correctly because the host is always the session-id, not a path segment.
pub fn handle(app: &tauri::AppHandle, webview_label: &str, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let uri = request.uri();

    // Look up the session by webview label.
    // On Windows, Tauri maps cyoaview://localhost/* → http://cyoaview.localhost/*,
    // so the URI host is always "cyoaview.localhost" — not a session UUID.
    // Using the webview label as the session key works for ALL requests from that
    // viewer window, including root-relative ones (/favicon.ico, /js/app.js).
    let sessions = app.state::<SessionStore>();
    let session = {
        match sessions.lock() {
            Ok(store) => store.get(webview_label).cloned(),
            Err(_) => return err(500, "session store lock poisoned"),
        }
    };
    let Some(session) = session else {
        return err(404, &format!("no session for webview: {}", webview_label));
    };

    let decoded=percent_encoding::percent_decode_str(uri.path()).decode_utf8_lossy();
    let file_path = decoded.trim_start_matches('/');
    // Normalize empty path to index.html
    let file_path = if file_path.is_empty() { "index.html" } else { file_path };

    if session.viewer_id=="website" {
        let snapshot=match crate::websites::read(Path::new(&session.file_path)){Ok(v)=>v,Err(e)=>return err(500,&e)};
        let path=if file_path=="index.html"{"index.html"}else{file_path};let folder=Path::new(&session.file_path).parent().unwrap();
        let Some(full)=safe_join(folder,path)else{return err(400,"Invalid website resource path")};
        let bytes=match std::fs::read(full){Ok(v)=>v,Err(_)=>return err(404,"Not captured in this snapshot. Use Website (online) for sites with live dependencies.")};
        let mime=snapshot.resources.get(path).map(|r|r.mime.as_str()).unwrap_or("application/octet-stream");
        let bytes=if mime.contains("html"){themed_html(&String::from_utf8_lossy(&bytes),&session).into_bytes()}else{bytes};
        return Response::builder().header("Content-Type",mime).body(bytes).unwrap();
    }
    if file_path == "__manager_legacy" {
        let state=app.state::<crate::models::LegacyRecovery>();
        if request.method().as_str()=="POST" && webview_label.starts_with("recovery-") {
            let values:Vec<serde_json::Value>=match serde_json::from_slice(request.body()){Ok(v)=>v,Err(e)=>return err(400,&e.to_string())};
            if let Ok(mut store)=state.lock(){*store=Some(values);}
            let app=app.clone();let label=webview_label.to_string();std::thread::spawn(move||{if let Some(window)=app.get_webview_window(&label){let _=window.close();}if let Ok(mut sessions)=app.state::<SessionStore>().lock(){sessions.remove(&label);}});
            return json(serde_json::json!({"ok":true}));
        }
        let values=state.lock().ok().and_then(|v|v.clone());return json(serde_json::json!({"ready":values.is_some(),"builds":values.unwrap_or_default()}));
    }
    if file_path == "__manager_reader.js" {return javascript(include_str!("viewer_presentation.js").as_bytes().to_vec());}
    if file_path == "__manager_reader_fonts" {return json(serde_json::json!(crate::preferences::get_fonts()));}
    if file_path == "__manager_reader_settings" {
        let result=crate::reader_settings::handle(&session,request.method().as_str(),request.body(),app);
        if result.is_ok() && request.method().as_str()=="POST" {
            for (label,window) in app.webview_windows(){if label.starts_with("viewer-"){let _=window.eval("window.__cyoaManagerPresentation?.reload()");}}
        }
        return match result {Ok(value)=>json(value),Err(error)=>Response::builder().status(400).header("Content-Type","application/json").body(serde_json::to_vec(&serde_json::json!({"error":error})).unwrap()).unwrap()};
    }
    if file_path == "__manager_tools.js" { return javascript(include_str!("viewer_tools.js").as_bytes().to_vec()); }
    if file_path == "__manager_images.js" { return javascript(include_str!("viewer_images.js").as_bytes().to_vec()); }
    if let Some(name) = file_path.strip_prefix("__manager_images/") {
        return match crate::reader_assets::image(webview_label, name) {
            Some((bytes,mime)) => Response::builder().header("Content-Type",mime).header("Cache-Control","no-store").body(bytes).unwrap(),
            None => err(404,"Reader image not found"),
        };
    }
    if file_path == "__manager_recovery.html" { return html(include_str!("viewer_recovery.html").as_bytes().to_vec()); }
    if file_path == "__manager_builds.js" {
        return javascript(include_str!("viewer_builds.js").as_bytes().to_vec());
    }
    if file_path == VIEWER_OVERLAY_SCRIPT_PATH {
        return javascript(VIEWER_OVERLAY_SCRIPT.as_bytes().to_vec());
    }

    if file_path == VIEWER_OVERLAY_TEMPLATE_PATH {
        return html(VIEWER_OVERLAY_TEMPLATE.as_bytes().to_vec());
    }

    if file_path == "__manager_builds" {
        let result = crate::builds::handle(&session, request.method().as_str(), request.body());
        if result.is_ok() && request.method().as_str()=="POST" {let _=app.emit("manager-build-saved",&session.project_id);}
        let (status, value) = match result {
            Ok(value) => (200, value),
            Err(error) => (400, serde_json::json!({"error": error})),
        };
        return Response::builder().status(status).header("Content-Type", "application/json")
            .body(serde_json::to_vec(&value).unwrap()).unwrap();
    }
    if file_path == "__manager_session" {
        return Response::builder().header("Content-Type", "application/json")
            .body(serde_json::to_vec(&serde_json::json!({"cheatsEnabled":session.cheats_enabled,"theme":session.theme,"cyoaFont":session.cyoa_font})).unwrap()).unwrap();
    }
    // Pin this window to the project version opened, even after a library update.
    if file_path == "project.json" {
        let bytes = if session.viewer_id == "icc2-plus" {
            crate::reader_assets::project(webview_label, &session.file_path, || {
                app.state::<SessionStore>().lock().is_ok_and(|store|store.contains_key(webview_label))
            }).map_err(std::io::Error::other)
        } else { std::fs::read(&session.file_path) };
        return match bytes {
            Ok(bytes) => Response::builder().header("Content-Type", "application/json").header("Cache-Control","no-store").body(bytes).unwrap(),
            Err(e) => err(500, &e.to_string()),
        };
    }
    if let Some(response) = serve_viewer_asset(app, &session, file_path) {
        return response;
    }
    if let Some(folder) = Path::new(&session.file_path).parent() {
        if let Some(response) = serve_local_asset(folder, file_path, None) { return response; }
    }

    err(404, &format!("file not found: {}", file_path))
}


fn serve_viewer_asset(
    app: &tauri::AppHandle,
    session: &crate::models::ViewerSession,
    file_path: &str,
) -> Option<Response<Vec<u8>>> {
    let base = viewers_base_dir(Some(app));

    // Find the viewer folder whose slug matches viewer_id
    let viewer_dir = if let Ok(entries) = std::fs::read_dir(&base) {
        let entries: Vec<_> = entries.flatten().collect();
        entries.into_iter().find(|e| {
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && slugify(&e.file_name().to_string_lossy()) == session.viewer_id
        }).map(|e| e.path())
    } else {
        None
    };

    let Some(viewer_dir) = viewer_dir else {
        return None;
    };

    serve_local_asset(&viewer_dir, file_path, Some(session))
}

fn serve_local_asset(base_dir: &Path, file_path: &str, session: Option<&crate::models::ViewerSession>) -> Option<Response<Vec<u8>>> {
    let full_path = safe_join(base_dir, file_path)?;
    let bytes = std::fs::read(&full_path).ok()?;
    let mime = mime_guess::from_path(&full_path)
        .first_raw()
        .unwrap_or("application/octet-stream");

    let bytes = bridge_native_bundle(full_path.file_name().and_then(|n|n.to_str()).unwrap_or(""), bytes);
    let bytes = if session.is_some() && mime.eq_ignore_ascii_case("text/html") {
        inject_viewer_overlay(bytes,session.unwrap())
    } else {
        bytes
    };

    Some(
        Response::builder()
            .header("Content-Type", mime)
            .header("Access-Control-Allow-Origin", "*")
            .status(200)
            .body(bytes)
            .unwrap(),
    )
}

fn bridge_native_bundle(filename: &str, bytes: Vec<u8>) -> Vec<u8> {
    if filename == "app.B6d7tc9y.js" {
        let mut source = String::from_utf8_lossy(&bytes).into_owned();
        // Short resource URLs plus viewport lifetime prevent visited images accumulating.
        let image_source = regex::Regex::new(r#"Le\(([A-Za-z_$][A-Za-z0-9_$]*),"src","#).unwrap();
        source = image_source.replace_all(&source, "window.__cyoaManagerImages.setSource($1,\"src\",").into_owned();
        // Preloading every resource would undo viewport loading on externalized exports.
        source = source.replace("app.preloadExternalImages&&", "false&&");
        // Hooks receive original authored CSS, including per-row/private typography.
        // Semantic score classification uses the engine's evaluated/discounted sign.
        let score_start=source.find("function Dr(");let score_end=source.find("var Fr=");
        if let (Some(start),Some(end))=(score_start,score_end) {
            let fragment=source[start..end].replace("function Dr(e,t){", "function Dr(e,t){const managerScoreKind=()=>t.score.setValue?\"set\":v(p)?\"gain\":\"cost\";").replace("qe(o,v(m))", "window.__cyoaManagerPresentation.style(o,v(m),\"score\",managerScoreKind())");
            source.replace_range(start..end,&fragment);
        }
        for (start_marker,end_marker,title,text) in [("function dl(","We([\"contextmenu\"])","j","q"),("function zl(","function va(","N","G")] {
            if let Some(start)=source.find(start_marker) {if let Some(relative)=source[start..].find(end_marker) {
                let end=start+relative;let mut fragment=source[start..end].to_string();
                for node in ["t","o"] {
                    fragment=fragment.replace(&format!("qe({node},v({title}))"),&format!("window.__cyoaManagerPresentation.style({node},v({title}),\"title\")"));
                    fragment=fragment.replace(&format!("qe({node},v({text}))"),&format!("window.__cyoaManagerPresentation.style({node},v({text}),\"description\")"));
                }
                if start_marker=="function zl(" {fragment=fragment.replace("var i=El(),n=D(i);", "var i=El(),n=D(i);window.__cyoaManagerPresentation.card(i);");}
                source.replace_range(start..end,&fragment);
            }}
        }
        // Keep authored requirement state p intact: deselectChoices must never react
        // to a presentation-only collapse. Svelte tracks parent requirement reads.
        source=source.replace("p=f(()=>Wo(t.row.requireds)),", "p=f(()=>Wo(t.row.requireds)),managerVisible=f(()=>{app.managerSectionRevision;return window.__cyoaManagerPresentation.rowVisible(t.row,Wo,id=>Ft.get(id)?.row)}),");
        for (before,after) in [
            ("ee(J,e=>{v(p)&&e(Z)})","ee(J,e=>{v(p)&&v(managerVisible)&&e(Z)})"),
            ("ee(X,e=>{v(p)&&e(K)})","ee(X,e=>{v(p)&&v(managerVisible)&&e(K)})"),
            (".concat(v(p)?\"\":\" hidden\")",".concat(v(p)&&v(managerVisible)?\"\":\" hidden\")"),
        ] {source=source.replace(before,after);}
        let authored_style=regex::Regex::new(r"qe\(([A-Za-z_$][A-Za-z0-9_$]*),v\(([A-Za-z_$][A-Za-z0-9_$]*)\)\)").unwrap();
        source=authored_style.replace_all(&source,"window.__cyoaManagerPresentation.style($1,v($2),\"body\")").into_owned();
        // Preserve row gates: they are also authored tab/navigation controls.
        // Reveal choices and addons only inside rows the player has actually opened.
        let gates = [
            ("ee(ie,e=>{v(O)&&e(ne)})", "ee(ie,e=>{(app.managerRevealHidden||v(O))&&e(ne)})"),
            ("(app.showAllAddons>0||(!v(o).hideAddon||t.choice.isActive)&&(v(o).showAddon||Wo(v(o).requireds)))&&e(a)", "(app.managerRevealHidden||app.showAllAddons>0||(!v(o).hideAddon||t.choice.isActive)&&(v(o).showAddon||Wo(v(o).requireds)))&&e(a)"),
        ];
        let supported = gates.iter().all(|(before,_)|source.contains(before));
        if supported { for (before,after) in gates {source=source.replace(before,after);} }
        source.push_str("\n;window.__cyoaManagerNative={capture:ho,load:vi,ready:()=>app.rows.length>0&&Ft.size>0};");
        if supported {source.push_str("window.__cyoaManagerNative.setRevealHidden=enabled=>{app.managerRevealHidden=!!enabled};");}
        return source.into_bytes();
    }
    if filename == "index-cEorXV3W.js" {
        let source = String::from_utf8_lossy(&bytes);
        let marker = "function c(){const u=a.value.join(\", \");navigator.clipboard.writeText(u)}return(u,f)=>";
        let replacement = "function c(){const u=a.value.join(\", \");navigator.clipboard.writeText(u)}window.__cyoaManagerImport={capture:()=>a.value.join(\", \") ,load:code=>{n.value=code;l()}};return(u,f)=>";
        let mut source=source.replacen(marker,replacement,1);
        source.push_str("\n;window.__cyoaManagerNative={open:()=>{bc().showImportExportDialog.value=true},close:()=>{bc().showImportExportDialog.value=false}};");
        return source.into_bytes();
    }
    bytes
}

fn themed_html(html:&str,session:&crate::models::ViewerSession)->String{
    let base=crate::preferences::base_color(&session.theme);let mut css=format!("html,body{{background-color:{base} !important;}}html{{color-scheme:{};}}",if session.theme=="latte"{"light"}else{"dark"});
    if !session.cyoa_font.is_empty(){let family=percent_encoding::utf8_percent_encode(&session.cyoa_font,percent_encoding::NON_ALPHANUMERIC);css.push_str(&format!("@font-face{{font-family:'CYOA Reader Regular';src:url('cyoafont://localhost/{family}');font-style:normal;font-weight:400;font-display:swap;}}body *:not(i):not(svg):not([class*=icon]):not(.mdi):not(.material-icons){{font-family:'CYOA Reader Regular',sans-serif !important;font-weight:400 !important;font-synthesis:none !important;}}"));}
    let style=format!(r#"<style id="manager-initial-theme">{css}</style>"#);
    if let Some(end)=html.find('>').filter(|_|html.to_lowercase().starts_with("<head")){return format!("{}{}{}",&html[..end+1],style,&html[end+1..]);}
    if let Some(start)=html.to_lowercase().find("<head"){if let Some(end)=html[start..].find('>'){let end=start+end+1;return format!("{}{}{}",&html[..end],style,&html[end..]);}}
    format!("{style}{html}")
}
fn inject_viewer_overlay(bytes: Vec<u8>,session:&crate::models::ViewerSession) -> Vec<u8> {
    let original = String::from_utf8_lossy(&bytes);
    let mut html = themed_html(&original,session);
    if session.viewer_id == "icc2-plus" {
        html = html.replacen("<head>", "<head><script src=\"/__manager_reader.js\"></script><script src=\"/__manager_images.js\"></script>", 1);
    }
    let injection = format!("<script src=\"/__manager_tools.js\"></script><script src=\"/__manager_builds.js\"></script><script src=\"/{VIEWER_OVERLAY_SCRIPT_PATH}\"></script>");

    if html.contains(VIEWER_OVERLAY_SCRIPT_PATH) {
        return bytes;
    }

    if html.contains("</body>") {
        return html.replacen("</body>", &(injection.clone() + "</body>"), 1).into_bytes();
    }

    if html.contains("</head>") {
        return html.replacen("</head>", &(injection + "</head>"), 1).into_bytes();
    }

    let mut updated = html;
    updated.push_str(&injection);
    updated.into_bytes()
}

fn safe_join(base_dir: &Path, file_path: &str) -> Option<PathBuf> {
    let mut result = base_dir.to_path_buf();
    for component in Path::new(file_path).components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            Component::RootDir => {}
            Component::ParentDir | Component::Prefix(_) => return None,
        }
    }
    Some(result)
}

// ─── Helper ─────────────────────────────────────────────────────────────────

fn json(value:serde_json::Value)->Response<Vec<u8>> {Response::builder().header("Content-Type","application/json").body(serde_json::to_vec(&value).unwrap()).unwrap()}

fn javascript(bytes: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/javascript; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .body(bytes)
        .unwrap()
}

fn html(bytes: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .body(bytes)
        .unwrap()
}

fn err(status: u16, message: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/plain")
        .body(message.as_bytes().to_vec())
        .unwrap()
}

#[cfg(test)]
include!("../../tests/rust/protocol.rs");
