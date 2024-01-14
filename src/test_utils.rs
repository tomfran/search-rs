#[cfg(test)]
pub mod utils {
    use tempdir::TempDir;

    pub fn create_temporary_dir_path() -> String {
        TempDir::new("tmp")
            .expect("Failed to create temporary directory")
            .path()
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn create_temporary_file_path(prefix: &str) -> String {
        let temp_dir = TempDir::new("tmp").expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join(prefix);
        file_path.to_str().unwrap().to_string()
    }
}
