use std::thread;
use std::time::Instant;

use crate::app::App;
use crate::app::state::{AppState, LoadingInfo, ThreadStatus};
use crate::borg::BorgManager;
use crate::config;

impl App {
    pub fn validate_and_submit_backup(&mut self) {
        let name = self.new_backup_name.trim().to_string();

        if name.is_empty() {
            self.state = AppState::ErrorPopup(self.t.err_name_empty().to_string());
            return;
        }

        if name.contains('/') || name.contains(':') {
            self.state = AppState::ErrorPopup(self.t.err_name_invalid().to_string());
            return;
        }

        let (includes, excludes) = self.file_browser.get_final_paths();

        if includes.is_empty() {
            self.state = AppState::ErrorPopup(self.t.err_no_files().to_string());
            return;
        }

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
            current_file: "Iniciando escaneamento...".to_string(),
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
                includes,
                excludes,
                None,
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
                crate::log_info!("create_backup thread terminated after cancellation; discarding result");
            }
        });
    }
}
