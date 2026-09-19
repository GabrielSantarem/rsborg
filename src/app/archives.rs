use std::path::PathBuf;
use std::thread;
use std::time::Instant;

use crate::app::App;
use crate::app::state::{AppState, LoadingInfo, RestoreRequest, ThreadStatus};
use crate::borg::{BackupProgress, BorgManager};
use crate::config;

impl App {
    pub fn ask_delete_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i)
        {
            self.state = AppState::ConfirmDelete(archive.name.clone());
        }
    }

    pub fn confirm_delete_archive(&mut self, archive_name: String) {
        let (repo_path, passphrase) = match self.get_active_repo() {
            Some(r) => (r.location.clone(), r.passphrase.clone()),
            None => (config::get_default_repo_path(), None),
        };

        self.loading_info = LoadingInfo {
            message: self.t.loading_deleting_fmt(&archive_name),
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
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i)
        {
            let name = archive.name.clone();
            let (repo_path, passphrase) = match self.get_active_repo() {
                Some(r) => (r.location.clone(), r.passphrase.clone()),
                None => (config::get_default_repo_path(), None),
            };

            if repo_path.contains(':') && !repo_path.starts_with('/') {
                self.state = AppState::ErrorPopup(self.t.err_fuse_unsupported().to_string());
                return;
            }

            self.loading_info = LoadingInfo {
                message: self.t.loading_inspecting_fmt(&name),
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
                let res = manager.list_archive_contents(&repo_path, &name, passphrase.as_deref());
                let _ = tx.send(ThreadStatus::DoneInspect(res, name));
            });
        }
    }

    pub fn ask_restore_selected_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i)
        {
            let dest = config::get_default_restore_dir(&archive.name);
            self.state = AppState::ConfirmRestore(RestoreRequest {
                archive_name: archive.name.clone(),
                destination_path: dest.to_string_lossy().to_string(),
                paths_to_extract: vec![],
            });
        }
    }

    pub fn ask_restore_specific_file(&mut self, file_path: String) {
        if let AppState::InspectArchive(ref inspect) = self.state {
            let archive_name = inspect.archive_name.clone();
            let dest = config::get_default_restore_dir(&archive_name);
            self.state = AppState::ConfirmRestore(RestoreRequest {
                archive_name,
                destination_path: dest.to_string_lossy().to_string(),
                paths_to_extract: vec![file_path],
            });
        }
    }

    pub fn confirm_restore(&mut self) {
        if let AppState::ConfirmRestore(ref req) = self.state {
            let archive_name = req.archive_name.clone();
            let dest_path = PathBuf::from(req.destination_path.trim());
            let paths = req.paths_to_extract.clone();

            if req.destination_path.trim().is_empty() {
                self.state = AppState::ErrorPopup(self.t.err_repo_loc_empty().to_string());
                return;
            }

            let (repo_path, passphrase) = match self.get_active_repo() {
                Some(r) => (r.location.clone(), r.passphrase.clone()),
                None => (config::get_default_repo_path(), None),
            };

            let is_granular = !paths.is_empty();
            let msg = if is_granular {
                let target = paths.first().map(|s| s.as_str()).unwrap_or("");
                format!(
                    "{}: {}",
                    self.t.loading_restoring_fmt(&archive_name),
                    target
                )
            } else {
                self.t.loading_restoring_fmt(&archive_name)
            };

            self.loading_info = LoadingInfo {
                message: msg,
                elapsed_secs: 0,
                original_size: "-".to_string(),
                compressed_size: "-".to_string(),
                deduplicated_size: "-".to_string(),
                files_count: "0".to_string(),
                current_file: "Iniciando extração de arquivos...".to_string(),
                raw_line: String::new(),
                spinner_frame: 0,
            };
            self.loading_start = Some(Instant::now());
            self.state = AppState::Loading;

            let tx = self.tx.clone();
            let manager = BorgManager::new();
            let dest_clone = dest_path.clone();

            thread::spawn(move || {
                let tx_prog = tx.clone();
                let mut count = 0;
                let res = manager.extract_archive(
                    &repo_path,
                    &archive_name,
                    &dest_clone,
                    &paths,
                    passphrase.as_deref(),
                    move |extracted_file| {
                        count += 1;
                        let _ = tx_prog.send(ThreadStatus::Progress(BackupProgress {
                            raw_line: format!("Extraindo {}", extracted_file),
                            files_count: format!("{}", count),
                            current_file: extracted_file,
                            ..Default::default()
                        }));
                    },
                );

                match res {
                    Ok(_) => {
                        let _ = tx.send(ThreadStatus::DoneRestore(
                            dest_clone.to_string_lossy().to_string(),
                        ));
                    }
                    Err(e) => {
                        let _ = tx.send(ThreadStatus::Error(e));
                    }
                }
            });
        }
    }

    pub fn mount_selected_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i)
        {
            let name = archive.name.clone();

            if let Some(existing) = self.mounted_archives.get(&name) {
                self.state = AppState::SuccessPopup(format!(
                    "Backup '{}' já está montado em:\n{}\n\nUse [u] para desmontar.",
                    name,
                    existing.display()
                ));
                return;
            }

            let mount_point = config::get_mount_dir().join(&name);
            let (repo_path, passphrase) = match self.get_active_repo() {
                Some(r) => (r.location.clone(), r.passphrase.clone()),
                None => (config::get_default_repo_path(), None),
            };

            if repo_path.contains(':') && !repo_path.starts_with('/') {
                self.state = AppState::ErrorPopup(self.t.err_fuse_unsupported().to_string());
                return;
            }

            match self.borg_manager.mount_archive(
                &repo_path,
                &name,
                &mount_point,
                passphrase.as_deref(),
            ) {
                Ok(_) => {
                    self.mounted_archives
                        .insert(name.clone(), mount_point.clone());
                    self.state = AppState::SuccessPopup(
                        self.t
                            .msg_mount_success_fmt(&name, &mount_point.display().to_string()),
                    );
                }
                Err(e) => {
                    self.state =
                        AppState::ErrorPopup(format!("Erro ao montar backup FUSE:\n{}", e));
                }
            }
        }
    }

    pub fn umount_selected_archive(&mut self) {
        if let Some(i) = self.table_state.selected()
            && let Some(archive) = self.archives.get(i)
        {
            let name = archive.name.clone();

            if let Some(mount_point) = self.mounted_archives.remove(&name) {
                match self.borg_manager.umount_archive(&mount_point) {
                    Ok(_) => {
                        self.state =
                            AppState::SuccessPopup(self.t.msg_umount_success().to_string());
                    }
                    Err(e) => {
                        self.mounted_archives.insert(name, mount_point);
                        self.state = AppState::ErrorPopup(format!("Erro ao desmontar:\n{}", e));
                    }
                }
            } else {
                self.state = AppState::ErrorPopup(self.t.err_not_mounted_fmt(&name));
            }
        }
    }
}
