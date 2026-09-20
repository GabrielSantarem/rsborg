#[cfg(test)]
mod tests {
    use crate::borg::*;
    use crate::config::PrunePolicy;

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
    fn test_diff_archives_mock() {
        let manager = BorgManager::new();
        let mut lines = Vec::new();
        let res =
            manager.diff_archives("/caminho/para/meu/repo", "b1", "b2", false, None, |line| {
                lines.push(line)
            });

        assert!(res.is_ok());
        let entries = res.unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].kind(), DiffKind::Modified);
        assert_eq!(entries[1].kind(), DiffKind::Added);
        assert_eq!(entries[2].kind(), DiffKind::Removed);
        assert_eq!(lines.len(), 3);
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

    #[test]
    fn test_borg_error_classification() {
        let err_locked =
            BorgError::from_stderr(Some(2), "Failed to create/acquire the lock on repo");
        assert!(matches!(err_locked, BorgError::RepositoryLocked(_)));

        let err_pass = BorgError::from_stderr(Some(2), "Passphrase incorrect");
        assert!(matches!(err_pass, BorgError::InvalidPassphrase(_)));

        let err_not_found = BorgError::from_stderr(Some(2), "Repository does not exist at /path");
        assert!(matches!(err_not_found, BorgError::RepositoryNotFound(_)));

        let err_cmd_not_found = BorgError::from_stderr(Some(127), "borg: command not found");
        assert!(matches!(err_cmd_not_found, BorgError::BorgNotFound(_)));

        let err_generic = BorgError::from_stderr(Some(1), "Some unknown failure");
        assert!(matches!(err_generic, BorgError::CommandFailed { .. }));
    }

    #[test]
    fn test_deserialize_real_borg_list_json() {
        let json_sample = r#"{
            "archives": [
                {
                    "archive": "backup1",
                    "barchive": "backup1",
                    "id": "f0d844a36aaae56e6e98cac9ad0e73d20cee6f2b54a78769844d6b8fa4bbad50",
                    "name": "backup1",
                    "start": "2026-09-19T05:40:45.000000",
                    "time": "2026-09-19T05:40:45.000000"
                }
            ],
            "encryption": { "mode": "none" },
            "repository": {
                "id": "731a661ac4393739d0fe4b02637af7a3c277a02f27d80daa774cf111ae713c8e",
                "last_modified": "2026-09-19T22:34:49.000000",
                "location": "/home/tomate/.rsborg/backups"
            }
        }"#;

        let res = serde_json::from_str::<BorgArchiveList>(json_sample);
        assert!(res.is_ok());
        let list = res.unwrap();
        assert_eq!(list.archives.len(), 1);
        assert_eq!(list.archives[0].name, "backup1");
        assert_eq!(list.repository.location, "/home/tomate/.rsborg/backups");
    }
}
