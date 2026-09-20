use crate::ui::theme::{Theme, ThemeMode};
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

use crate::borg::{BackupArchive, BorgManager, RepositoryInfo};
use crate::browser::FileBrowser;
use crate::checker;
use crate::config::{self, AppConfig, RepositoryConfig};
use crate::i18n::{Language, Translator};

pub mod archives;
pub mod backup;
pub mod diagnostics;
pub mod helpers;
pub mod profiles;
pub mod prune;
pub mod repositories;
pub mod state;
pub mod tasks;

#[cfg(test)]
mod tests;

pub use state::*;

pub struct App {
    pub state: AppState,
    pub config: AppConfig,
    pub borg_manager: BorgManager,
    pub borg_version: Option<String>,
    pub repository: Option<RepositoryInfo>,
    pub archives: Vec<BackupArchive>,
    pub table_state: TableState,
    pub should_quit: bool,
    pub t: Translator,
    pub theme: Theme,

    // Criação de backup
    pub create_focus: CreateFocus,
    pub new_backup_name: String,
    pub file_browser: FileBrowser,

    // Loading & Telemetria
    pub loading_info: LoadingInfo,
    pub loading_start: Option<Instant>,

    // Montagem FUSE
    pub mounted_archives: HashMap<String, PathBuf>,

    // Gerenciamento de Repositórios
    pub repo_list_index: usize,
    pub add_repo_focus: usize,
    pub new_repo_name: String,
    pub new_repo_location: String,
    pub new_repo_passphrase: String,

    pub tx: Sender<ThreadStatus>,
    pub rx: Receiver<ThreadStatus>,

    pub active_child_pid: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub is_task_cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let config = config::load_config();
        let lang = if config.language == "en" {
            Language::En
        } else {
            Language::Pt
        };

        let theme_mode = if config.theme.eq_ignore_ascii_case("catppuccin") {
            ThemeMode::Catppuccin
        } else {
            ThemeMode::Rust
        };
        let theme = Theme::new(theme_mode);

        Self {
            state: AppState::Initializing,
            config,
            borg_manager: BorgManager::new(),
            borg_version: None,
            repository: None,
            archives: Vec::new(),
            table_state: TableState::default(),
            should_quit: false,
            t: Translator::new(lang),
            theme,

            create_focus: CreateFocus::Name,
            new_backup_name: String::new(),
            file_browser: FileBrowser::new(),

            loading_info: LoadingInfo::default(),
            loading_start: None,

            mounted_archives: HashMap::new(),

            repo_list_index: 0,
            add_repo_focus: 0,
            new_repo_name: String::new(),
            new_repo_location: String::new(),
            new_repo_passphrase: String::new(),

            tx,
            rx,

            active_child_pid: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            is_task_cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn reset_cancellation_flags(&self) {
        self.active_child_pid.store(0, std::sync::atomic::Ordering::SeqCst);
        self.is_task_cancelled.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn open_settings(&mut self) {
        self.state = AppState::Settings(crate::app::state::SettingsState::default());
    }

    pub fn open_log_viewer(&mut self) {
        let lines = crate::logger::read_log_entries();
        self.state = AppState::LogViewer(crate::app::state::LogViewerState::new(lines));
    }

    pub fn cancel_active_task(&mut self) {
        self.is_task_cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        let pid = self.active_child_pid.swap(0, std::sync::atomic::Ordering::SeqCst);

        if pid > 0 {
            crate::log_warn!("User requested task cancellation. Terminating Borg PID {}", pid);
            let _ = std::process::Command::new("kill")
                .args(["-15", &pid.to_string()])
                .output();

            std::thread::sleep(std::time::Duration::from_millis(150));
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();
        } else {
            crate::log_info!("Cancellation requested; no active child process was running.");
        }

        while self.rx.try_recv().is_ok() {}

        self.loading_start = None;
        self.state = AppState::Browsing;
    }

    pub fn get_active_repo(&self) -> Option<&RepositoryConfig> {
        self.config
            .repositories
            .iter()
            .find(|r| r.id == self.config.active_repo_id)
            .or_else(|| self.config.repositories.first())
    }

    pub fn toggle_language(&mut self) {
        let (new_lang, code) = match self.t.lang {
            Language::Pt => (Language::En, "en"),
            Language::En => (Language::Pt, "pt"),
        };
        self.t = Translator::new(new_lang);
        self.config.language = code.to_string();
        let _ = config::save_config(&self.config);
    }

    pub fn toggle_theme(&mut self) {
        let new_mode = self.theme.mode.toggle();
        self.config.theme = match new_mode {
            ThemeMode::Rust => "rust".to_string(),
            ThemeMode::Catppuccin => "catppuccin".to_string(),
        };
        self.theme = Theme::new(new_mode);
        let _ = config::save_config(&self.config);
    }

    pub fn set_language(&mut self, lang_code: &str) {
        let lang = if lang_code.eq_ignore_ascii_case("en") {
            Language::En
        } else {
            Language::Pt
        };
        self.t = Translator::new(lang);
    }

    pub fn override_repo_path(&mut self, path: &str) {
        if let Some(existing) = self.config.repositories.iter().find(|r| r.location == path) {
            self.config.active_repo_id = existing.id.clone();
        } else {
            let id = format!("cli-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "CLI Repo".to_string());
            let repo = config::RepositoryConfig {
                id: id.clone(),
                name,
                location: path.to_string(),
                passphrase: None,
                prune_policy: Some(config::PrunePolicy::default()),
            };
            self.config.repositories.push(repo);
            self.config.active_repo_id = id;
        }
    }

    pub fn on_start(&mut self) {
        if let Err(e) = checker::check_instances() {
            self.state = AppState::InitError(e);
            return;
        }

        match self.borg_manager.verify_installation() {
            Ok(version) => {
                self.borg_version = Some(version);
                self.load_repository();
            }
            Err(e) => {
                self.state = AppState::InitError(e);
            }
        }
    }

    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.archives.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.archives.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn quit(&mut self) {
        for mount_path in self.mounted_archives.values() {
            let _ = self.borg_manager.umount_archive(mount_path);
        }
        self.mounted_archives.clear();
        self.should_quit = true;
    }
}
