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

use tauri::Manager;
use commands::*;
use library::load_library;
use models::{Library, SessionStore};
use perk_index::*;
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let startup = load_library().unwrap_or_else(|error| {
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
        .setup(|app| {if let Some(main)=app.get_webview_window("main"){main.set_background_color(Some(preferences::background_color(&preferences::load().theme)))?;}Ok(())})
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
            if invoke.message.webview().label().starts_with("website-"){invoke.resolver.reject("Website readers cannot call manager commands");return true;}
            let handler: fn(tauri::ipc::Invoke<tauri::Wry>)->bool=tauri::generate_handler![
            get_library,
            author_aliases::get_author_aliases,
            author_aliases::save_author_aliases,
            websites::download_website,
            metadata::enrich_library_metadata,
            catalog_metadata::enrich_catalog_metadata,
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
