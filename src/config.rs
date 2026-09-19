use std::path::PathBuf;

pub fn get_rsborg_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = std::path::Path::new(&home).join(".rsborg");

    // Tenta criar o diretório, ignoramos o erro caso já exista.
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
}

pub fn get_default_repo() -> String {
    let rsborg_dir = get_rsborg_dir();
    let backup_dir = rsborg_dir.join("backups");

    if !backup_dir.exists() {
        let _ = std::fs::create_dir_all(&backup_dir);
    }

    backup_dir.to_string_lossy().to_string()
}
