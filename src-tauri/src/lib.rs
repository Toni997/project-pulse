mod audio;
mod commands;
mod core;

use std::sync::OnceLock;
use tauri::AppHandle;

pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn app_handle() -> &'static AppHandle {
    APP_HANDLE.get().expect("AppHandle not initialized")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let _ = APP_HANDLE.set(app.handle().clone());
            // send project file name if it was a load
            core::logger::setup_logger().expect("Failed to initialize logging");
            core::initializator::initialize_project(None);
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::fs::fs_scan_directory_tree,
            commands::preview::preview_play,
            commands::transport::transport_stop,
            commands::mixer::mixer_add_audio_track,
            commands::mixer::mixer_assign_audio_to_track,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
