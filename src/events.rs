use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{
    App, AppState, CheckResultState, CheckWizardState, CreateFocus, InspectState,
    PrunePlanState, PrunePolicyState, RestoreRequest,
};
use crate::borg::BorgCheckMode;
use crate::config;

enum EventOutcome {
    None,
    Quit,
    BackToBrowsing,
    ConfirmRestore,
    AskRestoreSpecificFile(String),
    ExecutePruneDryRun,
    ConfirmExecutePrune,
    StartCheck,
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    let outcome = match &mut app.state {
        AppState::Browsing => {
            handle_browsing(app, key);
            EventOutcome::None
        }
        AppState::CreatingBackup => {
            handle_creating_backup(app, key);
            EventOutcome::None
        }
        AppState::ConfirmDelete(name) => {
            let target_name = name.clone();
            handle_confirm_delete(app, key, target_name);
            EventOutcome::None
        }
        AppState::ConfirmRestore(req) => handle_confirm_restore(req, key),
        AppState::InspectArchive(inspect) => handle_inspect_archive(inspect, key),
        AppState::PruningPolicy(policy_state) => handle_pruning_policy(policy_state, key),
        AppState::PrunePlanView(plan_state) => handle_prune_plan_view(plan_state, key),
        AppState::CheckWizard(wizard_state) => handle_check_wizard(wizard_state, key),
        AppState::CheckResultView(result_state) => handle_check_result_view(result_state, key),
        AppState::ManagingRepos => {
            handle_managing_repos(app, key);
            EventOutcome::None
        }
        AppState::AddingRepo => {
            handle_adding_repo(app, key);
            EventOutcome::None
        }
        AppState::ErrorPopup(_) | AppState::SuccessPopup(_) => match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('s') => {
                EventOutcome::BackToBrowsing
            }
            _ => EventOutcome::None,
        },
        AppState::Loading => EventOutcome::None,
        _ => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => EventOutcome::Quit,
            _ => EventOutcome::None,
        },
    };

    match outcome {
        EventOutcome::None => {}
        EventOutcome::Quit => app.quit(),
        EventOutcome::BackToBrowsing => app.state = AppState::Browsing,
        EventOutcome::ConfirmRestore => app.confirm_restore(),
        EventOutcome::AskRestoreSpecificFile(path) => app.ask_restore_specific_file(path),
        EventOutcome::ExecutePruneDryRun => app.execute_prune_dry_run(),
        EventOutcome::ConfirmExecutePrune => app.confirm_execute_prune(),
        EventOutcome::StartCheck => app.start_check(),
    }
}

fn handle_browsing(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Char('l') => app.toggle_language(),
        KeyCode::Char('c') => {
            app.state = AppState::CreatingBackup;
            app.create_focus = CreateFocus::Name;
            app.new_backup_name.clear();
            app.file_browser.explicit_includes.clear();
            app.file_browser.explicit_excludes.clear();
            app.file_browser.load_entries();
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.open_check_wizard();
        }
        KeyCode::Char('p') => {
            app.open_prune_policy_modal();
        }
        KeyCode::Char('x') => {
            app.ask_restore_selected_archive();
        }
        KeyCode::Char('m') => {
            app.mount_selected_archive();
        }
        KeyCode::Char('u') => {
            app.umount_selected_archive();
        }
        KeyCode::Char('d') => {
            app.ask_delete_archive();
        }
        KeyCode::Enter => {
            app.inspect_selected_archive();
        }
        KeyCode::Char('r') => {
            app.state = AppState::ManagingRepos;
            app.repo_list_index = 0;
        }
        _ => {}
    }
}

fn handle_creating_backup(app: &mut App, key: KeyEvent) {
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('s') {
        app.validate_and_submit_backup();
        return;
    }

    if key.code == KeyCode::Tab {
        app.create_focus = match app.create_focus {
            CreateFocus::Name => CreateFocus::Browser,
            CreateFocus::Browser => CreateFocus::Name,
        };
        return;
    }

    if key.code == KeyCode::Esc {
        app.state = AppState::Browsing;
        return;
    }

    match app.create_focus {
        CreateFocus::Name => match key.code {
            KeyCode::Enter => {
                app.create_focus = CreateFocus::Browser;
            }
            KeyCode::Backspace => {
                app.new_backup_name.pop();
            }
            KeyCode::Char(c) => {
                app.new_backup_name.push(c);
            }
            _ => {}
        },
        CreateFocus::Browser => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.file_browser.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.file_browser.previous();
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                if let Some(entry) = app
                    .file_browser
                    .entries
                    .get(app.file_browser.selected_index)
                {
                    if entry.is_parent_link {
                        app.file_browser.go_up();
                    } else if entry.is_dir {
                        app.file_browser.enter_dir();
                    }
                }
            }
            KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => {
                app.file_browser.go_up();
            }
            KeyCode::Char(' ') => {
                app.file_browser.toggle_selection();
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                app.validate_and_submit_backup();
            }
            _ => {}
        },
    }
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent, target_name: String) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            app.confirm_delete_archive(target_name);
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.state = AppState::Browsing;
        }
        _ => {}
    }
}

fn handle_confirm_restore(req: &mut RestoreRequest, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Enter => EventOutcome::ConfirmRestore,
        KeyCode::Esc => EventOutcome::BackToBrowsing,
        KeyCode::Backspace => {
            req.destination_path.pop();
            EventOutcome::None
        }
        KeyCode::Char(c) => {
            req.destination_path.push(c);
            EventOutcome::None
        }
        _ => EventOutcome::None,
    }
}

fn handle_inspect_archive(inspect: &mut InspectState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if !inspect.entries.is_empty() {
                if inspect.selected_index >= inspect.entries.len().saturating_sub(1) {
                    inspect.selected_index = 0;
                } else {
                    inspect.selected_index += 1;
                }
            }
            EventOutcome::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if !inspect.entries.is_empty() {
                if inspect.selected_index == 0 {
                    inspect.selected_index = inspect.entries.len().saturating_sub(1);
                } else {
                    inspect.selected_index -= 1;
                }
            }
            EventOutcome::None
        }
        KeyCode::Char('x') => {
            if let Some(entry) = inspect.entries.get(inspect.selected_index) {
                EventOutcome::AskRestoreSpecificFile(entry.path.clone())
            } else {
                EventOutcome::None
            }
        }
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => EventOutcome::BackToBrowsing,
        _ => EventOutcome::None,
    }
}

fn handle_pruning_policy(policy_state: &mut PrunePolicyState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Tab | KeyCode::Down => {
            policy_state.focus_field = (policy_state.focus_field + 1) % 6;
            EventOutcome::None
        }
        KeyCode::Up => {
            if policy_state.focus_field == 0 {
                policy_state.focus_field = 5;
            } else {
                policy_state.focus_field -= 1;
            }
            EventOutcome::None
        }
        KeyCode::Enter => EventOutcome::ExecutePruneDryRun,
        KeyCode::Esc => EventOutcome::BackToBrowsing,
        KeyCode::Backspace => {
            match policy_state.focus_field {
                0 => {
                    policy_state.last_str.pop();
                }
                1 => {
                    policy_state.daily_str.pop();
                }
                2 => {
                    policy_state.weekly_str.pop();
                }
                3 => {
                    policy_state.monthly_str.pop();
                }
                4 => {
                    policy_state.yearly_str.pop();
                }
                5 => {
                    policy_state.prefix_str.pop();
                }
                _ => {}
            }
            EventOutcome::None
        }
        KeyCode::Char(c) => {
            match policy_state.focus_field {
                0
                    if c.is_ascii_digit() => {
                        policy_state.last_str.push(c);
                    }
                1
                    if c.is_ascii_digit() => {
                        policy_state.daily_str.push(c);
                    }
                2
                    if c.is_ascii_digit() => {
                        policy_state.weekly_str.push(c);
                    }
                3
                    if c.is_ascii_digit() => {
                        policy_state.monthly_str.push(c);
                    }
                4
                    if c.is_ascii_digit() => {
                        policy_state.yearly_str.push(c);
                    }
                5 => {
                    policy_state.prefix_str.push(c);
                }
                _ => {}
            }
            EventOutcome::None
        }
        _ => EventOutcome::None,
    }
}

fn handle_prune_plan_view(plan_state: &mut PrunePlanState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if !plan_state.items.is_empty() {
                if plan_state.selected_index >= plan_state.items.len().saturating_sub(1) {
                    plan_state.selected_index = 0;
                } else {
                    plan_state.selected_index += 1;
                }
            }
            EventOutcome::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if !plan_state.items.is_empty() {
                if plan_state.selected_index == 0 {
                    plan_state.selected_index = plan_state.items.len().saturating_sub(1);
                } else {
                    plan_state.selected_index -= 1;
                }
            }
            EventOutcome::None
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => EventOutcome::ConfirmExecutePrune,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => EventOutcome::BackToBrowsing,
        _ => EventOutcome::None,
    }
}

fn handle_managing_repos(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down
            if !app.config.repositories.is_empty() => {
                if app.repo_list_index >= app.config.repositories.len().saturating_sub(1) {
                    app.repo_list_index = 0;
                } else {
                    app.repo_list_index += 1;
                }
            }
        KeyCode::Char('k') | KeyCode::Up
            if !app.config.repositories.is_empty() => {
                if app.repo_list_index == 0 {
                    app.repo_list_index = app.config.repositories.len().saturating_sub(1);
                } else {
                    app.repo_list_index -= 1;
                }
            }
        KeyCode::Enter => {
            if let Some(repo) = app.config.repositories.get(app.repo_list_index) {
                let id = repo.id.clone();
                app.switch_active_repo(id);
            }
        }
        KeyCode::Char('a') => {
            app.state = AppState::AddingRepo;
            app.add_repo_focus = 0;
            app.new_repo_name.clear();
            app.new_repo_location.clear();
            app.new_repo_passphrase.clear();
        }
        KeyCode::Char('d')
            if app.config.repositories.len() > 1 => {
                let removed = app.config.repositories.remove(app.repo_list_index);
                if app.repo_list_index >= app.config.repositories.len() {
                    app.repo_list_index = app.config.repositories.len().saturating_sub(1);
                }
                if app.config.active_repo_id == removed.id {
                    app.config.active_repo_id = app.config.repositories[0].id.clone();
                    app.load_repository();
                }
                let _ = config::save_config(&app.config);
            }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.state = AppState::Browsing;
        }
        _ => {}
    }
}

fn handle_adding_repo(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => {
            app.add_repo_focus = (app.add_repo_focus + 1) % 3;
        }
        KeyCode::Enter => {
            app.save_new_repository();
        }
        KeyCode::Esc => {
            app.state = AppState::ManagingRepos;
        }
        KeyCode::Backspace => match app.add_repo_focus {
            0 => {
                app.new_repo_name.pop();
            }
            1 => {
                app.new_repo_location.pop();
            }
            2 => {
                app.new_repo_passphrase.pop();
            }
            _ => {}
        },
        KeyCode::Char(c) => match app.add_repo_focus {
            0 => {
                app.new_repo_name.push(c);
            }
            1 => {
                app.new_repo_location.push(c);
            }
            2 => {
                app.new_repo_passphrase.push(c);
            }
            _ => {}
        },
        _ => {}
    }
}

fn handle_check_wizard(wizard_state: &mut CheckWizardState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Esc => EventOutcome::BackToBrowsing,
        KeyCode::Tab => {
            if wizard_state.target_archive.is_some() {
                wizard_state.check_archive_only = !wizard_state.check_archive_only;
            }
            EventOutcome::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            wizard_state.check_mode = match wizard_state.check_mode {
                BorgCheckMode::RepositoryOnly => BorgCheckMode::Repair,
                BorgCheckMode::Standard => BorgCheckMode::RepositoryOnly,
                BorgCheckMode::VerifyData => BorgCheckMode::Standard,
                BorgCheckMode::Repair => BorgCheckMode::VerifyData,
            };
            EventOutcome::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            wizard_state.check_mode = match wizard_state.check_mode {
                BorgCheckMode::RepositoryOnly => BorgCheckMode::Standard,
                BorgCheckMode::Standard => BorgCheckMode::VerifyData,
                BorgCheckMode::VerifyData => BorgCheckMode::Repair,
                BorgCheckMode::Repair => BorgCheckMode::RepositoryOnly,
            };
            EventOutcome::None
        }
        KeyCode::Enter => EventOutcome::StartCheck,
        _ => EventOutcome::None,
    }
}

fn handle_check_result_view(result_state: &mut CheckResultState, key: KeyEvent) -> EventOutcome {
    let total_lines = result_state.result.log_output.len();
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => EventOutcome::BackToBrowsing,
        KeyCode::Down | KeyCode::Char('j') => {
            if total_lines > 0 && result_state.log_scroll < total_lines.saturating_sub(1) {
                result_state.log_scroll += 1;
            }
            EventOutcome::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            result_state.log_scroll = result_state.log_scroll.saturating_sub(1);
            EventOutcome::None
        }
        KeyCode::PageDown => {
            if total_lines > 0 {
                result_state.log_scroll =
                    (result_state.log_scroll + 10).min(total_lines.saturating_sub(1));
            }
            EventOutcome::None
        }
        KeyCode::PageUp => {
            result_state.log_scroll = result_state.log_scroll.saturating_sub(10);
            EventOutcome::None
        }
        _ => EventOutcome::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borg::CheckResult;

    #[test]
    fn test_browsing_v_opens_check_wizard() {
        let mut app = App::new();
        app.state = AppState::Browsing;

        handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('v')));
        match app.state {
            AppState::CheckWizard(ref state) => {
                assert_eq!(state.check_mode, BorgCheckMode::Standard);
            }
            _ => panic!("Esperava CheckWizard ao pressionar 'v'"),
        }
    }

    #[test]
    fn test_check_wizard_navigation_and_toggle() {
        let mut app = App::new();
        app.state = AppState::CheckWizard(CheckWizardState {
            target_archive: Some("archive1".to_string()),
            check_mode: BorgCheckMode::Standard,
            check_archive_only: false,
        });

        // Tab should toggle check_archive_only
        handle_key_event(&mut app, KeyEvent::from(KeyCode::Tab));
        if let AppState::CheckWizard(ref state) = app.state {
            assert!(state.check_archive_only);
        }

        // Down should cycle to VerifyData
        handle_key_event(&mut app, KeyEvent::from(KeyCode::Down));
        if let AppState::CheckWizard(ref state) = app.state {
            assert_eq!(state.check_mode, BorgCheckMode::VerifyData);
        }

        // Up should cycle back to Standard
        handle_key_event(&mut app, KeyEvent::from(KeyCode::Up));
        if let AppState::CheckWizard(ref state) = app.state {
            assert_eq!(state.check_mode, BorgCheckMode::Standard);
        }

        // Esc should exit back to browsing
        handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.state, AppState::Browsing);
    }

    #[test]
    fn test_check_result_view_scroll_and_exit() {
        let mut app = App::new();
        app.state = AppState::CheckResultView(CheckResultState {
            result: CheckResult {
                success: true,
                warnings: false,
                log_output: vec!["line 1".to_string(), "line 2".to_string(), "line 3".to_string()],
            },
            target_display: "Repo".to_string(),
            mode_display: "Standard".to_string(),
            log_scroll: 0,
        });

        // Down should scroll
        handle_key_event(&mut app, KeyEvent::from(KeyCode::Down));
        if let AppState::CheckResultView(ref state) = app.state {
            assert_eq!(state.log_scroll, 1);
        }

        // Esc should exit
        handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.state, AppState::Browsing);
    }
}
