use crate::app::App;
use crate::app::helpers::generate_id;
use crate::app::state::AppState;
use crate::config::{self, PrunePolicy, RepositoryConfig};

impl App {
    pub fn switch_active_repo(&mut self, repo_id: String) {
        self.config.active_repo_id = repo_id;
        let _ = config::save_config(&self.config);
        self.load_repository();
    }

    pub fn save_new_repository(&mut self) {
        let name = self.new_repo_name.trim().to_string();
        let location = self.new_repo_location.trim().to_string();
        let pass = self.new_repo_passphrase.trim().to_string();

        if name.is_empty() || location.is_empty() {
            self.state = AppState::ErrorPopup(self.t.err_repo_name_empty().to_string());
            return;
        }

        let new_repo = RepositoryConfig {
            id: generate_id("repo"),
            name,
            location,
            passphrase: if pass.is_empty() { None } else { Some(pass) },
            prune_policy: Some(PrunePolicy::default()),
        };

        let repo_name = new_repo.name.clone();
        self.config.repositories.push(new_repo.clone());
        self.config.active_repo_id = new_repo.id;
        let _ = config::save_config(&self.config);

        self.new_repo_name.clear();
        self.new_repo_location.clear();
        self.new_repo_passphrase.clear();

        self.load_repository();
        self.state = AppState::SuccessPopup(self.t.msg_repo_added_fmt(&repo_name));
    }

    pub fn load_repository(&mut self) {
        let (repo_path, passphrase) = match self.get_active_repo() {
            Some(r) => (r.location.clone(), r.passphrase.clone()),
            None => (config::get_default_repo_path(), None),
        };

        match self
            .borg_manager
            .list_archives(&repo_path, passphrase.as_deref())
        {
            Ok(data) => {
                self.repository = Some(data.repository);
                self.archives = data.archives;
                if !self.archives.is_empty() {
                    self.table_state.select(Some(0));
                } else {
                    self.table_state.select(None);
                }
                self.state = AppState::Browsing;
            }
            Err(e) => {
                if repo_path.contains(".rsborg/backups") {
                    let _ = self
                        .borg_manager
                        .init_repository(&repo_path, passphrase.as_deref());
                    if let Ok(data) = self
                        .borg_manager
                        .list_archives(&repo_path, passphrase.as_deref())
                    {
                        self.repository = Some(data.repository);
                        self.archives = data.archives;
                        if !self.archives.is_empty() {
                            self.table_state.select(Some(0));
                        }
                        self.state = AppState::Browsing;
                        return;
                    }
                }
                self.state =
                    AppState::InitError(format!("Repositório inacessível ({repo_path}):\n{}", e));
            }
        }
    }

    pub fn break_lock_active_repo(&mut self) {
        let (repo_path, passphrase) = match self.get_active_repo() {
            Some(r) => (r.location.clone(), r.passphrase.clone()),
            None => (config::get_default_repo_path(), None),
        };

        crate::log_info!("Attempting break-lock on repository: {}", repo_path);
        match self.borg_manager.break_lock(&repo_path, passphrase.as_deref()) {
            Ok(_) => {
                crate::log_info!("break-lock succeeded on repository: {}", repo_path);
                self.load_repository();
                if self.state == AppState::Browsing {
                    self.state = AppState::SuccessPopup(self.t.msg_break_lock_success().to_string());
                }
            }
            Err(e) => {
                crate::log_error!("break-lock failed on repository {}: {}", repo_path, e);
                self.state = AppState::ErrorPopup(e);
            }
        }
    }
}
