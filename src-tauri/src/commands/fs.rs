use crate::core::fs_utils::{self, FileEntry};

#[tauri::command]
pub async fn fs_scan_directory_tree(path: String) -> Result<Vec<FileEntry>, String> {
    fs_utils::scan_directory_tree(path).await
}
