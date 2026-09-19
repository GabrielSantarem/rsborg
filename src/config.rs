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


fn default_compression() -> String {
    "lz4".to_string()
}

fn default_schedule() -> String {
    "daily".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackupProfile {
    pub id: String,
    pub name: String,
    pub paths: Vec<PathBuf>,
    #[serde(default)]
    pub excludes: Vec<PathBuf>,
    #[serde(default)]
    pub counter: u64,
    #[serde(default = "default_compression")]
    pub compression: String,
    #[serde(default = "default_schedule")]
    pub schedule: String,
}

impl BackupProfile {
    pub fn next_archive_name(&self, now: chrono::DateTime<chrono::Local>) -> String {
        let date_str = now.format("%Y-%m-%d_%H-%M").to_string();
        format!("{}_#{}_{}", self.name, self.counter + 1, date_str)
    }

    pub fn generate_systemd_service(&self, repo_path: &str, passphrase: Option<&str>) -> String {
        let mut paths_str = String::new();
        for p in &self.paths {
            paths_str.push_str(&format!(" \"{}\"", p.display()));
        }
        let mut excludes_str = String::new();
        for e in &self.excludes {
            excludes_str.push_str(&format!(" -e \"{}\"", e.display()));
        }
        let env_pass = if let Some(pass) = passphrase {
            format!("Environment=\"BORG_PASSPHRASE={}\"\n", pass)
        } else {
            String::new()
        };

        format!(
            "[Unit]\nDescription=RsBorg Automated Backup Profile: {}\nAfter=network.target\n\n[Service]\nType=oneshot\n{}ExecStart=/usr/bin/borg create --compression {} \"{}::{}_{{now}}\"{}{}\n",
            self.name,
            env_pass,
            self.compression,
            repo_path,
            self.name,
            excludes_str,
            paths_str
        )
    }

    pub fn generate_systemd_timer(&self) -> String {
        let on_calendar = match self.schedule.as_str() {
            "hourly" => "hourly",
            "weekly" => "weekly",
            "monthly" => "monthly",
            _ => "daily",
        };
        format!(
            "[Unit]\nDescription=RsBorg Timer for Profile: {}\n\n[Timer]\nOnCalendar={}\nPersistent=true\n\n[Install]\nWantedBy=timers.target\n",
            self.name, on_calendar
        )
    }

    pub fn generate_crontab_line(&self, repo_path: &str, passphrase: Option<&str>) -> String {
        let cron_time = match self.schedule.as_str() {
            "hourly" => "0 * * * *",
            "weekly" => "0 3 * * 0",
            "monthly" => "0 3 1 * *",
            _ => "0 2 * * *",
        };
        let mut paths_str = String::new();
        for p in &self.paths {
            paths_str.push_str(&format!(" \"{}\"", p.display()));
        }
        let mut excludes_str = String::new();
        for e in &self.excludes {
            excludes_str.push_str(&format!(" -e \"{}\"", e.display()));
        }
        let pass_prefix = if let Some(pass) = passphrase {
            format!("BORG_PASSPHRASE=\"{}\" ", pass)
        } else {
            String::new()
        };

        format!(
            "{} {}borg create --compression {} \"{}::{}_{{now}}\"{}{}",
            cron_time, pass_prefix, self.compression, repo_path, self.name, excludes_str, paths_str
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub active_repo_id: String,
    pub language: String,
    pub repositories: Vec<RepositoryConfig>,
    #[serde(default)]
    pub profiles: Vec<BackupProfile>,
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
            profiles: Vec::new(),
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
    fn test_backup_profile_naming_and_automation() {
        let profile = BackupProfile {
            id: "prof-1".to_string(),
            name: "BACKUP_DIARIO".to_string(),
            paths: vec![PathBuf::from("/home/user/docs")],
            excludes: vec![PathBuf::from("/home/user/docs/cache")],
            counter: 2,
            compression: "zstd,3".to_string(),
            schedule: "daily".to_string(),
        };

        let now = chrono::Local::now();
        let name = profile.next_archive_name(now);
        assert!(name.starts_with("BACKUP_DIARIO_#3_"));

        let service = profile.generate_systemd_service("/repo", Some("secret"));
        assert!(service.contains("BORG_PASSPHRASE=secret"));
        assert!(service.contains("--compression zstd,3"));

        let cron = profile.generate_crontab_line("/repo", None);
        assert!(cron.starts_with("0 2 * * *"));
        assert!(cron.contains("--compression zstd,3"));
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
