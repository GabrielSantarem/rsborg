use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Instant;

use crate::app::App;
use crate::app::helpers::generate_id;
use crate::app::state::{AppState, LoadingInfo, ProfileFocus, ProfileWizardState, ThreadStatus};
use crate::borg::BorgManager;
use crate::browser::FileBrowser;
use crate::config::{self, BackupProfile};

impl App {
    pub fn open_profiles_view(&mut self) {
        self.state = AppState::ManagingProfiles { selected_index: 0 };
    }

    pub fn open_create_profile(&mut self) {
        self.file_browser = FileBrowser::new();
        self.file_browser.load_entries();
        self.file_browser.explicit_includes.clear();
        self.file_browser.explicit_excludes.clear();

        self.state = AppState::CreatingProfile(ProfileWizardState {
            focus: ProfileFocus::Name,
            name: String::new(),
            compression_idx: 1, // zstd,3
            schedule_idx: 0,    // daily
        });
    }

    pub fn save_profile(&mut self) {
        let (name, comp_idx, sched_idx) = if let AppState::CreatingProfile(ref wizard) = self.state
        {
            (
                wizard.name.clone(),
                wizard.compression_idx,
                wizard.schedule_idx,
            )
        } else {
            return;
        };

        let trimmed_name = name.trim().to_uppercase().replace(' ', "_");
        if trimmed_name.is_empty() {
            self.state = AppState::ErrorPopup(self.t.err_name_empty().to_string());
            return;
        }

        let (includes, _) = self.file_browser.get_final_paths();
        if includes.is_empty() {
            self.state = AppState::ErrorPopup(self.t.err_no_files().to_string());
            return;
        }

        let compression = crate::ui::profiles::COMPRESSION_OPTIONS
            .get(comp_idx)
            .copied()
            .unwrap_or("lz4")
            .to_string();
        let schedule = crate::ui::profiles::SCHEDULE_OPTIONS
            .get(sched_idx)
            .map(|(s, _)| *s)
            .unwrap_or("daily")
            .to_string();

        let new_profile = BackupProfile {
            id: generate_id("prof"),
            name: trimmed_name,
            paths: self
                .file_browser
                .explicit_includes
                .iter()
                .cloned()
                .collect(),
            excludes: self
                .file_browser
                .explicit_excludes
                .iter()
                .cloned()
                .collect(),
            counter: 0,
            compression,
            schedule,
        };

        self.config.profiles.push(new_profile);
        let _ = config::save_config(&self.config);
        self.state = AppState::ManagingProfiles {
            selected_index: self.config.profiles.len().saturating_sub(1),
        };
    }

    pub fn run_profile_backup(&mut self, profile_idx: usize) {
        let (name, paths, excludes, compression) =
            if let Some(p) = self.config.profiles.get_mut(profile_idx) {
                let archive_name = p.next_archive_name(chrono::Local::now());
                p.counter += 1;
                let inc: Vec<String> = p
                    .paths
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect();
                let exc: Vec<String> = p
                    .excludes
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect();
                let comp = p.compression.clone();
                (archive_name, inc, exc, comp)
            } else {
                return;
            };

        let _ = config::save_config(&self.config);

        let (repo_path, passphrase) = match self.get_active_repo() {
            Some(r) => (r.location.clone(), r.passphrase.clone()),
            None => (config::get_default_repo_path(), None),
        };

        self.loading_info = LoadingInfo {
            message: self.t.loading_creating_fmt(&name),
            elapsed_secs: 0,
            original_size: "0 B".to_string(),
            compressed_size: "0 B".to_string(),
            deduplicated_size: "0 B".to_string(),
            files_count: "0".to_string(),
            current_file: "Iniciando backup de perfil...".to_string(),
            raw_line: String::new(),
            spinner_frame: 0,
        };
        self.loading_start = Some(Instant::now());
        self.state = AppState::Loading;

        self.reset_cancellation_flags();
        let tx = self.tx.clone();
        let manager = BorgManager::with_cancellation(self.active_child_pid.clone());
        let is_cancelled = self.is_task_cancelled.clone();

        thread::spawn(move || {
            let tx_prog = tx.clone();
            let is_canc_prog = is_cancelled.clone();
            let res = manager.create_backup_with_progress(
                &repo_path,
                &name,
                paths,
                excludes,
                Some(&compression),
                passphrase.as_deref(),
                move |p| {
                    if !is_canc_prog.load(std::sync::atomic::Ordering::SeqCst) {
                        let _ = tx_prog.send(ThreadStatus::Progress(p));
                    }
                },
            );

            if !is_cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                match res {
                    Ok(_) => {
                        let _ = tx.send(ThreadStatus::DoneCreate);
                    }
                    Err(e) => {
                        let _ = tx.send(ThreadStatus::Error(e));
                    }
                }
            } else {
                crate::log_info!("run_profile_backup thread terminated after cancellation; discarding result");
            }
        });
    }

    pub fn delete_profile(&mut self, profile_idx: usize) {
        if profile_idx < self.config.profiles.len() {
            self.config.profiles.remove(profile_idx);
            let _ = config::save_config(&self.config);
            let next_idx = profile_idx.min(self.config.profiles.len().saturating_sub(1));
            self.state = AppState::ManagingProfiles {
                selected_index: next_idx,
            };
        }
    }

    pub fn install_systemd_profile(&mut self, profile: &BackupProfile) {
        let (repo_path, passphrase) = match self.get_active_repo() {
            Some(r) => (r.location.clone(), r.passphrase.clone()),
            None => (config::get_default_repo_path(), None),
        };

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let unit_dir = Path::new(&home).join(".config/systemd/user");
        if let Err(e) = std::fs::create_dir_all(&unit_dir) {
            self.state = AppState::ErrorPopup(self.t.err_systemd_dir_fmt(&e.to_string()));
            return;
        }

        let service_file = unit_dir.join(format!("rsborg-{}.service", profile.name.to_lowercase()));
        let timer_file = unit_dir.join(format!("rsborg-{}.timer", profile.name.to_lowercase()));

        let service_content = profile.generate_systemd_service(&repo_path, passphrase.as_deref());
        let timer_content = profile.generate_systemd_timer();

        if let Err(e) = std::fs::write(&service_file, service_content) {
            self.state = AppState::ErrorPopup(self.t.err_service_file_fmt(&e.to_string()));
            return;
        }

        if let Err(e) = std::fs::write(&timer_file, timer_content) {
            self.state = AppState::ErrorPopup(self.t.err_timer_file_fmt(&e.to_string()));
            return;
        }

        // Tenta rodar systemctl --user daemon-reload
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output();

        self.state = AppState::SuccessPopup(self.t.automation_installed_msg().to_string());
    }
}
