use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn default_keep_daily() -> Option<u32> {
    Some(7)
}
fn default_keep_weekly() -> Option<u32> {
    Some(4)
}
fn default_keep_monthly() -> Option<u32> {
    Some(12)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrunePolicy {
    #[serde(default)]
    pub keep_last: Option<u32>,
    #[serde(default = "default_keep_daily")]
    pub keep_daily: Option<u32>,
    #[serde(default = "default_keep_weekly")]
    pub keep_weekly: Option<u32>,
    #[serde(default = "default_keep_monthly")]
    pub keep_monthly: Option<u32>,
    #[serde(default)]
    pub keep_yearly: Option<u32>,
    #[serde(default)]
    pub prefix: Option<String>,
}

impl Default for PrunePolicy {
    fn default() -> Self {
        Self {
            keep_last: None,
            keep_daily: Some(7),
            keep_weekly: Some(4),
            keep_monthly: Some(12),
            keep_yearly: None,
            prefix: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryConfig {
    pub id: String,
    pub name: String,
    pub location: String,
    pub passphrase: Option<String>,
    #[serde(default)]
    pub prune_policy: Option<PrunePolicy>,
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
            prune_policy: Some(PrunePolicy::default()),
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
    let path = Path::new(&home).join(".rsborg");
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

pub fn get_mount_dir() -> PathBuf {
    let rsborg_dir = get_rsborg_dir();
    let mount_dir = rsborg_dir.join("mnt");
    if !mount_dir.exists() {
        let _ = fs::create_dir_all(&mount_dir);
    }
    mount_dir
}

pub fn get_default_restore_dir(archive_name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let restore_base = Path::new(&home).join("Restaurados");
    let safe_name = archive_name.replace(['/', ':'], "_");
    restore_base.join(safe_name)
}

pub fn get_config_file_path() -> PathBuf {
    get_rsborg_dir().join("config.json")
}

pub fn load_config() -> AppConfig {
    let config_path = get_config_file_path();
    if !config_path.exists() {
        let def = AppConfig::default();
        let _ = save_config(&def);
        return def;
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str::<AppConfig>(&content).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) -> io::Result<()> {
    let config_path = get_config_file_path();
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = AppConfig::default();
        let json_str = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json_str).unwrap();

        assert_eq!(config, deserialized);
        assert_eq!(deserialized.repositories.len(), 1);
        assert_eq!(deserialized.repositories[0].id, "default-local");
        assert_eq!(
            deserialized.repositories[0].prune_policy,
            Some(PrunePolicy::default())
        );
    }

    #[test]
    fn test_default_config_has_active_repo() {
        let config = AppConfig::default();
        assert!(!config.active_repo_id.is_empty());
        assert!(
            config
                .repositories
                .iter()
                .any(|r| r.id == config.active_repo_id)
        );
    }

    #[test]
    fn test_corrupt_json_fallback() {
        let corrupt_json = "{ invalid_json: true }";
        let res = serde_json::from_str::<AppConfig>(corrupt_json);
        assert!(res.is_err());
    }

    #[test]
    fn test_mount_and_restore_paths() {
        let mount_path = get_mount_dir();
        assert!(mount_path.ends_with(".rsborg/mnt"));

        let restore_path = get_default_restore_dir("meu_backup:2026/01/01");
        assert!(
            restore_path
                .to_string_lossy()
                .contains("Restaurados/meu_backup_2026_01_01")
        );
    }

    #[test]
    fn test_prune_policy_default_values() {
        let policy = PrunePolicy::default();
        assert_eq!(policy.keep_last, None);
        assert_eq!(policy.keep_daily, Some(7));
        assert_eq!(policy.keep_weekly, Some(4));
        assert_eq!(policy.keep_monthly, Some(12));
        assert_eq!(policy.keep_yearly, None);
        assert_eq!(policy.prefix, None);
    }
}
