mod audio;
mod core;

use crate::core::fs_utils::scan_directory_tree;

#[tauri::command]
fn preview_audio(file_path: &str) {
    // let samples = decoder::decode_audio_sample(file_path);
}

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
        .invoke_handler(tauri::generate_handler![preview_audio, scan_directory_tree])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
