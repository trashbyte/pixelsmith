use std::path::Path;
use walkdir::{WalkDir, DirEntry};

pub fn entries_in_path(path: &str) -> Result<Vec<DirEntry>, String> {
    let path = Path::new(path);
    if !path.exists() {
        Err(format!("Path does not exist: {}", path.to_str().unwrap()))
    }
    else {
        let entry_names = WalkDir::new(path).max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .skip(1)
            .collect();
        Ok(entry_names)
    }
}

pub fn files_in_path(path: &str) -> Result<Vec<DirEntry>, String> {
    match entries_in_path(path) {
        Ok(entries) => Ok(entries.iter().cloned().filter(|e| !e.file_type().is_dir()).collect()),
        Err(e) => Err(e)
    }
}

pub fn directories_in_path(path: &str) -> Result<Vec<DirEntry>, String> {
    match entries_in_path(path) {
        Ok(entries) => Ok(entries.iter().cloned().filter(|e| e.file_type().is_dir()).collect()),
        Err(e) => Err(e)
    }
}

pub fn entry_names_in_path(path: &str) -> Result<Vec<String>, String> {
    match entries_in_path(path) {
        Ok(entries) => Ok(entries.iter().cloned().map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string()).collect()),
        Err(e) => Err(e)
    }
}

pub fn file_names_in_path(path: &str) -> Result<Vec<String>, String> {
    match entries_in_path(path) {
        Ok(entries) => Ok(entries.iter().cloned()
                                 .filter(|e| !e.file_type().is_dir())
                                 .map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string())
                                 .collect()),
        Err(e) => Err(e)
    }
}

pub fn directory_names_in_path(path: &str) -> Result<Vec<String>, String> {
    match entries_in_path(path) {
        Ok(entries) => Ok(entries.iter().cloned()
                                        .filter(|e| e.file_type().is_dir())
                                        .map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string())
                                        .collect()),
        Err(e) => Err(e)
    }
}
