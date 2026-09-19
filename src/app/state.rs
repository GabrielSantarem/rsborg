use crate::borg::{ArchiveFileEntry, BackupProgress, BorgCheckMode, CheckResult, PruneArchiveItem};
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
    DoneRestore(String),
    DoneInspect(Result<Vec<ArchiveFileEntry>, String>, String),
    DonePruneDryRun(Result<Vec<PruneArchiveItem>, String>, PrunePolicy),
    DonePruneExecute(Result<usize, String>),
    DoneCheck(Result<CheckResult, String>, String, String),
    Error(String),
}
