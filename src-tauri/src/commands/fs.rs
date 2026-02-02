use crate::core::fs_utils::{self, FileEntry};
use log::info;

#[tauri::command]
pub async fn fs_scan_directory_tree(path: String) -> Result<Vec<FileEntry>, String> {
    info!("fs::scan_directory_tree");
    fs_utils::scan_directory_tree(path).await
}
