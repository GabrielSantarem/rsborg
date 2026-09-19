use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryConfig {
    pub id: String,
    pub name: String,
    pub location: String,
    pub passphrase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub active_repo_id: String,
    pub language: String,
    pub repositories: Vec<RepositoryConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_repo_path = get_default_repo_path();
        let default_repo = RepositoryConfig {
            id: "default-local".to_string(),
            name: "Padrão Local".to_string(),
            location: default_repo_path,
            passphrase: None,
        };

        Self {
            active_repo_id: default_repo.id.clone(),
            language: "pt".to_string(),
            repositories: vec![default_repo],
        }
    }
}

pub fn get_rsborg_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = std::path::Path::new(&home).join(".rsborg");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

pub fn get_default_repo_path() -> String {
    let rsborg_dir = get_rsborg_dir();
    let backup_dir = rsborg_dir.join("backups");
    if !backup_dir.exists() {
        let _ = fs::create_dir_all(&backup_dir);
    }
    backup_dir.to_string_lossy().to_string()
}

pub fn get_config_file_path() -> PathBuf {
    get_rsborg_dir().join("config.json")
}

pub fn load_config() -> AppConfig {
    let config_path = get_config_file_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }

    let default_config = AppConfig::default();
    let _ = save_config(&default_config);
    default_config
}

pub fn save_config(config: &AppConfig) -> io::Result<()> {
    let config_path = get_config_file_path();
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(config_path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_active_repo() {
        let config = AppConfig::default();
        assert!(!config.repositories.is_empty());
        assert_eq!(config.active_repo_id, config.repositories[0].id);
        assert_eq!(config.language, "pt");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let mut config = AppConfig::default();
        config.repositories.push(RepositoryConfig {
            id: "external-usb".to_string(),
            name: "HD Externo".to_string(),
            location: "/media/tomate/Backup".to_string(),
            passphrase: Some("super_segura".to_string()),
        });

        let json = serde_json::to_string_pretty(&config).expect("Serialization failed");
        let deserialized: AppConfig = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(config, deserialized);
        assert_eq!(deserialized.repositories.len(), 2);
    }

    #[test]
    fn test_corrupt_json_fallback() {
        let corrupt_json = "{ \"active_repo_id\": 12345, INVALID JSON HERE }";
        let parse_result = serde_json::from_str::<AppConfig>(corrupt_json);
        assert!(parse_result.is_err());
    }
}
