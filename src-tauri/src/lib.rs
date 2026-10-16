#[cfg(all(not(debug_assertions), not(feature = "custom-protocol")))]
compile_error!("Standalone release builds require --features custom-protocol; without it the app opens the development server.");

mod commands;
mod metadata;
mod catalog_metadata;
mod author_aliases;
mod websites;
mod update_diff;
mod preferences;
mod history;
mod archive_storage;
mod assets;
mod builds;
mod library;
mod models;
mod perk_index;
mod protocol;
mod reader_assets;
mod reader_settings;

use tauri::Manager;
use commands::*;
use library::load_library;
use models::{Library, SessionStore};
use perk_index::*;
use std::collections::HashMap;
use std::sync::Mutex;

struct StartupFailure(Option<String>);
fn command_error(label: &str, startup: &StartupFailure) -> Option<String> {
    if let Some(error) = &startup.0 { return Some(error.clone()); }
    if label != "main" { return Some("Reader windows cannot call manager commands".into()); }
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut startup_error = None;
    let startup = load_library().unwrap_or_else(|error| {
        startup_error = Some(format!("Library storage could not be opened. No manager commands were run. Preserve your data and fix the storage error, then restart: {error}"));
        eprintln!("Failed to load library storage: {}", error);
        library::LibraryLoadResult {
            library: Library::default(),
            migration_notice: None,
        }
    });
    let library = startup.library;
    let sessions: SessionStore = Mutex::new(HashMap::new());
    let migration_notice: MigrationNoticeState = Mutex::new(startup.migration_notice);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(target_os = "linux")]
            reader_assets::cleanup_stale();
            if let Some(main)=app.get_webview_window("main") {
                main.set_background_color(Some(preferences::background_color(&preferences::load().theme)))?;
                #[cfg(target_os = "linux")]
                main.with_webview(|webview| {
                    use webkit2gtk::{WebViewExt, WebContextExt};
                    if let Some(context)=webview.inner().context() {
                        // Long local documents should not retain a browser history/page cache.
                        context.set_cache_model(webkit2gtk::CacheModel::DocumentViewer);
                    }
                })?;
            }
            Ok(())
        })
        .manage(StartupFailure(startup_error))
        .manage(Mutex::new(library))
        .manage(sessions)
        .manage(models::LegacyRecovery::new(None))
        .manage(migration_notice)
        .register_asynchronous_uri_scheme_protocol("cyoafont",|_ctx,request,responder|{std::thread::spawn(move||responder.respond(preferences::serve_regular_font(request.uri())));})
        .register_asynchronous_uri_scheme_protocol("cyoaview", |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            let label = ctx.webview_label().to_string();
            std::thread::spawn(move || responder.respond(protocol::handle(&app, &label, request)));
        })
        .invoke_handler(|invoke| {
            if let Some(error) = command_error(invoke.message.webview().label(), &invoke.message.webview().state::<StartupFailure>()) { invoke.resolver.reject(error); return true; }
            let handler: fn(tauri::ipc::Invoke<tauri::Wry>)->bool=tauri::generate_handler![
            get_library,
            author_aliases::get_author_aliases,
            author_aliases::save_author_aliases,
            websites::download_website,
            metadata::enrich_library_metadata,
            catalog_metadata::enrich_catalog_metadata,
            catalog_metadata::get_catalog_rows,
            metadata::find_duplicates,
            preferences::set_preferences,
            preferences::get_fonts,
            history::list_archives,
            history::optimize_storage,
            history::star_version,
            history::reconcile_imported_archives,
            history::list_versions,
            history::archive_version,
            history::archive_existing_copy,
            history::restore_version,
            history::list_archive_groups,
            history::rename_archive_group,
            history::rename_archive_version,
            history::move_archive_version,
            take_library_migration_notice,
            add_project,
            clear_library,
            compress_library_cover_images,
            resolve_cover_image_src,
            resolve_local_image_src,
            start_download_project,
            start_download_catalog_entry,
            start_overwrite_catalog_entry,
            start_apply_oversize_project_action,
            remove_project,
            remove_project_from_disk,
            update_project,
            set_project_favorite,
            set_project_viewer_preference,
            apply_oversize_project_action,
            get_project_json,
            scan_folder,
            start_scan_folder,
            get_viewers,
            open_viewer_window,
            get_perk_index_status,
            start_perk_index_task,
            sync_perk_index,
            rebuild_perk_index,
            search_perks,
        ]; handler(invoke) })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
include!("../../tests/rust/command_access.rs");
