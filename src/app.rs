use ratatui::widgets::TableState;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crate::borg::{ArchiveFileEntry, BackupArchive, BackupProgress, BorgManager, RepositoryInfo};
use crate::browser::FileBrowser;
use crate::checker;
use crate::config::{self, AppConfig, RepositoryConfig};
use crate::i18n::{Language, Translator};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoadingInfo {
    pub message: String,
    pub elapsed_secs: u64,
    pub original_size: String,
    pub compressed_size: String,
    pub deduplicated_size: String,
    pub files_count: String,
    pub current_file: String,
    pub raw_line: String,
    pub spinner_frame: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectState {
    pub archive_name: String,
    pub entries: Vec<ArchiveFileEntry>,
    pub selected_index: usize,
}

#[derive(Debug, PartialEq)]
pub enum AppState {
    Initializing,
    InitError(String),
    Browsing,
    CreatingBackup,
    Loading,
    ErrorPopup(String),
    ConfirmDelete(String),
    InspectArchive(InspectState),
    ManagingRepos,
    AddingRepo,
}

#[derive(Debug, PartialEq)]
pub enum CreateFocus {
    Name,
    Browser,
}

pub enum ThreadStatus {
    Progress(BackupProgress),
    DoneCreate,
    DoneDelete,
    DoneInspect(Result<Vec<ArchiveFileEntry>, String>, String),
    Error(String),
}

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
                AppState::ErrorPopup("Erro: O Nome do backup não pode ser vazio!".to_string());
            return;
        }

        if name.contains('/') || name.contains(':') {
            self.state = AppState::ErrorPopup(
                "Erro: O Nome do backup não pode conter '/' ou ':' (caracteres reservados)!"
                    .to_string(),
            );
            return;
        }

        let (includes, excludes) = self.file_browser.get_final_paths();

        if includes.is_empty() {
            self.state = AppState::ErrorPopup(
                "Erro: Você precisa selecionar ao menos 1 pasta/arquivo principal com [+] (Aperte Espaço)!"
                    .to_string(),
            );
            return;
        }

        let (repo_path, passphrase) = match self.get_active_repo() {
            Some(r) => (r.location.clone(), r.passphrase.clone()),
            None => (config::get_default_repo_path(), None),
        };

        self.loading_info = LoadingInfo {
            message: format!("Criando arquivo '{}'...", name),
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
        if let Some(i) = self.table_state.selected() {
            if let Some(archive) = self.archives.get(i) {
                self.state = AppState::ConfirmDelete(archive.name.clone());
            }
        }
    }

    pub fn confirm_delete_archive(&mut self, archive_name: String) {
        let (repo_path, passphrase) = match self.get_active_repo() {
            Some(r) => (r.location.clone(), r.passphrase.clone()),
            None => (config::get_default_repo_path(), None),
        };

        self.loading_info = LoadingInfo {
            message: format!(
                "Excluindo backup '{}' e compactando repositório...",
                archive_name
            ),
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
        if let Some(i) = self.table_state.selected() {
            if let Some(archive) = self.archives.get(i) {
                let name = archive.name.clone();
                let (repo_path, passphrase) = match self.get_active_repo() {
                    Some(r) => (r.location.clone(), r.passphrase.clone()),
                    None => (config::get_default_repo_path(), None),
                };

                self.loading_info = LoadingInfo {
                    message: format!("Lendo arquivos do backup '{}'...", name),
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
            self.state = AppState::ErrorPopup("Nome e Localização são obrigatórios!".to_string());
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
        };

        self.config.repositories.push(new_repo.clone());
        self.config.active_repo_id = new_repo.id;
        let _ = config::save_config(&self.config);

        self.new_repo_name.clear();
        self.new_repo_location.clear();
        self.new_repo_passphrase.clear();

        self.load_repository();
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
                    self.new_backup_name.clear();
                    self.file_browser.explicit_includes.clear();
                    self.file_browser.explicit_excludes.clear();
                    self.loading_start = None;
                    self.state = AppState::Browsing;
                }
                ThreadStatus::DoneDelete => {
                    self.load_repository();
                    self.loading_start = None;
                    self.state = AppState::Browsing;
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
    }

    #[test]
    fn test_validate_and_submit_backup_empty_name() {
        let mut app = App::new();
        app.new_backup_name = "   ".to_string();
        app.validate_and_submit_backup();

        match app.state {
            AppState::ErrorPopup(msg) => {
                assert!(msg.contains("não pode ser vazio"));
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
                assert!(msg.contains("caracteres reservados"));
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
                assert!(msg.contains("precisa selecionar"));
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
}
