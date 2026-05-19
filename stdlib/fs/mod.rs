use std::path::{Path, PathBuf};

pub fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn write(path: &str, contents: &str) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

pub fn list_dir(path: &str) -> Result<Vec<String>, String> {
    std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .map(|e| e.map(|p| p.file_name().to_string_lossy().to_string()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_dir(path: &str) -> Result<(), String> {
    std::fs::create_dir(path).map_err(|e| e.to_string())
}

pub fn create_dir_all(path: &str) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())
}

pub fn remove(path: &str) -> Result<(), String> {
    if is_dir(path) {
        std::fs::remove_dir(path)
    } else {
        std::fs::remove_file(path)
    }.map_err(|e| e.to_string())
}

pub fn copy(from: &str, to: &str) -> Result<(), String> {
    std::fs::copy(from, to).map(|_| ()).map_err(|e| e.to_string())
}

pub fn rename(from: &str, to: &str) -> Result<(), String> {
    std::fs::rename(from, to).map_err(|e| e.to_string())
}

pub fn metadata(path: &str) -> Result<FileMetadata, String> {
    std::fs::metadata(path)
        .map(|m| FileMetadata {
            size: m.len(),
            is_dir: m.is_dir(),
            is_file: m.is_file(),
            modified: m.modified().ok().map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
        })
        .map_err(|e| e.to_string())
}

pub struct FileMetadata {
    pub size: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub modified: Option<u64>,
}

pub fn current_dir() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn home_dir() -> Option<String> {
    std::env::var("HOME").ok()
}

pub fn temp_dir() -> std::path::PathBuf {
    std::env::temp_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exists() {
        assert!(exists("/"));
    }
}