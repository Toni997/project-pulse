mod audio;
mod core;

use crate::{audio::mixer::preview_audio_file, core::fs_utils::scan_directory_tree};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // send project file name if it was a load
            core::initializator::initialize_project(None);
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            preview_audio_file,
            scan_directory_tree
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
