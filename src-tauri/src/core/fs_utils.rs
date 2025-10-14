use std::path::PathBuf;

use serde::Serialize;
use walkdir::WalkDir;

#[derive(Serialize)]
pub struct FileEntry {
    pub value: String,
    pub label: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileEntry>>,
}

fn scan_directory_tree_recursively(path: &PathBuf) -> Vec<FileEntry> {
    let mut result = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = path.is_dir();
            result.push(FileEntry {
                value: path.display().to_string(),
                label: entry.file_name().to_string_lossy().into_owned(),
                is_dir,
                children: if is_dir {
                    Some(scan_directory_tree_recursively(&path))
                } else {
                    None
                },
            });
        }
    }

    result
}

#[tauri::command]
pub fn scan_directory_tree(path: String) -> Vec<FileEntry> {
    scan_directory_tree_recursively(&PathBuf::from(path))
}
