use std::thread;
use std::time::Instant;

use crate::app::App;
use crate::app::state::{AppState, CheckWizardState, DiffWizardState, LoadingInfo, ThreadStatus};
use crate::borg::{BackupProgress, BorgCheckMode, BorgManager};
use crate::config;

impl App {
    pub fn open_check_wizard(&mut self) {
        let selected_archive = self
            .table_state
            .selected()
            .and_then(|i| self.archives.get(i))
            .map(|a| a.name.clone());

        self.state = AppState::CheckWizard(CheckWizardState {
            target_archive: selected_archive,
            check_mode: BorgCheckMode::Standard,
            check_archive_only: false,
        });
    }

    pub fn start_check(&mut self) {
        if let AppState::CheckWizard(ref wizard_state) = self.state {
            let archive_target = if wizard_state.check_archive_only {
                wizard_state.target_archive.clone()
            } else {
                None
            };
            let mode = wizard_state.check_mode;

            let (repo_path, passphrase) = match self.get_active_repo() {
                Some(r) => (r.location.clone(), r.passphrase.clone()),
                None => (config::get_default_repo_path(), None),
            };

            let target_display = match archive_target.as_deref() {
                Some(name) => format!("Backup: {}", name),
                None => format!("Repo: {}", repo_path),
            };

            let mode_display = match mode {
                BorgCheckMode::RepositoryOnly => "Quick (--repository-only)".to_string(),
                BorgCheckMode::Standard => "Standard".to_string(),
                BorgCheckMode::VerifyData => "Deep (--verify-data)".to_string(),
                BorgCheckMode::Repair => "Repair (--repair)".to_string(),
            };

            self.loading_info = LoadingInfo {
                message: self.t.loading_checking_fmt(&target_display),
                elapsed_secs: 0,
                original_size: "-".to_string(),
                compressed_size: "-".to_string(),
                deduplicated_size: "-".to_string(),
                files_count: "0".to_string(),
                current_file: "Iniciando diagnóstico borg check...".to_string(),
                raw_line: String::new(),
                spinner_frame: 0,
            };
            self.loading_start = Some(Instant::now());
            self.state = AppState::Loading;

            self.reset_cancellation_flags();
            let tx = self.tx.clone();
            let manager = BorgManager::with_cancellation(self.active_child_pid.clone());
            let is_cancelled = self.is_task_cancelled.clone();
            let t_disp = target_display.clone();
            let m_disp = mode_display.clone();

            thread::spawn(move || {
                let tx_prog = tx.clone();
                let is_canc_prog = is_cancelled.clone();
                let res = manager.check_repository(
                    &repo_path,
                    archive_target.as_deref(),
                    mode,
                    passphrase.as_deref(),
                    move |line| {
                        if !is_canc_prog.load(std::sync::atomic::Ordering::SeqCst) {
                            let _ = tx_prog.send(ThreadStatus::Progress(BackupProgress {
                                current_file: line.clone(),
                                raw_line: line,
                                ..Default::default()
                            }));
                        }
                    },
                );

                if !is_cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    let _ = tx.send(ThreadStatus::DoneCheck(res, t_disp, m_disp));
                } else {
                    crate::log_info!("check_repository thread terminated after cancellation; discarding result");
                }
            });
        }
    }

    pub fn open_diff_wizard(&mut self) {
        if self.archives.len() < 2 {
            self.state = AppState::ErrorPopup(self.t.diff_err_need_two().to_string());
            return;
        }

        let selected_idx = self.table_state.selected().unwrap_or(0);
        let base_archive = match self.archives.get(selected_idx) {
            Some(a) => a.name.clone(),
            None => match self.archives.first() {
                Some(a) => a.name.clone(),
                None => return,
            },
        };

        let candidates: Vec<String> = self
            .archives
            .iter()
            .map(|a| a.name.clone())
            .filter(|name| name != &base_archive)
            .collect();

        if candidates.is_empty() {
            self.state = AppState::ErrorPopup(self.t.diff_err_need_two().to_string());
            return;
        }

        self.state = AppState::DiffWizard(DiffWizardState {
            base_archive,
            candidates,
            selected_candidate_idx: 0,
            content_only: false,
        });
    }

    pub fn start_diff(&mut self) {
        if let AppState::DiffWizard(ref wizard_state) = self.state {
            let base = wizard_state.base_archive.clone();
            let target = match wizard_state
                .candidates
                .get(wizard_state.selected_candidate_idx)
            {
                Some(name) => name.clone(),
                None => return,
            };
            let content_only = wizard_state.content_only;

            let (repo_path, passphrase) = match self.get_active_repo() {
                Some(r) => (r.location.clone(), r.passphrase.clone()),
                None => (config::get_default_repo_path(), None),
            };

            self.loading_info = LoadingInfo {
                message: self.t.loading_diffing_fmt(&base, &target),
                elapsed_secs: 0,
                original_size: "-".to_string(),
                compressed_size: "-".to_string(),
                deduplicated_size: "-".to_string(),
                files_count: "0".to_string(),
                current_file: "Iniciando comparação borg diff...".to_string(),
                raw_line: String::new(),
                spinner_frame: 0,
            };
            self.loading_start = Some(Instant::now());
            self.state = AppState::Loading;

            self.reset_cancellation_flags();
            let tx = self.tx.clone();
            let manager = BorgManager::with_cancellation(self.active_child_pid.clone());
            let is_cancelled = self.is_task_cancelled.clone();
            let b_clone = base.clone();
            let t_clone = target.clone();

            thread::spawn(move || {
                let tx_prog = tx.clone();
                let is_canc_prog = is_cancelled.clone();
                let res = manager.diff_archives(
                    &repo_path,
                    &base,
                    &target,
                    content_only,
                    passphrase.as_deref(),
                    move |line| {
                        if !is_canc_prog.load(std::sync::atomic::Ordering::SeqCst) {
                            let _ = tx_prog.send(ThreadStatus::Progress(BackupProgress {
                                current_file: line.clone(),
                                raw_line: line,
                                ..Default::default()
                            }));
                        }
                    },
                );

                if !is_cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                    let _ = tx.send(ThreadStatus::DoneDiff(res, b_clone, t_clone));
                } else {
                    crate::log_info!("diff_archives thread terminated after cancellation; discarding result");
                }
            });
        }
    }
}
