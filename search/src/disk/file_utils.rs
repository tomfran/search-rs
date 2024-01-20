use std::{
    fs::{create_dir_all, File},
    path::Path,
};

pub fn create_and_open_file(file_path: &str) -> File {
    let path = Path::new(file_path);
    path.parent().map(create_dir_all);

    File::create(path).expect("error while creating file")
}
