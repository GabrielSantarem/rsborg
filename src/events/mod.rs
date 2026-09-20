pub mod browsing;
pub mod diagnostics;
pub mod dialogs;
pub mod profiles;
pub mod prune;
pub mod repos;

#[cfg(test)]
mod tests;

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::app::state::AppState;

#[derive(Debug, PartialEq, Eq)]
pub enum EventOutcome {
    Continue,
    ExitModal,
    Done,
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match &mut app.state {
        AppState::ErrorPopup(err) => {
            let is_lock = err.contains("break-lock") || err.contains("bloqueado") || err.contains("locked");
            if is_lock && (key.code == KeyCode::Char('b') || key.code == KeyCode::Char('B')) {
                app.break_lock_active_repo();
                return;
            }
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                    app.state = AppState::Browsing;
                }
                _ => {}
            }
        }
        AppState::InitError(err) => {
            let is_lock = err.contains("break-lock") || err.contains("bloqueado") || err.contains("locked");
            if is_lock && (key.code == KeyCode::Char('b') || key.code == KeyCode::Char('B')) {
                app.break_lock_active_repo();
                return;
            }
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                    app.state = AppState::Browsing;
                }
                _ => {}
            }
        }
        AppState::SuccessPopup(_) => {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                    app.state = AppState::Browsing;
                }
                _ => {}
            }
        }
        AppState::Loading => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                app.state = AppState::Browsing;
            }
            _ => {}
        },
        AppState::Browsing => browsing::handle_browsing(app, key),
        AppState::CreatingBackup => browsing::handle_creating_backup(app, key),
        AppState::ConfirmDelete(target_name) => {
            let name = target_name.clone();
            dialogs::handle_confirm_delete(app, key, name);
        }
        AppState::ConfirmRestore(req) => match dialogs::handle_confirm_restore(req, key) {
            EventOutcome::Continue => {}
            EventOutcome::ExitModal => {
                app.state = AppState::Browsing;
            }
            EventOutcome::Done => {
                app.confirm_restore();
            }
        },
        AppState::InspectArchive(inspect) => match dialogs::handle_inspect_archive(inspect, key) {
            EventOutcome::Continue => {}
            EventOutcome::ExitModal => {
                app.state = AppState::Browsing;
            }
            EventOutcome::Done => {
                if let Some(entry) = inspect.entries.get(inspect.selected_index) {
                    let path = entry.path.clone();
                    app.ask_restore_specific_file(path);
                }
            }
        },
        AppState::PruningPolicy(policy_state) => {
            match prune::handle_pruning_policy(policy_state, key) {
                EventOutcome::Continue => {}
                EventOutcome::ExitModal => {
                    app.state = AppState::Browsing;
                }
                EventOutcome::Done => {
                    app.execute_prune_dry_run();
                }
            }
        }
        AppState::PrunePlanView(plan_state) => {
            match prune::handle_prune_plan_view(plan_state, key) {
                EventOutcome::Continue => {}
                EventOutcome::ExitModal => {
                    app.state = AppState::Browsing;
                }
                EventOutcome::Done => {
                    app.confirm_execute_prune();
                }
            }
        }
        AppState::ManagingRepos => repos::handle_managing_repos(app, key),
        AppState::AddingRepo => repos::handle_adding_repo(app, key),
        AppState::CheckWizard(wizard_state) => {
            match diagnostics::handle_check_wizard(wizard_state, key) {
                EventOutcome::Continue => {}
                EventOutcome::ExitModal => {
                    app.state = AppState::Browsing;
                }
                EventOutcome::Done => {
                    app.start_check();
                }
            }
        }
        AppState::CheckResultView(result_state) => {
            match diagnostics::handle_check_result_view(result_state, key) {
                EventOutcome::Continue => {}
                EventOutcome::ExitModal => {
                    app.state = AppState::Browsing;
                }
                EventOutcome::Done => {}
            }
        }
        AppState::DiffWizard(wizard_state) => {
            match diagnostics::handle_diff_wizard(wizard_state, key) {
                EventOutcome::Continue => {}
                EventOutcome::ExitModal => {
                    app.state = AppState::Browsing;
                }
                EventOutcome::Done => {
                    app.start_diff();
                }
            }
        }
        AppState::DiffView(view_state) => match diagnostics::handle_diff_view(view_state, key) {
            EventOutcome::Continue => {}
            EventOutcome::ExitModal => {
                app.state = AppState::Browsing;
            }
            EventOutcome::Done => {}
        },
        AppState::ManagingProfiles { selected_index } => {
            let idx = *selected_index;
            profiles::handle_managing_profiles(app, key, idx);
        }
        AppState::CreatingProfile(_) => profiles::handle_creating_profile(app, key),
        AppState::AutomationView(_) => profiles::handle_automation_view(app, key),
        AppState::HelpModal => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
                app.state = AppState::Browsing;
            }
            _ => {}
        },
        AppState::Initializing => {}
    }
}
