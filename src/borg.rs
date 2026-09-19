use serde::{Deserialize, Serialize};
use std::io::Read;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupArchive {
    pub archive: String,
    pub barchive: String,
    pub id: String,
    pub name: String,
    pub start: String,
    pub time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepositoryInfo {
    pub id: String,
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BorgArchiveList {
    pub archives: Vec<BackupArchive>,
    pub repository: RepositoryInfo,
}

#[derive(Debug, Clone, Default)]
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

        // Borg format: "12.34 MB O 5.67 MB C 1.20 MB D 45 N /path/to/file"
        // Let's parse components if available
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if let Some(pos_o) = tokens.iter().position(|&t| t == "O") {
            if pos_o >= 2 {
                progress.original_size = format!("{} {}", tokens[pos_o - 2], tokens[pos_o - 1]);
            }
        }
        if let Some(pos_c) = tokens.iter().position(|&t| t == "C") {
            if pos_c >= 2 {
                progress.compressed_size = format!("{} {}", tokens[pos_c - 2], tokens[pos_c - 1]);
            }
        }
        if let Some(pos_d) = tokens.iter().position(|&t| t == "D") {
            if pos_d >= 2 {
                progress.deduplicated_size = format!("{} {}", tokens[pos_d - 2], tokens[pos_d - 1]);
            }
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

    pub fn init_repository(&self, repo_path: &str) -> Result<(), String> {
        let o = Command::new(&self.bin_path)
            .args(["init", "-e", "none", repo_path])
            .output()
            .map_err(|e| e.to_string())?;

        if o.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&o.stderr).to_string())
        }
    }

    pub fn list_archives(&self, repo_path: &str) -> Result<BorgArchiveList, String> {
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

        let output = Command::new(&self.bin_path)
            .args(["list", "--json", repo_path])
            .output()
            .map_err(|e| format!("Falha: {}", e))?;

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
        mut on_progress: F,
    ) -> Result<(), String>
    where
        F: FnMut(BackupProgress) + Send + 'static,
    {
        // Se for o repositório de teste/mock, simula o progresso realista
        if repo_path == "/caminho/para/meu/repo" {
            let sample_files = [
                "documentos/relatorio.pdf",
                "projetos/src/main.rs",
                "imagens/foto1.png",
                "musicas/faixa.flac",
                "videos/demo.mp4",
                "config/settings.json",
                "banco_de_dados.sqlite",
                "finalizando_arquivos...",
            ];

            for (i, file) in sample_files.iter().enumerate() {
                std::thread::sleep(std::time::Duration::from_millis(350));
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

        let mut child = Command::new(&self.bin_path)
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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
}
