use crate::borg::{
    ArchiveFileEntry, BackupProgress, BorgCheckMode, CheckResult, DiffEntry, PruneArchiveItem,
};
use crate::config::PrunePolicy;

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

#[derive(Debug, Clone, PartialEq)]
pub struct RestoreRequest {
    pub archive_name: String,
    pub destination_path: String,
    pub paths_to_extract: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrunePolicyState {
    pub focus_field: usize,
    pub last_str: String,
    pub daily_str: String,
    pub weekly_str: String,
    pub monthly_str: String,
    pub yearly_str: String,
    pub prefix_str: String,
}

impl PrunePolicyState {
    pub fn from_policy(p: &PrunePolicy) -> Self {
        Self {
            focus_field: 1, // default focus: keep_daily
            last_str: p.keep_last.map(|v| v.to_string()).unwrap_or_default(),
            daily_str: p.keep_daily.map(|v| v.to_string()).unwrap_or_default(),
            weekly_str: p.keep_weekly.map(|v| v.to_string()).unwrap_or_default(),
            monthly_str: p.keep_monthly.map(|v| v.to_string()).unwrap_or_default(),
            yearly_str: p.keep_yearly.map(|v| v.to_string()).unwrap_or_default(),
            prefix_str: p.prefix.clone().unwrap_or_default(),
        }
    }

    pub fn to_policy(&self) -> PrunePolicy {
        PrunePolicy {
            keep_last: self.last_str.trim().parse().ok(),
            keep_daily: self.daily_str.trim().parse().ok(),
            keep_weekly: self.weekly_str.trim().parse().ok(),
            keep_monthly: self.monthly_str.trim().parse().ok(),
            keep_yearly: self.yearly_str.trim().parse().ok(),
            prefix: if self.prefix_str.trim().is_empty() {
                None
            } else {
                Some(self.prefix_str.trim().to_string())
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrunePlanState {
    pub items: Vec<PruneArchiveItem>,
    pub selected_index: usize,
    pub policy: PrunePolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckWizardState {
    pub target_archive: Option<String>,
    pub check_mode: BorgCheckMode,
    pub check_archive_only: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckResultState {
    pub result: CheckResult,
    pub target_display: String,
    pub mode_display: String,
    pub log_scroll: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffWizardState {
    pub base_archive: String,
    pub candidates: Vec<String>,
    pub selected_candidate_idx: usize,
    pub content_only: bool,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileFocus {
    Name,
    Compression,
    Schedule,
    Browser,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileWizardState {
    pub focus: ProfileFocus,
    pub name: String,
    pub compression_idx: usize,
    pub schedule_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutomationViewState {
    pub profile: crate::config::BackupProfile,
    pub active_tab: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffViewState {
    pub archive1: String,
    pub archive2: String,
    pub entries: Vec<DiffEntry>,
    pub selected_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFilterLevel {
    All,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogViewerState {
    pub all_lines: Vec<String>,
    pub filter: LogFilterLevel,
    pub scroll: usize,
}

impl LogViewerState {
    pub fn new(lines: Vec<String>) -> Self {
        let count = lines.len();
        Self {
            all_lines: lines,
            filter: LogFilterLevel::All,
            scroll: count.saturating_sub(20),
        }
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        self.all_lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let matches = match self.filter {
                    LogFilterLevel::All => true,
                    LogFilterLevel::Info => line.contains("[INFO]"),
                    LogFilterLevel::Warn => line.contains("[WARN]"),
                    LogFilterLevel::Error => line.contains("[ERROR]"),
                };
                if matches {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn set_filter(&mut self, filter: LogFilterLevel) {
        self.filter = filter;
        let count = self.filtered_indices().len();
        self.scroll = count.saturating_sub(20);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, max_lines: usize) {
        let total = self.filtered_indices().len();
        if self.scroll + max_lines < total {
            self.scroll += 1;
        }
    }

    pub fn scroll_page_up(&mut self, page: usize) {
        self.scroll = self.scroll.saturating_sub(page);
    }

    pub fn scroll_page_down(&mut self, page: usize, max_lines: usize) {
        let total = self.filtered_indices().len();
        if self.scroll + max_lines + page <= total {
            self.scroll += page;
        } else if self.scroll + max_lines < total {
            self.scroll = total.saturating_sub(max_lines);
        }
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_bottom(&mut self, max_lines: usize) {
        let total = self.filtered_indices().len();
        self.scroll = total.saturating_sub(max_lines);
    }
}

#[derive(Debug, PartialEq)]
pub enum AppState {
    Initializing,
    InitError(String),
    Browsing,
    CreatingBackup,
    Loading,
    ErrorPopup(String),
    SuccessPopup(String),
    ConfirmDelete(String),
    ConfirmRestore(RestoreRequest),
    InspectArchive(InspectState),
    ManagingRepos,
    AddingRepo,
    PruningPolicy(PrunePolicyState),
    PrunePlanView(PrunePlanState),
    CheckWizard(CheckWizardState),
    CheckResultView(CheckResultState),
    DiffWizard(DiffWizardState),
    DiffView(DiffViewState),
    ManagingProfiles { selected_index: usize },
    CreatingProfile(ProfileWizardState),
    AutomationView(AutomationViewState),
    HelpModal,
    LogViewer(LogViewerState),
    Settings(SettingsState),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SettingsState {
    pub selected_index: usize,
}

#[derive(Debug, PartialEq)]
pub enum CreateFocus {
    Name,
    Browser,
}

pub enum ThreadStatus {
    Progress(BackupProgress),
    DoneCreate,
    DoneDelete(String),
    DoneRestore(String),
    DoneInspect(Result<Vec<ArchiveFileEntry>, String>, String),
    DonePruneDryRun(Result<Vec<PruneArchiveItem>, String>, PrunePolicy),
    DonePruneExecute(Result<usize, String>),
    DoneCheck(Result<CheckResult, String>, String, String),
    DoneDiff(Result<Vec<DiffEntry>, String>, String, String),
    Error(String),
}
