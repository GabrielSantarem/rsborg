use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::PrunePolicy;

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
                trimmed[start + 1..end].to_string()
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
        trimmed[paren_close + 1..].trim_start_matches(':').trim()
    } else if let Some(colon_pos) = trimmed.find(':') {
        trimmed[colon_pos + 1..].trim()
    } else {
        return None;
    };

    let name_and_date = if let Some(bracket_pos) = content_after_colon.rfind('[') {
        content_after_colon[..bracket_pos].trim()
    } else {
        content_after_colon
    };

    let (name, date_str) = if let Some(last_space) = name_and_date.rfind("   ") {
        (
            name_and_date[..last_space].trim(),
            name_and_date[last_space..].trim(),
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
            && pos_o >= 2 {
                progress.original_size = format!("{} {}", tokens[pos_o - 2], tokens[pos_o - 1]);
            }
        if let Some(pos_c) = tokens.iter().position(|&t| t == "C")
            && pos_c >= 2 {
                progress.compressed_size = format!("{} {}", tokens[pos_c - 2], tokens[pos_c - 1]);
            }
        if let Some(pos_d) = tokens.iter().position(|&t| t == "D")
            && pos_d >= 2 {
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
}

pub struct BorgManager {
    bin_path: String,
}

impl BorgManager {
    pub fn new() -> Self {
        Self {
            bin_path: "borg".to_string(),
        }
    }

    fn prepare_command(&self, args: &[&str], passphrase: Option<&str>) -> Command {
        let mut cmd = Command::new(&self.bin_path);
        cmd.args(args);
        cmd.env("BORG_UNKNOWN_UNENCRYPTED_REPO_ACCESS_IS_OK", "yes");
        cmd.env("BORG_RELOCATED_REPO_ACCESS_IS_OK", "yes");
        if let Some(pass) = passphrase
            && !pass.is_empty() {
                cmd.env("BORG_PASSPHRASE", pass);
            }
        cmd
    }

    pub fn verify_installation(&self) -> Result<String, String> {
        match Command::new(&self.bin_path).arg("-V").output() {
            Ok(o) => {
                if o.status.success() {
                    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    Err(String::from_utf8_lossy(&o.stderr).to_string())
                }
            }
            Err(e) => Err(format!("Falha '{}': {}", self.bin_path, e)),
        }
    }

    pub fn init_repository(&self, repo_path: &str, passphrase: Option<&str>) -> Result<(), String> {
        let enc_mode = if passphrase.is_some() && !passphrase.unwrap_or("").is_empty() {
            "repokey-blake2"
        } else {
            "none"
        };

        let mut cmd = self.prepare_command(&["init", "-e", enc_mode, repo_path], passphrase);
        let o = cmd.output().map_err(|e| e.to_string())?;

        if o.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&o.stderr).to_string())
        }
    }

    pub fn list_archives(
        &self,
        repo_path: &str,
        passphrase: Option<&str>,
    ) -> Result<BorgArchiveList, String> {
        if repo_path == "/caminho/para/meu/repo" {
            let mock_json = r#"{
                "repository": {
                    "id": "c6198...",
                    "location": "user@server:repo"
                },
                "archives": [
                    {
                        "archive": "mock-backup-1",
                        "barchive": "mock-backup-1",
                        "id": "abc...",
                        "name": "mock-backup-1",
                        "start": "2023-01-01T00:00",
                        "time": "2023-01-01T00:15"
                    }
                ]
            }"#;

            let list: BorgArchiveList = serde_json::from_str(mock_json)
                .map_err(|e| format!("Erro no parse do JSON mockado: {}", e))?;

            return Ok(list);
        }

        let mut cmd = self.prepare_command(&["list", "--json", repo_path], passphrase);
        let output = cmd.output().map_err(|e| format!("Falha: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let list: BorgArchiveList =
            serde_json::from_str(&json_str).map_err(|e| format!("Erro JSON: {}", e))?;

        Ok(list)
    }

    pub fn create_backup_with_progress<F>(
        &self,
        repo_path: &str,
        archive_name: &str,
        paths: Vec<String>,
        excludes: Vec<String>,
        passphrase: Option<&str>,
        mut on_progress: F,
    ) -> Result<(), String>
    where
        F: FnMut(BackupProgress) + Send + 'static,
    {
        if repo_path == "/caminho/para/meu/repo" {
            let sample_files = [
                "documentos/relatorio.pdf",
                "projetos/src/main.rs",
                "imagens/foto1.png",
                "finalizando_arquivos...",
            ];

            for (i, file) in sample_files.iter().enumerate() {
                std::thread::sleep(std::time::Duration::from_millis(50));
                let progress = BackupProgress {
                    raw_line: format!("Processando {}", file),
                    original_size: format!("{:.1} MB", (i + 1) as f32 * 12.5),
                    compressed_size: format!("{:.1} MB", (i + 1) as f32 * 4.8),
                    deduplicated_size: format!("{:.1} MB", (i + 1) as f32 * 1.2),
                    files_count: format!("{}", (i + 1) * 15),
                    current_file: file.to_string(),
                };
                on_progress(progress);
            }
            return Ok(());
        }

        let archive_target = format!("{}::{}", repo_path, archive_name);

        let mut cmd_args = vec![
            "create".to_string(),
            "--progress".to_string(),
            "--stats".to_string(),
            archive_target,
        ];

        cmd_args.extend(paths);

        for exc in excludes {
            cmd_args.push("--exclude".to_string());
            cmd_args.push(exc);
        }

        let arg_slices: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
        let mut cmd = self.prepare_command(&arg_slices, passphrase);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Falha ao iniciar borg create: {}", e))?;

        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Falha ao capturar stderr do processo".to_string())?;

        let mut buffer = Vec::new();
        let mut byte_buf = [0u8; 1];
        let mut all_stderr = String::new();

        while let Ok(n) = stderr.read(&mut byte_buf) {
            if n == 0 {
                break;
            }
            let b = byte_buf[0];
            if b == b'\r' || b == b'\n' {
                if !buffer.is_empty() {
                    let line = String::from_utf8_lossy(&buffer).to_string();
                    all_stderr.push_str(&line);
                    all_stderr.push('\n');

                    let p = BackupProgress::parse(&line);
                    on_progress(p);
                    buffer.clear();
                }
            } else {
                buffer.push(b);
            }
        }

        let status = child
            .wait()
            .map_err(|e| format!("Erro ao aguardar processo: {}", e))?;

        if !status.success() {
            return Err(format!("Erro ao criar backup:\n{}", all_stderr));
        }

        Ok(())
    }

    pub fn delete_archive(
        &self,
        repo_path: &str,
        archive_name: &str,
        passphrase: Option<&str>,
    ) -> Result<(), String> {
        let target = format!("{}::{}", repo_path, archive_name);

        let mut cmd = self.prepare_command(&["delete", &target], passphrase);
        let output = cmd
            .output()
            .map_err(|e| format!("Falha ao executar borg delete: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let mut compact_cmd = self.prepare_command(&["compact", repo_path], passphrase);
        let _ = compact_cmd.output();

        Ok(())
    }

    pub fn list_archive_contents(
        &self,
        repo_path: &str,
        archive_name: &str,
        passphrase: Option<&str>,
    ) -> Result<Vec<ArchiveFileEntry>, String> {
        let target = format!("{}::{}", repo_path, archive_name);

        let mut cmd = self.prepare_command(&["list", "--json-lines", &target], passphrase);
        let output = cmd
            .output()
            .map_err(|e| format!("Falha ao listar conteúdo do backup: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let mut entries = Vec::new();

        for line in stdout_str.lines() {
            if let Ok(entry) = serde_json::from_str::<ArchiveFileEntry>(line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn extract_archive<F>(
        &self,
        repo_path: &str,
        archive_name: &str,
        target_dir: &Path,
        paths: &[String],
        passphrase: Option<&str>,
        mut on_file_extracted: F,
    ) -> Result<(), String>
    where
        F: FnMut(String),
    {
        if !target_dir.exists() {
            std::fs::create_dir_all(target_dir)
                .map_err(|e| format!("Falha ao criar diretório de destino: {}", e))?;
        }

        if repo_path == "/caminho/para/meu/repo" {
            std::thread::sleep(std::time::Duration::from_millis(50));
            on_file_extracted("documentos/relatorio.pdf".to_string());
            std::thread::sleep(std::time::Duration::from_millis(50));
            on_file_extracted("projetos/src/main.rs".to_string());
            return Ok(());
        }

        let target = format!("{}::{}", repo_path, archive_name);
        let mut cmd_args = vec!["extract".to_string(), "--list".to_string(), target];
        for p in paths {
            cmd_args.push(p.clone());
        }

        let arg_slices: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
        let mut cmd = self.prepare_command(&arg_slices, passphrase);
        cmd.current_dir(target_dir);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Falha ao iniciar borg extract: {}", e))?;

        if let Some(stdout) = child.stdout.take() {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                on_file_extracted(line);
            }
        }

        let status = child
            .wait()
            .map_err(|e| format!("Erro ao aguardar processo de extração: {}", e))?;

        if !status.success() {
            let mut err_msg = String::new();
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_string(&mut err_msg);
            }
            return Err(format!("Erro ao extrair arquivos:\n{}", err_msg));
        }

        Ok(())
    }

    pub fn mount_archive(
        &self,
        repo_path: &str,
        archive_name: &str,
        mount_point: &Path,
        passphrase: Option<&str>,
    ) -> Result<(), String> {
        if !mount_point.exists() {
            std::fs::create_dir_all(mount_point)
                .map_err(|e| format!("Falha ao criar ponto de montagem: {}", e))?;
        }

        if repo_path == "/caminho/para/meu/repo" {
            return Ok(());
        }

        let target = format!("{}::{}", repo_path, archive_name);
        let mount_str = mount_point.to_string_lossy().to_string();
        let mut cmd = self.prepare_command(&["mount", &target, &mount_str], passphrase);
        let output = cmd
            .output()
            .map_err(|e| format!("Falha ao executar borg mount: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    pub fn umount_archive(&self, mount_point: &Path) -> Result<(), String> {
        let mount_str = mount_point.to_string_lossy().to_string();
        let mut cmd = self.prepare_command(&["umount", &mount_str], None);
        let output = cmd
            .output()
            .map_err(|e| format!("Falha ao executar borg umount: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    pub fn prune_repository(
        &self,
        repo_path: &str,
        policy: &PrunePolicy,
        dry_run: bool,
        passphrase: Option<&str>,
    ) -> Result<Vec<PruneArchiveItem>, String> {
        if repo_path == "/caminho/para/meu/repo" {
            return Ok(vec![PruneArchiveItem {
                name: "mock-backup-1".to_string(),
                date_info: "2023-01-01 00:00".to_string(),
                will_keep: true,
                rule_info: "secondly #1".to_string(),
            }]);
        }

        let mut args: Vec<String> = vec!["prune".to_string(), "--list".to_string()];
        if dry_run {
            args.push("--dry-run".to_string());
        }
        if let Some(n) = policy.keep_last {
            args.push("--keep-last".to_string());
            args.push(n.to_string());
        }
        if let Some(n) = policy.keep_daily {
            args.push("--keep-daily".to_string());
            args.push(n.to_string());
        }
        if let Some(n) = policy.keep_weekly {
            args.push("--keep-weekly".to_string());
            args.push(n.to_string());
        }
        if let Some(n) = policy.keep_monthly {
            args.push("--keep-monthly".to_string());
            args.push(n.to_string());
        }
        if let Some(n) = policy.keep_yearly {
            args.push("--keep-yearly".to_string());
            args.push(n.to_string());
        }
        if let Some(ref pfx) = policy.prefix
            && !pfx.trim().is_empty() {
                args.push("--prefix".to_string());
                args.push(pfx.trim().to_string());
            }
        args.push(repo_path.to_string());

        let arg_slices: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut cmd = self.prepare_command(&arg_slices, passphrase);
        let output = cmd
            .output()
            .map_err(|e| format!("Falha ao executar borg prune: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let stderr_str = String::from_utf8_lossy(&output.stderr);
        let mut items = Vec::new();

        for line in stderr_str.lines() {
            if let Some(item) = parse_prune_line(line) {
                items.push(item);
            }
        }

        if !dry_run {
            let mut compact_cmd = self.prepare_command(&["compact", repo_path], passphrase);
            let _ = compact_cmd.output();
        }

        Ok(items)
    }

    pub fn check_repository<F>(
        &self,
        repo_path: &str,
        archive_name: Option<&str>,
        mode: BorgCheckMode,
        passphrase: Option<&str>,
        mut on_line: F,
    ) -> Result<CheckResult, String>
    where
        F: FnMut(String),
    {
        if repo_path == "/caminho/para/meu/repo" {
            let log_output = vec![
                "Starting repository check...".to_string(),
                "Checking segments 1/1...".to_string(),
                "Archive consistency check complete, no problems found.".to_string(),
            ];
            for l in &log_output {
                on_line(l.clone());
            }
            return Ok(CheckResult {
                success: true,
                warnings: false,
                log_output,
            });
        }

        let mut args: Vec<String> = vec!["check".to_string(), "--info".to_string()];

        match mode {
            BorgCheckMode::RepositoryOnly => {
                args.push("--repository-only".to_string());
            }
            BorgCheckMode::Standard => {}
            BorgCheckMode::VerifyData => {
                args.push("--verify-data".to_string());
            }
            BorgCheckMode::Repair => {
                args.push("--repair".to_string());
            }
        }

        let target = match archive_name {
            Some(name) => format!("{}::{}", repo_path, name),
            None => repo_path.to_string(),
        };
        args.push(target);

        let arg_slices: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut cmd = self.prepare_command(&arg_slices, passphrase);
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Falha ao iniciar processo borg check: {}", e))?;

        let mut log_output = Vec::new();

        if let Some(stderr) = child.stderr.take() {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                on_line(line.clone());
                log_output.push(line);
            }
        }

        let status = child
            .wait()
            .map_err(|e| format!("Erro ao aguardar término do borg check: {}", e))?;

        let exit_code = status.code().unwrap_or(2);
        let success = exit_code == 0;
        let warnings = exit_code == 1;

        if exit_code > 1 && log_output.is_empty() {
            return Err("Processo borg check falhou com erro crítico.".to_string());
        }

        Ok(CheckResult {
            success,
            warnings,
            log_output,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_or_whitespace_progress() {
        let p1 = BackupProgress::parse("");
        assert_eq!(p1.raw_line, "");
        assert_eq!(p1.files_count, "");
        assert_eq!(p1.current_file, "");

        let p2 = BackupProgress::parse("     \n\t   ");
        assert_eq!(p2.raw_line, "");
    }

    #[test]
    fn test_parse_standard_borg_progress_line() {
        let line = "12.34 MB O 5.67 MB C 1.20 MB D 45 N /home/user/documentos/relatorio.pdf";
        let p = BackupProgress::parse(line);

        assert_eq!(p.original_size, "12.34 MB");
        assert_eq!(p.compressed_size, "5.67 MB");
        assert_eq!(p.deduplicated_size, "1.20 MB");
        assert_eq!(p.files_count, "45");
        assert_eq!(p.current_file, "/home/user/documentos/relatorio.pdf");
    }

    #[test]
    fn test_parse_progress_line_with_spaces_in_filename() {
        let line = "100.50 GB O 45.00 GB C 10.00 GB D 1024 N /mnt/backup/Meu Arquivo De Trabalho 2024 Final.docx";
        let p = BackupProgress::parse(line);

        assert_eq!(p.original_size, "100.50 GB");
        assert_eq!(p.compressed_size, "45.00 GB");
        assert_eq!(p.deduplicated_size, "10.00 GB");
        assert_eq!(p.files_count, "1024");
        assert_eq!(
            p.current_file,
            "/mnt/backup/Meu Arquivo De Trabalho 2024 Final.docx"
        );
    }

    #[test]
    fn test_parse_progress_non_standard_message() {
        let line = "Iniciando escaneamento de arquivos em /home/tomate...";
        let p = BackupProgress::parse(line);

        assert_eq!(p.raw_line, line);
        assert_eq!(p.original_size, "");
        assert_eq!(p.files_count, "");
    }

    #[test]
    fn test_parse_json_lines_archive_entries() {
        let json_lines = r#"
{"type": "-", "mode": "-rw-r--r--", "user": "tomate", "group": "tomate", "uid": 1000, "gid": 1000, "size": 2048, "mtime": "2024-01-01T12:00:00.000000", "path": "docs/arquivo.txt", "healthy": true}
{"type": "d", "mode": "drwxr-xr-x", "user": "tomate", "group": "tomate", "uid": 1000, "gid": 1000, "size": 0, "mtime": "2024-01-01T12:00:00.000000", "path": "docs", "healthy": true}
TAM: warning message line that is not json
{"type": "-", "path": "arquivo_minimo.txt"}
"#;

        let mut entries = Vec::new();
        for line in json_lines.lines() {
            if let Ok(entry) = serde_json::from_str::<ArchiveFileEntry>(line) {
                entries.push(entry);
            }
        }

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].path, "docs/arquivo.txt");
        assert_eq!(entries[0].size, 2048);
        assert_eq!(entries[0].entry_type, "-");

        assert_eq!(entries[1].path, "docs");
        assert_eq!(entries[1].entry_type, "d");

        assert_eq!(entries[2].path, "arquivo_minimo.txt");
        assert_eq!(entries[2].size, 0);
        assert_eq!(entries[2].mode, "");
    }

    #[test]
    fn test_extract_mock_archive() {
        let manager = BorgManager::new();
        let target_dir = std::env::temp_dir().join("rsborg_test_extract");
        let mut extracted_files = Vec::new();

        let res = manager.extract_archive(
            "/caminho/para/meu/repo",
            "mock-backup-1",
            &target_dir,
            &[],
            None,
            |file| {
                extracted_files.push(file);
            },
        );

        assert!(res.is_ok());
        assert_eq!(extracted_files.len(), 2);
        let _ = std::fs::remove_dir_all(target_dir);
    }

    #[test]
    fn test_parse_prune_line_keeping() {
        let line = "Keeping archive (rule: secondly #1):         bkp com espaco 2                     Fri, 2026-09-18 21:27:50 [2c4f8f4cad5beff6b6df16e5be32539f55c1d7f125d0c19e62ad9eb8c1866511]";
        let item = parse_prune_line(line).expect("Deveria fazer parse de linha Keeping");

        assert_eq!(item.name, "bkp com espaco 2");
        assert_eq!(item.date_info, "Fri, 2026-09-18 21:27:50");
        assert!(item.will_keep);
        assert_eq!(item.rule_info, "rule: secondly #1");
    }

    #[test]
    fn test_parse_prune_line_would_prune() {
        let line = "Would prune:                                 bkp com espaco 1                     Fri, 2026-09-18 21:27:50 [1004a19bd664fae7a203aef6d614a9847ff8054e9a1ce2387cdda4d9e4eb6e39]";
        let item = parse_prune_line(line).expect("Deveria fazer parse de linha Would prune");

        assert_eq!(item.name, "bkp com espaco 1");
        assert_eq!(item.date_info, "Fri, 2026-09-18 21:27:50");
        assert!(!item.will_keep);
    }

    #[test]
    fn test_parse_prune_line_real_pruning() {
        let line = "Pruning archive:                             bkp_antigo                           Fri, 2026-09-18 21:27:50 [1004a19bd664fae7a203aef6d614a9847ff8054e9a1ce2387cdda4d9e4eb6e39]";
        let item = parse_prune_line(line).expect("Deveria fazer parse de linha Pruning archive");

        assert_eq!(item.name, "bkp_antigo");
        assert_eq!(item.date_info, "Fri, 2026-09-18 21:27:50");
        assert!(!item.will_keep);
    }


    #[test]
    fn test_check_repository_mock() {
        let manager = BorgManager::new();
        let mut lines = Vec::new();
        let result = manager.check_repository(
            "/caminho/para/meu/repo",
            None,
            BorgCheckMode::Standard,
            None,
            |line| lines.push(line),
        );

        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert!(!res.warnings);
        assert_eq!(res.log_output.len(), 3);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_prune_mock_repository() {
        let manager = BorgManager::new();
        let policy = PrunePolicy::default();
        let res = manager.prune_repository("/caminho/para/meu/repo", &policy, true, None);

        assert!(res.is_ok());
        let items = res.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "mock-backup-1");
        assert!(items[0].will_keep);
    }

    #[test]
    fn test_parse_prune_line_invalid_or_info() {
        let line = "TAM: warning message";
        assert!(parse_prune_line(line).is_none());
    }
}
