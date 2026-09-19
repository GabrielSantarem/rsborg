use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::app::state::{AppState, ProfileFocus};
use crate::browser::ItemStatus;

pub fn handle_managing_profiles(app: &mut App, key: KeyEvent, selected_index: usize) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.state = AppState::Browsing;
        }
        KeyCode::Down | KeyCode::Char('j') if !app.config.profiles.is_empty() => {
            let new_idx = (selected_index + 1).min(app.config.profiles.len() - 1);
            app.state = AppState::ManagingProfiles {
                selected_index: new_idx,
            };
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let new_idx = selected_index.saturating_sub(1);
            app.state = AppState::ManagingProfiles {
                selected_index: new_idx,
            };
        }
        KeyCode::Char('a') => {
            app.open_create_profile();
        }
        KeyCode::Char('d') => {
            app.delete_profile(selected_index);
        }
        KeyCode::Enter => {
            app.run_profile_backup(selected_index);
        }
        KeyCode::Char('s') => {
            if let Some(p) = app.config.profiles.get(selected_index) {
                app.state = AppState::AutomationView(crate::app::state::AutomationViewState {
                    profile: p.clone(),
                    active_tab: 0,
                });
            }
        }
        _ => {}
    }
}

pub fn handle_creating_profile(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        app.save_profile();
        return;
    }

    let mut current_focus = match &app.state {
        AppState::CreatingProfile(wizard) => wizard.focus,
        _ => ProfileFocus::Name,
    };

    match current_focus {
        ProfileFocus::Name => match key.code {
            KeyCode::Esc => {
                app.state = AppState::ManagingProfiles { selected_index: 0 };
            }
            KeyCode::Tab | KeyCode::Down => {
                current_focus = ProfileFocus::Compression;
            }
            KeyCode::Char(c) => {
                if let AppState::CreatingProfile(ref mut w) = app.state {
                    w.name.push(c);
                }
            }
            KeyCode::Backspace => {
                if let AppState::CreatingProfile(ref mut w) = app.state {
                    w.name.pop();
                }
            }
            KeyCode::Enter => {
                app.save_profile();
            }
            _ => {}
        },
        ProfileFocus::Compression => match key.code {
            KeyCode::Esc => {
                app.state = AppState::ManagingProfiles { selected_index: 0 };
            }
            KeyCode::Tab | KeyCode::Down => {
                current_focus = ProfileFocus::Schedule;
            }
            KeyCode::Up => {
                current_focus = ProfileFocus::Name;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let AppState::CreatingProfile(ref mut w) = app.state {
                    w.compression_idx = w.compression_idx.saturating_sub(1);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let AppState::CreatingProfile(ref mut w) = app.state {
                    w.compression_idx = (w.compression_idx + 1)
                        .min(crate::ui::profiles::COMPRESSION_OPTIONS.len() - 1);
                }
            }
            _ => {}
        },
        ProfileFocus::Schedule => match key.code {
            KeyCode::Esc => {
                app.state = AppState::ManagingProfiles { selected_index: 0 };
            }
            KeyCode::Tab | KeyCode::Down => {
                current_focus = ProfileFocus::Browser;
            }
            KeyCode::Up => {
                current_focus = ProfileFocus::Compression;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let AppState::CreatingProfile(ref mut w) = app.state {
                    w.schedule_idx = w.schedule_idx.saturating_sub(1);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let AppState::CreatingProfile(ref mut w) = app.state {
                    w.schedule_idx =
                        (w.schedule_idx + 1).min(crate::ui::profiles::SCHEDULE_OPTIONS.len() - 1);
                }
            }
            _ => {}
        },
        ProfileFocus::Browser => match key.code {
            KeyCode::Esc => {
                app.state = AppState::ManagingProfiles { selected_index: 0 };
            }
            KeyCode::Tab => {
                current_focus = ProfileFocus::Name;
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

    if let AppState::CreatingProfile(ref mut w) = app.state {
        w.focus = current_focus;
    }
}

pub fn handle_automation_view(app: &mut App, key: KeyEvent) {
    let (profile_clone, mut tab) = match &app.state {
        AppState::AutomationView(s) => (s.profile.clone(), s.active_tab),
        _ => return,
    };

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.state = AppState::ManagingProfiles { selected_index: 0 };
        }
        KeyCode::Tab | KeyCode::Right => {
            tab = (tab + 1) % 2;
            if let AppState::AutomationView(ref mut s) = app.state {
                s.active_tab = tab;
            }
        }
        KeyCode::Left => {
            tab = if tab == 0 { 1 } else { 0 };
            if let AppState::AutomationView(ref mut s) = app.state {
                s.active_tab = tab;
            }
        }
        KeyCode::Char('i') => {
            app.install_systemd_profile(&profile_clone);
        }
        _ => {}
    }
}
