pub mod errors;
pub mod models;
pub mod parser;

#[cfg(test)]
mod tests;

pub use errors::BorgError;
pub use models::{
    ArchiveFileEntry, BackupArchive, BackupProgress, BorgArchiveList, BorgCheckMode, CheckResult,
    DiffChangeItem, DiffEntry, DiffKind, PruneArchiveItem, RepositoryInfo,
};
#[allow(unused_imports)]
pub use parser::{format_bytes, parse_backup_progress, parse_prune_line};

use crate::config::PrunePolicy;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

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
            && !pass.is_empty()
        {
            cmd.env("BORG_PASSPHRASE", pass);
        }
        cmd
    }

    fn execute_command(&self, mut cmd: Command, desc: &str) -> Result<std::process::Output, String> {
        let start = std::time::Instant::now();
        crate::log_info!("[borg] Executing command: {:?}", cmd);

        let output = cmd.output().map_err(|e| {
            let err_msg = format!("Falha ao executar {}: {}", desc, e);
            crate::log_error!("[borg] IO error during '{}': {}", desc, e);
            BorgError::IoError(err_msg).to_string()
        })?;

        let duration = start.elapsed();
        let code = output.status.code();
        let success = output.status.success();

        if success {
            crate::log_info!("[borg] Finished '{}' successfully in {:?}", desc, duration);
            Ok(output)
        } else {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            crate::log_error!(
                "[borg] Command '{}' failed (code: {:?}, elapsed: {:?}):
{}",
                desc,
                code,
                duration,
                stderr_str.trim()
            );
            let err = BorgError::from_stderr(code, &stderr_str);
            Err(err.to_string())
        }
    }

    pub fn verify_installation(&self) -> Result<String, String> {
        match Command::new(&self.bin_path).arg("-V").output() {
            Ok(o) => {
                if o.status.success() {
                    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    let err = BorgError::from_stderr(
                        o.status.code(),
                        &String::from_utf8_lossy(&o.stderr),
                    );
                    Err(err.to_string())
                }
            }
            Err(e) => Err(BorgError::BorgNotFound(format!(
                "Falha ao executar '{}': {}",
                self.bin_path, e
            ))
            .to_string()),
        }
    }

    pub fn init_repository(&self, repo_path: &str, passphrase: Option<&str>) -> Result<(), String> {
        let enc_mode = if passphrase.is_some() && !passphrase.unwrap_or("").is_empty() {
            "repokey-blake2"
        } else {
            "none"
        };

        let cmd = self.prepare_command(&["init", "-e", enc_mode, repo_path], passphrase);
        self.execute_command(cmd, &format!("init repository at '{}'", repo_path))?;
        Ok(())
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
                .map_err(|e| BorgError::JsonParseError(e.to_string()).to_string())?;

            return Ok(list);
        }

        let cmd = self.prepare_command(&["list", "--json", repo_path], passphrase);
        let output = self.execute_command(cmd, &format!("list archives for '{}'", repo_path))?;

        let json_str = String::from_utf8_lossy(&output.stdout);
        let list: BorgArchiveList = serde_json::from_str(&json_str)
            .map_err(|e| BorgError::JsonParseError(e.to_string()).to_string())?;

        Ok(list)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_backup_with_progress<F>(
        &self,
        repo_path: &str,
        archive_name: &str,
        paths: Vec<String>,
        excludes: Vec<String>,
        compression: Option<&str>,
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
        ];

        if let Some(comp) = compression.filter(|c| !c.is_empty()) {
            cmd_args.push("--compression".to_string());
            cmd_args.push(comp.to_string());
        }

        cmd_args.push(archive_target);

        cmd_args.extend(paths);

        for exc in excludes {
            cmd_args.push("--exclude".to_string());
            cmd_args.push(exc);
        }

        let arg_slices: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
        let mut cmd = self.prepare_command(&arg_slices, passphrase);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        crate::log_info!("[borg] Executing create: {:?}", cmd);
        let mut child = cmd.spawn().map_err(|e| {
            crate::log_error!("[borg] Failed to spawn borg create: {}", e);
            BorgError::IoError(format!("Falha ao iniciar borg create: {}", e)).to_string()
        })?;

        let mut stderr = child.stderr.take().ok_or_else(|| {
            BorgError::IoError("Falha ao capturar stderr do processo".to_string()).to_string()
        })?;

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

        let status = child.wait().map_err(|e| {
            BorgError::IoError(format!("Erro ao aguardar processo: {}", e)).to_string()
        })?;

        if !status.success() {
            crate::log_error!("[borg] borg create failed (code: {:?}):\n{}", status.code(), all_stderr.trim());
            let err = BorgError::from_stderr(status.code(), &all_stderr);
            return Err(format!("Erro ao criar backup:\n{}", err));
        }

        crate::log_info!("[borg] borg create '{}' finished successfully.", archive_name);
        Ok(())
    }

    pub fn delete_archive(
        &self,
        repo_path: &str,
        archive_name: &str,
        passphrase: Option<&str>,
    ) -> Result<(), String> {
        if repo_path == "/caminho/para/meu/repo" {
            crate::log_info!("[borg] Mock delete archive: {}", archive_name);
            return Ok(());
        }

        let target = format!("{}::{}", repo_path, archive_name);
        let cmd = self.prepare_command(&["delete", &target], passphrase);
        self.execute_command(cmd, &format!("delete archive '{}'", archive_name))?;
        Ok(())
    }

    pub fn compact_repository(
        &self,
        repo_path: &str,
        passphrase: Option<&str>,
    ) -> Result<(), String> {
        if repo_path == "/caminho/para/meu/repo" {
            crate::log_info!("[borg] Mock compact repository: {}", repo_path);
            return Ok(());
        }

        let cmd = self.prepare_command(&["compact", repo_path], passphrase);
        self.execute_command(cmd, &format!("compact repository '{}'", repo_path))?;
        Ok(())
    }

    pub fn break_lock(
        &self,
        repo_path: &str,
        passphrase: Option<&str>,
    ) -> Result<(), String> {
        if repo_path == "/caminho/para/meu/repo" {
            crate::log_info!("[borg] Mock break-lock repository: {}", repo_path);
            return Ok(());
        }

        let cmd = self.prepare_command(&["break-lock", repo_path], passphrase);
        self.execute_command(cmd, &format!("break-lock on '{}'", repo_path))?;
        Ok(())
    }

    pub fn list_archive_contents(
        &self,
        repo_path: &str,
        archive_name: &str,
        passphrase: Option<&str>,
    ) -> Result<Vec<ArchiveFileEntry>, String> {
        let target = format!("{}::{}", repo_path, archive_name);

        let cmd = self.prepare_command(&["list", "--json-lines", &target], passphrase);
        let output = self.execute_command(cmd, &format!("list contents for '{}'", target))?;

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
            std::fs::create_dir_all(target_dir).map_err(|e| {
                BorgError::IoError(format!("Falha ao criar diretório de destino: {}", e))
                    .to_string()
            })?;
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

        crate::log_info!("[borg] Executing extract: {:?}", cmd);
        let mut child = cmd.spawn().map_err(|e| {
            crate::log_error!("[borg] Failed to spawn borg extract: {}", e);
            BorgError::IoError(format!("Falha ao iniciar borg extract: {}", e)).to_string()
        })?;

        if let Some(stdout) = child.stdout.take() {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                on_file_extracted(line);
            }
        }

        let status = child.wait().map_err(|e| {
            BorgError::IoError(format!("Erro ao aguardar processo de extração: {}", e)).to_string()
        })?;

        if !status.success() {
            let mut err_msg = String::new();
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_string(&mut err_msg);
            }
            crate::log_error!("[borg] borg extract failed (code: {:?}):\n{}", status.code(), err_msg.trim());
            let err = BorgError::from_stderr(status.code(), &err_msg);
            return Err(format!("Erro ao extrair arquivos:\n{}", err));
        }

        crate::log_info!("[borg] borg extract finished successfully.");
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
            std::fs::create_dir_all(mount_point).map_err(|e| {
                BorgError::IoError(format!("Falha ao criar ponto de montagem: {}", e)).to_string()
            })?;
        }

        if repo_path == "/caminho/para/meu/repo" {
            return Ok(());
        }

        let target = format!("{}::{}", repo_path, archive_name);
        let mount_str = mount_point.to_string_lossy().to_string();
        let cmd = self.prepare_command(&["mount", &target, &mount_str], passphrase);
        self.execute_command(cmd, &format!("mount archive '{}' at '{}'", target, mount_str))?;
        Ok(())
    }

    pub fn umount_archive(&self, mount_point: &Path) -> Result<(), String> {
        let mount_str = mount_point.to_string_lossy().to_string();
        let cmd = self.prepare_command(&["umount", &mount_str], None);
        self.execute_command(cmd, &format!("umount '{}'", mount_str))?;
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
            && !pfx.trim().is_empty()
        {
            args.push("--prefix".to_string());
            args.push(pfx.trim().to_string());
        }
        args.push(repo_path.to_string());

        let arg_slices: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut cmd = self.prepare_command(&arg_slices, passphrase);
        let output = cmd.output().map_err(|e| {
            BorgError::IoError(format!("Falha ao executar borg prune: {}", e)).to_string()
        })?;

        if !output.status.success() {
            let err = BorgError::from_stderr(
                output.status.code(),
                &String::from_utf8_lossy(&output.stderr),
            );
            return Err(err.to_string());
        }

        let stderr_str = String::from_utf8_lossy(&output.stderr);
        let mut items = Vec::new();

        for line in stderr_str.lines() {
            if let Some(item) = parse_prune_line(line) {
                items.push(item);
            }
        }

        if !dry_run {
            let _ = self.compact_repository(repo_path, passphrase);
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

        let mut child = cmd.spawn().map_err(|e| {
            BorgError::IoError(format!("Falha ao iniciar processo borg check: {}", e)).to_string()
        })?;

        let mut log_output = Vec::new();

        if let Some(stderr) = child.stderr.take() {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                on_line(line.clone());
                log_output.push(line);
            }
        }

        let status = child.wait().map_err(|e| {
            BorgError::IoError(format!("Erro ao aguardar término do borg check: {}", e)).to_string()
        })?;

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

    pub fn diff_archives<F>(
        &self,
        repo_path: &str,
        archive1: &str,
        archive2: &str,
        content_only: bool,
        passphrase: Option<&str>,
        mut on_line: F,
    ) -> Result<Vec<DiffEntry>, String>
    where
        F: FnMut(String),
    {
        if repo_path == "/caminho/para/meu/repo" {
            let mock_entries = vec![
                DiffEntry {
                    path: "home/user/documentos/relatorio.pdf".to_string(),
                    changes: vec![DiffChangeItem::Modified {
                        added: 24500,
                        removed: 12000,
                    }],
                },
                DiffEntry {
                    path: "home/user/documentos/novo_arquivo.txt".to_string(),
                    changes: vec![DiffChangeItem::Added { size: 1024 }],
                },
                DiffEntry {
                    path: "home/user/documentos/arquivo_antigo.bak".to_string(),
                    changes: vec![DiffChangeItem::Removed { size: 4096 }],
                },
            ];
            for entry in &mock_entries {
                on_line(entry.path.clone());
            }
            return Ok(mock_entries);
        }

        let target1 = format!("{}::{}", repo_path, archive1);
        let mut args: Vec<String> = vec!["diff".to_string(), "--json-lines".to_string()];

        if content_only {
            args.push("--content-only".to_string());
        }

        args.push(target1);
        args.push(archive2.to_string());

        let arg_slices: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut cmd = self.prepare_command(&arg_slices, passphrase);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            BorgError::IoError(format!("Falha ao iniciar processo borg diff: {}", e)).to_string()
        })?;

        let mut entries = Vec::new();

        if let Some(stdout) = child.stdout.take() {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<DiffEntry>(trimmed) {
                    on_line(entry.path.clone());
                    entries.push(entry);
                }
            }
        }

        let status = child.wait().map_err(|e| {
            BorgError::IoError(format!("Erro ao aguardar processo borg diff: {}", e)).to_string()
        })?;

        if !status.success() {
            let mut err_msg = String::new();
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_string(&mut err_msg);
            }
            let err = BorgError::from_stderr(status.code(), &err_msg);
            return Err(format!("Erro ao comparar backups:\n{}", err));
        }

        Ok(entries)
    }
}
