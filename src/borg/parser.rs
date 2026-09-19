use crate::borg::models::{BackupProgress, PruneArchiveItem};

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} kB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn parse_prune_line(line: &str) -> Option<PruneArchiveItem> {
    let trimmed = line.trim();
    let will_keep = if trimmed.starts_with("Keeping archive") {
        true
    } else if trimmed.starts_with("Would prune:") || trimmed.starts_with("Pruning archive:") {
        false
    } else {
        return None;
    };

    let rule_info = if will_keep {
        if let Some(start) = trimmed.find('(') {
            if let Some(end) = trimmed.find(')') {
                if start < end {
                    trimmed.get(start + 1..end).unwrap_or("Manter").to_string()
                } else {
                    "Manter".to_string()
                }
            } else {
                "Manter".to_string()
            }
        } else {
            "Manter".to_string()
        }
    } else {
        "Excluir (Fora da política)".to_string()
    };

    let content_after_colon = if let Some(paren_close) = trimmed.find(')') {
        trimmed
            .get(paren_close + 1..)
            .map(|s| s.trim_start_matches(':').trim())
            .unwrap_or("")
    } else if let Some(colon_pos) = trimmed.find(':') {
        trimmed.get(colon_pos + 1..).map(|s| s.trim()).unwrap_or("")
    } else {
        return None;
    };

    let name_and_date = if let Some(bracket_pos) = content_after_colon.rfind('[') {
        content_after_colon
            .get(..bracket_pos)
            .map(|s| s.trim())
            .unwrap_or(content_after_colon)
    } else {
        content_after_colon
    };

    let (name, date_str) = if let Some(last_space) = name_and_date.rfind("   ") {
        (
            name_and_date
                .get(..last_space)
                .map(|s| s.trim())
                .unwrap_or(name_and_date),
            name_and_date
                .get(last_space..)
                .map(|s| s.trim())
                .unwrap_or(""),
        )
    } else {
        (name_and_date, "")
    };

    Some(PruneArchiveItem {
        name: name.to_string(),
        date_info: date_str.to_string(),
        will_keep,
        rule_info,
    })
}

pub fn parse_backup_progress(line: &str) -> BackupProgress {
    let trimmed = line.trim();
    let mut progress = BackupProgress {
        raw_line: trimmed.to_string(),
        ..Default::default()
    };

    if trimmed.is_empty() {
        return progress;
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if let Some(pos_o) = tokens.iter().position(|&t| t == "O")
        && pos_o >= 2
    {
        progress.original_size = format!("{} {}", tokens[pos_o - 2], tokens[pos_o - 1]);
    }
    if let Some(pos_c) = tokens.iter().position(|&t| t == "C")
        && pos_c >= 2
    {
        progress.compressed_size = format!("{} {}", tokens[pos_c - 2], tokens[pos_c - 1]);
    }
    if let Some(pos_d) = tokens.iter().position(|&t| t == "D")
        && pos_d >= 2
    {
        progress.deduplicated_size = format!("{} {}", tokens[pos_d - 2], tokens[pos_d - 1]);
    }
    if let Some(pos_n) = tokens.iter().position(|&t| t == "N") {
        if pos_n >= 1 {
            progress.files_count = tokens[pos_n - 1].to_string();
        }
        if pos_n + 1 < tokens.len() {
            progress.current_file = tokens[pos_n + 1..].join(" ");
        }
    }

    progress
}
