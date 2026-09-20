use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::app::state::{AppState, CreateFocus};
use crate::browser::ItemStatus;

pub fn handle_browsing(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('l') | KeyCode::Char('L') => app.toggle_language(),
        KeyCode::Char('t') | KeyCode::Char('T') => app.toggle_theme(),
        KeyCode::Char('?') | KeyCode::F(1) => {
            app.state = AppState::HelpModal;
        }
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Up | KeyCode::Char('k') => app.previous(),
        KeyCode::Char('r') => {
            app.repo_list_index = 0;
            app.state = AppState::ManagingRepos;
        }
        KeyCode::Char('c') => {
            app.state = AppState::CreatingBackup;
            app.create_focus = CreateFocus::Name;
            app.new_backup_name.clear();
        }
        KeyCode::Char('b') => {
            app.open_profiles_view();
        }
        KeyCode::Char('p') => {
            app.open_prune_policy_modal();
        }
        KeyCode::Char('v') => {
            app.open_check_wizard();
        }
        KeyCode::Char('f') => {
            app.open_diff_wizard();
        }
        KeyCode::Char('d') => {
            app.ask_delete_archive();
        }
        KeyCode::Enter => {
            app.inspect_selected_archive();
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
        _ => {}
    }
}

pub fn handle_creating_backup(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        app.validate_and_submit_backup();
        return;
    }

    match app.create_focus {
        CreateFocus::Name => match key.code {
            KeyCode::Esc => {
                app.state = AppState::Browsing;
            }
            KeyCode::Tab | KeyCode::Down => {
                app.create_focus = CreateFocus::Browser;
            }
            KeyCode::Char('s') if key.modifiers.is_empty() => {
                app.validate_and_submit_backup();
            }
            KeyCode::Char(c) => {
                app.new_backup_name.push(c);
            }
            KeyCode::Backspace => {
                app.new_backup_name.pop();
            }
            KeyCode::Enter => {
                app.validate_and_submit_backup();
            }
            _ => {}
        },
        CreateFocus::Browser => match key.code {
            KeyCode::Esc => {
                app.state = AppState::Browsing;
            }
            KeyCode::Tab => {
                app.create_focus = CreateFocus::Name;
            }
            KeyCode::Char('s') => {
                app.validate_and_submit_backup();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.file_browser.previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.file_browser.next();
            }
            KeyCode::Enter | KeyCode::Right => {
                app.file_browser.enter_dir();
            }
            KeyCode::Backspace | KeyCode::Left => {
                app.file_browser.go_up();
            }
            KeyCode::Char(' ') => {
                app.file_browser.toggle_selection();
            }
            KeyCode::Char('e') => {
                if let Some(entry) = app
                    .file_browser
                    .entries
                    .get(app.file_browser.selected_index)
                {
                    if entry.is_parent_link {
                        return;
                    }
                    let path = entry.path.clone();
                    let current_status = app.file_browser.get_status(&path);
                    if current_status == ItemStatus::ExplicitExclude {
                        app.file_browser.explicit_excludes.remove(&path);
                    } else {
                        app.file_browser.explicit_includes.remove(&path);
                        app.file_browser.explicit_excludes.insert(path);
                    }
                }
            }
            _ => {}
        },
    }
}
