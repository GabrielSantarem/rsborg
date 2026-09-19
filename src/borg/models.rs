use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BackupArchive {
    pub archive: String,
    pub barchive: String,
    pub id: String,
    pub name: String,
    pub start: String,
    pub time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RepositoryInfo {
    pub id: String,
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BorgArchiveList {
    pub archives: Vec<BackupArchive>,
    pub repository: RepositoryInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct ArchiveFileEntry {
    pub path: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub mtime: String,
    #[serde(rename = "type", default)]
    pub entry_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorgCheckMode {
    RepositoryOnly,
    Standard,
    VerifyData,
    Repair,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum DiffChangeItem {
    #[serde(rename = "added")]
    Added {
        #[serde(default)]
        size: u64,
    },
    #[serde(rename = "removed")]
    Removed {
        #[serde(default)]
        size: u64,
    },
    #[serde(rename = "modified")]
    Modified {
        #[serde(default)]
        added: u64,
        #[serde(default)]
        removed: u64,
    },
    #[serde(rename = "mode")]
    Mode {
        #[serde(default)]
        old_mode: Option<String>,
        #[serde(default)]
        new_mode: Option<String>,
    },
    #[serde(rename = "owner")]
    Owner {
        #[serde(default)]
        old_user: Option<String>,
        #[serde(default)]
        new_user: Option<String>,
        #[serde(default)]
        old_group: Option<String>,
        #[serde(default)]
        new_group: Option<String>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Added,
    Removed,
    Modified,
    Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DiffEntry {
    pub path: String,
    #[serde(default)]
    pub changes: Vec<DiffChangeItem>,
}

impl DiffEntry {
    pub fn kind(&self) -> DiffKind {
        for change in &self.changes {
            match change {
                DiffChangeItem::Added { .. } => return DiffKind::Added,
                DiffChangeItem::Removed { .. } => return DiffKind::Removed,
                DiffChangeItem::Modified { .. } => return DiffKind::Modified,
                _ => {}
            }
        }
        DiffKind::Metadata
    }

    pub fn formatted_change(&self) -> String {
        for change in &self.changes {
            match change {
                DiffChangeItem::Added { size } => {
                    return format!("+{}", crate::borg::parser::format_bytes(*size));
                }
                DiffChangeItem::Removed { size } => {
                    return format!("-{}", crate::borg::parser::format_bytes(*size));
                }
                DiffChangeItem::Modified { added, removed } => {
                    return format!(
                        "+{} / -{}",
                        crate::borg::parser::format_bytes(*added),
                        crate::borg::parser::format_bytes(*removed)
                    );
                }
                DiffChangeItem::Mode { old_mode, new_mode } => {
                    let old = old_mode.as_deref().unwrap_or("?");
                    let new = new_mode.as_deref().unwrap_or("?");
                    return format!("mode: {} -> {}", old, new);
                }
                DiffChangeItem::Owner {
                    old_user, new_user, ..
                } => {
                    let old = old_user.as_deref().unwrap_or("?");
                    let new = new_user.as_deref().unwrap_or("?");
                    return format!("owner: {} -> {}", old, new);
                }
                DiffChangeItem::Other => {}
            }
        }
        "-".to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    pub success: bool,
    pub warnings: bool,
    pub log_output: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PruneArchiveItem {
    pub name: String,
    pub date_info: String,
    pub will_keep: bool,
    pub rule_info: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackupProgress {
    pub raw_line: String,
    pub original_size: String,
    pub compressed_size: String,
    pub deduplicated_size: String,
    pub files_count: String,
    pub current_file: String,
}

impl BackupProgress {
    pub fn parse(line: &str) -> Self {
        crate::borg::parser::parse_backup_progress(line)
    }
}
