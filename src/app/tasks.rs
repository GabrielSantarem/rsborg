use crate::app::App;
use crate::app::state::{
    AppState, CheckResultState, DiffViewState, InspectState, PrunePlanState, ThreadStatus,
};

impl App {
    pub fn handle_background_tasks(&mut self) {
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
                    let created_name = std::mem::take(&mut self.new_backup_name);
                    self.file_browser.explicit_includes.clear();
                    self.file_browser.explicit_excludes.clear();
                    self.loading_start = None;
                    self.state = if !created_name.is_empty() {
                        AppState::SuccessPopup(self.t.msg_backup_success_fmt(&created_name))
                    } else {
                        AppState::Browsing
                    };
                }
                ThreadStatus::DoneDelete => {
                    self.load_repository();
                    self.loading_start = None;
                    self.state = AppState::Browsing;
                }
                ThreadStatus::DoneRestore(target) => {
                    self.loading_start = None;
                    self.state = AppState::SuccessPopup(self.t.msg_restore_success_fmt(&target));
                }
                ThreadStatus::DoneInspect(res, name) => {
                    self.loading_start = None;
                    match res {
                        Ok(entries) => {
                            self.state = AppState::InspectArchive(InspectState {
                                archive_name: name,
                                entries,
                                selected_index: 0,
                            });
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::DonePruneDryRun(res, policy) => {
                    self.loading_start = None;
                    match res {
                        Ok(items) => {
                            self.state = AppState::PrunePlanView(PrunePlanState {
                                items,
                                selected_index: 0,
                                policy,
                            });
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::DonePruneExecute(res) => {
                    self.loading_start = None;
                    match res {
                        Ok(pruned_count) => {
                            self.load_repository();
                            self.state =
                                AppState::SuccessPopup(self.t.msg_prune_success_fmt(pruned_count));
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::DoneCheck(res, target_display, mode_display) => {
                    self.loading_start = None;
                    match res {
                        Ok(result) => {
                            self.state = AppState::CheckResultView(CheckResultState {
                                result,
                                target_display,
                                mode_display,
                                log_scroll: 0,
                            });
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::DoneDiff(res, archive1, archive2) => {
                    self.loading_start = None;
                    match res {
                        Ok(entries) => {
                            self.state = AppState::DiffView(DiffViewState {
                                archive1,
                                archive2,
                                entries,
                                selected_index: 0,
                            });
                        }
                        Err(e) => {
                            self.state = AppState::ErrorPopup(e);
                        }
                    }
                }
                ThreadStatus::Error(e) => {
                    self.loading_start = None;
                    self.state = AppState::ErrorPopup(e);
                }
            }
        }
    }
}
