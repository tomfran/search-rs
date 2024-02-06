use std::{
    fs::{create_dir_all, File},
    path::Path,
};
use walkdir::{DirEntry, WalkDir};

pub fn create_and_open_file(file_path: &str) -> File {
    let path = Path::new(file_path);
    path.parent().map(create_dir_all);

    File::create(path).expect("error while creating file")
}

pub fn walk_dir(input_dir: &str) -> Vec<DirEntry> {
    WalkDir::new(input_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().is_dir())
        .collect()
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
