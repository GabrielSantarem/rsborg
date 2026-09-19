use std::thread;
use std::time::Instant;

use crate::app::App;
use crate::app::state::{AppState, LoadingInfo, PrunePolicyState, ThreadStatus};
use crate::borg::BorgManager;
use crate::config;

impl App {
    pub fn open_prune_policy_modal(&mut self) {
        let policy = self
            .get_active_repo()
            .and_then(|r| r.prune_policy.clone())
            .unwrap_or_default();
        self.state = AppState::PruningPolicy(PrunePolicyState::from_policy(&policy));
    }

    pub fn execute_prune_dry_run(&mut self) {
        if let AppState::PruningPolicy(ref state) = self.state {
            let policy = state.to_policy();

            if let Some(active_id) = self.get_active_repo().map(|r| r.id.clone())
                && let Some(repo) = self
                    .config
                    .repositories
                    .iter_mut()
                    .find(|r| r.id == active_id)
            {
                repo.prune_policy = Some(policy.clone());
                let _ = config::save_config(&self.config);
            }

            let (repo_path, passphrase) = match self.get_active_repo() {
                Some(r) => (r.location.clone(), r.passphrase.clone()),
                None => (config::get_default_repo_path(), None),
            };

            self.loading_info = LoadingInfo {
                message: self.t.loading_pruning_sim().to_string(),
                elapsed_secs: 0,
                original_size: "-".to_string(),
                compressed_size: "-".to_string(),
                deduplicated_size: "-".to_string(),
                files_count: "0".to_string(),
                current_file: "Executando borg prune --dry-run...".to_string(),
                raw_line: String::new(),
                spinner_frame: 0,
            };
            self.loading_start = Some(Instant::now());
            self.state = AppState::Loading;

            let tx = self.tx.clone();
            let manager = BorgManager::new();
            let pol_clone = policy.clone();

            thread::spawn(move || {
                let res =
                    manager.prune_repository(&repo_path, &pol_clone, true, passphrase.as_deref());
                let _ = tx.send(ThreadStatus::DonePruneDryRun(res, pol_clone));
            });
        }
    }

    pub fn confirm_execute_prune(&mut self) {
        if let AppState::PrunePlanView(ref plan_state) = self.state {
            let policy = plan_state.policy.clone();
            let (repo_path, passphrase) = match self.get_active_repo() {
                Some(r) => (r.location.clone(), r.passphrase.clone()),
                None => (config::get_default_repo_path(), None),
            };

            let to_prune_count = plan_state.items.iter().filter(|i| !i.will_keep).count();

            self.loading_info = LoadingInfo {
                message: self.t.loading_pruning_exec().to_string(),
                elapsed_secs: 0,
                original_size: "-".to_string(),
                compressed_size: "-".to_string(),
                deduplicated_size: "-".to_string(),
                files_count: format!("{}", to_prune_count),
                current_file: "Executando borg prune + compact...".to_string(),
                raw_line: String::new(),
                spinner_frame: 0,
            };
            self.loading_start = Some(Instant::now());
            self.state = AppState::Loading;

            let tx = self.tx.clone();
            let manager = BorgManager::new();

            thread::spawn(move || {
                let res =
                    manager.prune_repository(&repo_path, &policy, false, passphrase.as_deref());
                match res {
                    Ok(_) => {
                        let _ = tx.send(ThreadStatus::DonePruneExecute(Ok(to_prune_count)));
                    }
                    Err(e) => {
                        let _ = tx.send(ThreadStatus::DonePruneExecute(Err(e)));
                    }
                }
            });
        }
    }
}
