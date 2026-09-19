use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crate::borg::{
    BackupArchive, BackupProgress, BorgCheckMode, BorgManager, RepositoryInfo,
};
use crate::browser::FileBrowser;
use crate::checker;
use crate::config::{self, AppConfig, PrunePolicy, RepositoryConfig};
use crate::i18n::{Language, Translator};

pub mod state;
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
        }
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

    pub fn validate_and_submit_backup(&mut self) {
        let name = self.new_backup_name.trim().to_string();

        if name.is_empty() {
            self.state =
                AppState::ErrorPopup(self.t.err_name_empty().to_string());
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

        let tx = self.tx.clone();
        let manager = BorgManager::new();

        thread::spawn(move || {
            let tx_prog = tx.clone();
            let res = manager.create_backup_with_progress(
                &repo_path,
                &name,
                includes,
                excludes,
                passphrase.as_deref(),
                move |p| {
                    let _ = tx_prog.send(ThreadStatus::Progress(p));
                },
            );

            match res {
                Ok(_) => {
                    let _ = tx.send(ThreadStatus::DoneCreate);
                }
                Err(e) => {
                    let _ = tx.send(ThreadStatus::Error(e));
                }
            }
        });
    }

    pub fn ask_delete_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i) {
                self.state = AppState::ConfirmDelete(archive.name.clone());
            }
    }

    pub fn confirm_delete_archive(&mut self, archive_name: String) {
        let (repo_path, passphrase) = match self.get_active_repo() {
            Some(r) => (r.location.clone(), r.passphrase.clone()),
            None => (config::get_default_repo_path(), None),
        };

        self.loading_info = LoadingInfo {
            message: self.t.loading_deleting_fmt(&archive_name),
            elapsed_secs: 0,
            original_size: "0 B".to_string(),
            compressed_size: "0 B".to_string(),
            deduplicated_size: "0 B".to_string(),
            files_count: "0".to_string(),
            current_file: "Executando borg delete + compact...".to_string(),
            raw_line: String::new(),
            spinner_frame: 0,
        };
        self.loading_start = Some(Instant::now());
        self.state = AppState::Loading;

        let tx = self.tx.clone();
        let manager = BorgManager::new();

        thread::spawn(move || {
            match manager.delete_archive(&repo_path, &archive_name, passphrase.as_deref()) {
                Ok(_) => {
                    let _ = tx.send(ThreadStatus::DoneDelete);
                }
                Err(e) => {
                    let _ = tx.send(ThreadStatus::Error(e));
                }
            }
        });
    }

    pub fn inspect_selected_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i) {
                let name = archive.name.clone();
                let (repo_path, passphrase) = match self.get_active_repo() {
                    Some(r) => (r.location.clone(), r.passphrase.clone()),
                    None => (config::get_default_repo_path(), None),
                };

                if repo_path.contains(':') && !repo_path.starts_with('/') {
                    self.state = AppState::ErrorPopup(self.t.err_fuse_unsupported().to_string());
                    return;
                }

                self.loading_info = LoadingInfo {
                    message: self.t.loading_inspecting_fmt(&name),
                    elapsed_secs: 0,
                    original_size: "0 B".to_string(),
                    compressed_size: "0 B".to_string(),
                    deduplicated_size: "0 B".to_string(),
                    files_count: "0".to_string(),
                    current_file: "Executando borg list...".to_string(),
                    raw_line: String::new(),
                    spinner_frame: 0,
                };
                self.loading_start = Some(Instant::now());
                self.state = AppState::Loading;

                let tx = self.tx.clone();
                let manager = BorgManager::new();

                thread::spawn(move || {
                    let res =
                        manager.list_archive_contents(&repo_path, &name, passphrase.as_deref());
                    let _ = tx.send(ThreadStatus::DoneInspect(res, name));
                });
            }
    }

    pub fn ask_restore_selected_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i) {
                let dest = config::get_default_restore_dir(&archive.name);
                self.state = AppState::ConfirmRestore(RestoreRequest {
                    archive_name: archive.name.clone(),
                    destination_path: dest.to_string_lossy().to_string(),
                    paths_to_extract: vec![],
                });
            }
    }

    pub fn ask_restore_specific_file(&mut self, file_path: String) {
        if let AppState::InspectArchive(ref inspect) = self.state {
            let archive_name = inspect.archive_name.clone();
            let dest = config::get_default_restore_dir(&archive_name);
            self.state = AppState::ConfirmRestore(RestoreRequest {
                archive_name,
                destination_path: dest.to_string_lossy().to_string(),
                paths_to_extract: vec![file_path],
            });
        }
    }

    pub fn confirm_restore(&mut self) {
        if let AppState::ConfirmRestore(ref req) = self.state {
            let archive_name = req.archive_name.clone();
            let dest_path = PathBuf::from(req.destination_path.trim());
            let paths = req.paths_to_extract.clone();

            if req.destination_path.trim().is_empty() {
                self.state =
                    AppState::ErrorPopup(self.t.err_repo_loc_empty().to_string());
                return;
            }

            let (repo_path, passphrase) = match self.get_active_repo() {
                Some(r) => (r.location.clone(), r.passphrase.clone()),
                None => (config::get_default_repo_path(), None),
            };

            let is_granular = !paths.is_empty();
            let msg = if is_granular {
                format!("{}: {}", self.t.loading_restoring_fmt(&archive_name), paths[0])
            } else {
                self.t.loading_restoring_fmt(&archive_name)
            };

            self.loading_info = LoadingInfo {
                message: msg,
                elapsed_secs: 0,
                original_size: "-".to_string(),
                compressed_size: "-".to_string(),
                deduplicated_size: "-".to_string(),
                files_count: "0".to_string(),
                current_file: "Iniciando extração de arquivos...".to_string(),
                raw_line: String::new(),
                spinner_frame: 0,
            };
            self.loading_start = Some(Instant::now());
            self.state = AppState::Loading;

            let tx = self.tx.clone();
            let manager = BorgManager::new();
            let dest_clone = dest_path.clone();

            thread::spawn(move || {
                let tx_prog = tx.clone();
                let mut count = 0;
                let res = manager.extract_archive(
                    &repo_path,
                    &archive_name,
                    &dest_clone,
                    &paths,
                    passphrase.as_deref(),
                    move |extracted_file| {
                        count += 1;
                        let _ = tx_prog.send(ThreadStatus::Progress(BackupProgress {
                            raw_line: format!("Extraindo {}", extracted_file),
                            files_count: format!("{}", count),
                            current_file: extracted_file,
                            ..Default::default()
                        }));
                    },
                );

                match res {
                    Ok(_) => {
                        let _ = tx.send(ThreadStatus::DoneRestore(
                            dest_clone.to_string_lossy().to_string(),
                        ));
                    }
                    Err(e) => {
                        let _ = tx.send(ThreadStatus::Error(e));
                    }
                }
            });
        }
    }

    pub fn mount_selected_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i) {
                let name = archive.name.clone();

                if let Some(existing) = self.mounted_archives.get(&name) {
                    self.state = AppState::SuccessPopup(format!(
                        "Backup '{}' já está montado em:\n{}\n\nUse [u] para desmontar.",
                        name,
                        existing.display()
                    ));
                    return;
                }

                let mount_point = config::get_mount_dir().join(&name);
                let (repo_path, passphrase) = match self.get_active_repo() {
                    Some(r) => (r.location.clone(), r.passphrase.clone()),
                    None => (config::get_default_repo_path(), None),
                };

                if repo_path.contains(':') && !repo_path.starts_with('/') {
                    self.state = AppState::ErrorPopup(self.t.err_fuse_unsupported().to_string());
                    return;
                }

                match self.borg_manager.mount_archive(
                    &repo_path,
                    &name,
                    &mount_point,
                    passphrase.as_deref(),
                ) {
                    Ok(_) => {
                        self.mounted_archives
                            .insert(name.clone(), mount_point.clone());
                        self.state = AppState::SuccessPopup(self.t.msg_mount_success_fmt(&name, &mount_point.display().to_string()));
                    }
                    Err(e) => {
                        self.state =
                            AppState::ErrorPopup(format!("Erro ao montar backup FUSE:\n{}", e));
                    }
                }
            }
    }

    pub fn umount_selected_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i) {
                let name = archive.name.clone();

                if let Some(mount_point) = self.mounted_archives.remove(&name) {
                    match self.borg_manager.umount_archive(&mount_point) {
                        Ok(_) => {
                            self.state = AppState::SuccessPopup(self.t.msg_umount_success().to_string());
                        }
                        Err(e) => {
                            self.mounted_archives.insert(name, mount_point);
                            self.state = AppState::ErrorPopup(format!("Erro ao desmontar:\n{}", e));
                        }
                    }
                } else {
                    self.state = AppState::ErrorPopup(self.t.err_not_mounted_fmt(&name));
                }
            }
    }

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

            let tx = self.tx.clone();
            let manager = BorgManager::new();
            let t_disp = target_display.clone();
            let m_disp = mode_display.clone();

            thread::spawn(move || {
                let tx_prog = tx.clone();
                let res = manager.check_repository(
                    &repo_path,
                    archive_target.as_deref(),
                    mode,
                    passphrase.as_deref(),
                    move |line| {
                        let _ = tx_prog.send(ThreadStatus::Progress(BackupProgress {
                            current_file: line.clone(),
                            raw_line: line,
                            ..Default::default()
                        }));
                    },
                );

                let _ = tx.send(ThreadStatus::DoneCheck(res, t_disp, m_disp));
            });
        }
    }

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
            id: format!(
                "repo-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            ),
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

    pub fn handle_background_tasks(&mut self) {
        if self.state == AppState::Loading {
            if let Some(start) = self.loading_start {
                self.loading_info.elapsed_secs = start.elapsed().as_secs();
            }
            self.loading_info.spinner_frame = (self.loading_info.spinner_frame + 1) % 10;
        }

        while let Ok(status) = self.rx.try_recv() {
            match status {
                ThreadStatus::Progress(p) => {
                    if !p.original_size.is_empty() {
                        self.loading_info.original_size = p.original_size;
                    }
                    if !p.compressed_size.is_empty() {
                        self.loading_info.compressed_size = p.compressed_size;
                    }
                    if !p.deduplicated_size.is_empty() {
                        self.loading_info.deduplicated_size = p.deduplicated_size;
                    }
                    if !p.files_count.is_empty() {
                        self.loading_info.files_count = p.files_count;
                    }
                    if !p.current_file.is_empty() {
                        self.loading_info.current_file = p.current_file;
                    }
                    self.loading_info.raw_line = p.raw_line;
                }
                ThreadStatus::DoneCreate => {
                    self.load_repository();
                    let created_name = std::mem::take(&mut self.new_backup_name);
                    self.file_browser.explicit_includes.clear();
                    self.file_browser.explicit_excludes.clear();
                    self.loading_start = None;
                    self.state = if !created_name.is_empty() {
                        AppState::SuccessPopup(self.t.msg_backup_success_fmt(&created_name))
                    } else {
                        AppState::Browsing
                    };
                }
                ThreadStatus::DoneDelete => {
                    self.load_repository();
                    self.loading_start = None;
                    self.state = AppState::Browsing;
                }
                ThreadStatus::DoneRestore(target) => {
                    self.loading_start = None;
                    self.state = AppState::SuccessPopup(self.t.msg_restore_success_fmt(&target));
                }
                ThreadStatus::DoneInspect(res, name) => {
                    self.loading_start = None;
                    match res {
                        Ok(entries) => {
                            self.state = AppState::InspectArchive(InspectState {
                                archive_name: name,
                                entries,
                                selected_index: 0,
                            });
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::DonePruneDryRun(res, policy) => {
                    self.loading_start = None;
                    match res {
                        Ok(items) => {
                            self.state = AppState::PrunePlanView(PrunePlanState {
                                items,
                                selected_index: 0,
                                policy,
                            });
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::DonePruneExecute(res) => {
                    self.loading_start = None;
                    match res {
                        Ok(pruned_count) => {
                            self.load_repository();
                            self.state = AppState::SuccessPopup(
                                self.t.msg_prune_success_fmt(pruned_count)
                            );
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::DoneCheck(res, target_display, mode_display) => {
                    self.loading_start = None;
                    match res {
                        Ok(result) => {
                            self.state = AppState::CheckResultView(CheckResultState {
                                result,
                                target_display,
                                mode_display,
                                log_scroll: 0,
                            });
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::Error(e) => {
                    self.loading_start = None;
                    self.state = AppState::ErrorPopup(e);
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initial_state() {
        let app = App::new();
        assert_eq!(app.state, AppState::Initializing);
        assert!(!app.should_quit);
        assert!(app.mounted_archives.is_empty());
    }

    #[test]
    fn test_validate_and_submit_backup_empty_name() {
        let mut app = App::new();
        app.new_backup_name = "   ".to_string();
        app.validate_and_submit_backup();

        match app.state {
            AppState::ErrorPopup(msg) => {
                assert!(msg.contains("não pode ser vazio") || msg.contains("cannot be empty"));
            }
            _ => panic!("Esperava ErrorPopup ao submeter nome vazio"),
        }
    }

    #[test]
    fn test_validate_and_submit_backup_invalid_characters() {
        let mut app = App::new();
        app.new_backup_name = "meu/backup:teste".to_string();
        app.file_browser
            .explicit_includes
            .insert(std::path::PathBuf::from("/tmp"));
        app.validate_and_submit_backup();

        match app.state {
            AppState::ErrorPopup(msg) => {
                assert!(msg.contains("caracteres reservados") || msg.contains("reserved characters"));
            }
            _ => panic!("Esperava ErrorPopup ao submeter nome com caracteres reservados"),
        }
    }

    #[test]
    fn test_validate_and_submit_backup_no_files_selected() {
        let mut app = App::new();
        app.new_backup_name = "meu_backup".to_string();
        app.file_browser.explicit_includes.clear();
        app.validate_and_submit_backup();

        match app.state {
            AppState::ErrorPopup(msg) => {
                assert!(msg.contains("precisa selecionar") || msg.contains("must select"));
            }
            _ => panic!("Esperava ErrorPopup ao submeter sem arquivos selecionados"),
        }
    }

    #[test]
    fn test_active_repo_fallback() {
        let mut app = App::new();
        app.config.active_repo_id = "non-existent-id".to_string();
        let active = app.get_active_repo();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, app.config.repositories[0].id);
    }

    #[test]
    fn test_empty_archive_navigation_no_panic() {
        let mut app = App::new();
        app.archives.clear();
        app.table_state.select(None);

        app.next();
        assert_eq!(app.table_state.selected(), Some(0));

        app.previous();
        assert_eq!(app.table_state.selected(), Some(0));
    }

    #[test]
    fn test_ask_delete_archive_no_selection_no_panic() {
        let mut app = App::new();
        app.archives.clear();
        app.table_state.select(None);

        app.ask_delete_archive();
        assert_eq!(app.state, AppState::Initializing);
    }

    #[test]
    fn test_ask_restore_archive_no_selection_no_panic() {
        let mut app = App::new();
        app.archives.clear();
        app.table_state.select(None);

        app.ask_restore_selected_archive();
        assert_eq!(app.state, AppState::Initializing);
    }

    #[test]
    fn test_umount_archive_not_mounted_gives_error() {
        let mut app = App::new();
        app.archives.push(BackupArchive {
            archive: "b1".to_string(),
            barchive: "b1".to_string(),
            id: "1".to_string(),
            name: "b1".to_string(),
            start: "2023-01-01".to_string(),
            time: "2023-01-01".to_string(),
        });
        app.table_state.select(Some(0));

        app.umount_selected_archive();
        match app.state {
            AppState::ErrorPopup(msg) => {
                assert!(msg.contains("não está montado") || msg.contains("not mounted"));
            }
            _ => panic!("Esperava ErrorPopup avisando que não está montado"),
        }
    }


    #[test]
    fn test_open_check_wizard_and_mode() {
        let mut app = App::new();
        app.open_check_wizard();

        match app.state {
            AppState::CheckWizard(ref state) => {
                assert_eq!(state.check_mode, BorgCheckMode::Standard);
                assert!(!state.check_archive_only);
            }
            _ => panic!("Esperava AppState::CheckWizard"),
        }
    }

    #[test]
    fn test_prune_policy_state_conversion() {
        let policy = PrunePolicy {
            keep_last: Some(5),
            keep_daily: Some(7),
            keep_weekly: Some(4),
            keep_monthly: Some(12),
            keep_yearly: Some(1),
            prefix: Some("auto-".to_string()),
        };

        let state = PrunePolicyState::from_policy(&policy);
        assert_eq!(state.last_str, "5");
        assert_eq!(state.daily_str, "7");
        assert_eq!(state.weekly_str, "4");
        assert_eq!(state.monthly_str, "12");
        assert_eq!(state.yearly_str, "1");
        assert_eq!(state.prefix_str, "auto-");

        let converted_back = state.to_policy();
        assert_eq!(policy, converted_back);
    }
}
