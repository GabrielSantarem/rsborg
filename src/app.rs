use ratatui::widgets::TableState;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crate::borg::{BackupArchive, BackupProgress, BorgManager, RepositoryInfo};
use crate::browser::FileBrowser;
use crate::checker;
use crate::i18n::{Language, Translator};

#[derive(Clone, PartialEq, Default)]
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

#[derive(PartialEq)]
pub enum AppState {
    Initializing,
    InitError(String),
    Browsing,
    CreatingBackup,
    Loading,
    ErrorPopup(String),
}

#[derive(PartialEq)]
pub enum CreateFocus {
    Name,
    Browser,
}

pub enum ThreadStatus {
    Progress(BackupProgress),
    DoneCreate,
    Error(String),
}

pub struct App {
    pub state: AppState,
    pub borg_manager: BorgManager,
    pub borg_version: Option<String>,
    pub repo_path: String,
    pub repository: Option<RepositoryInfo>,
    pub archives: Vec<BackupArchive>,
    pub table_state: TableState,
    pub should_quit: bool,
    pub t: Translator,

    pub create_focus: CreateFocus,
    pub new_backup_name: String,
    pub file_browser: FileBrowser,

    pub loading_info: LoadingInfo,
    pub loading_start: Option<Instant>,

    pub tx: Sender<ThreadStatus>,
    pub rx: Receiver<ThreadStatus>,
}

impl App {
    pub fn new(repo_path: String) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            state: AppState::Initializing,
            borg_manager: BorgManager::new(),
            borg_version: None,
            repo_path,
            repository: None,
            archives: Vec::new(),
            table_state: TableState::default(),
            should_quit: false,
            t: Translator::new(Language::Pt),

            create_focus: CreateFocus::Name,
            new_backup_name: String::new(),
            file_browser: FileBrowser::new(),

            loading_info: LoadingInfo::default(),
            loading_start: None,

            tx,
            rx,
        }
    }

    pub fn toggle_language(&mut self) {
        let new_lang = match self.t.lang {
            Language::Pt => Language::En,
            Language::En => Language::Pt,
        };
        self.t = Translator::new(new_lang);
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
        match self.borg_manager.list_archives(&self.repo_path) {
            Ok(data) => {
                self.repository = Some(data.repository);
                self.archives = data.archives;
                if !self.archives.is_empty() {
                    self.table_state.select(Some(0));
                }
                self.state = AppState::Browsing;
            }
            Err(e) => {
                if self.repo_path.contains(".rsborg/backups") {
                    let _ = self.borg_manager.init_repository(&self.repo_path);
                    if let Ok(data) = self.borg_manager.list_archives(&self.repo_path) {
                        self.repository = Some(data.repository);
                        self.archives = data.archives;
                        if !self.archives.is_empty() {
                            self.table_state.select(Some(0));
                        }
                        self.state = AppState::Browsing;
                        return;
                    }
                }
                self.state = AppState::InitError(format!("Repositório inacessível: {}", e));
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

        let (includes, excludes) = self.file_browser.get_final_paths();

        if includes.is_empty() {
            self.state = AppState::ErrorPopup(
                "Erro: Você precisa selecionar ao menos 1 pasta/arquivo principal com [+] (Aperte Espaço)!"
                    .to_string(),
            );
            return;
        }

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
        let repo = self.repo_path.clone();

        thread::spawn(move || {
            let tx_prog = tx.clone();
            let res =
                manager.create_backup_with_progress(&repo, &name, includes, excludes, move |p| {
                    let _ = tx_prog.send(ThreadStatus::Progress(p));
                });

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

    pub fn handle_background_tasks(&mut self) {
        // Atualiza o tempo decorrido e a animação do spinner enquanto estiver carregando
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
