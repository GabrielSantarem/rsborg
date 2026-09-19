use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
};
use std::{error::Error, io, time::Duration};

mod app;
mod borg;
mod browser;
mod checker;
mod config;
mod i18n;
mod ui;

use app::{App, AppState, CreateFocus};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    let mut app = App::new();
    app.on_start();

    let res = run_app(&mut terminal, &mut app);

    ratatui::restore();

    if let Err(err) = res {
        println!("Erro na aplicação: {err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        app.handle_background_tasks();

        if event::poll(Duration::from_millis(40))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match &mut app.state {
                        AppState::Browsing => match key.code {
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
                        },
                        AppState::CreatingBackup => {
                            if key.modifiers == KeyModifiers::CONTROL
                                && key.code == KeyCode::Char('s')
                            {
                                app.validate_and_submit_backup();
                                continue;
                            }

                            if key.code == KeyCode::Tab {
                                app.create_focus = match app.create_focus {
                                    CreateFocus::Name => CreateFocus::Browser,
                                    CreateFocus::Browser => CreateFocus::Name,
                                };
                                continue;
                            }

                            if key.code == KeyCode::Esc {
                                app.state = AppState::Browsing;
                                continue;
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
                        AppState::ConfirmDelete(name) => match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                                let target_name = name.clone();
                                app.confirm_delete_archive(target_name);
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.state = AppState::Browsing;
                            }
                            _ => {}
                        },
                        AppState::InspectArchive(inspect) => match key.code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                if !inspect.entries.is_empty() {
                                    if inspect.selected_index
                                        >= inspect.entries.len().saturating_sub(1)
                                    {
                                        inspect.selected_index = 0;
                                    } else {
                                        inspect.selected_index += 1;
                                    }
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                if !inspect.entries.is_empty() {
                                    if inspect.selected_index == 0 {
                                        inspect.selected_index =
                                            inspect.entries.len().saturating_sub(1);
                                    } else {
                                        inspect.selected_index -= 1;
                                    }
                                }
                            }
                            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                                app.state = AppState::Browsing;
                            }
                            _ => {}
                        },
                        AppState::ManagingRepos => match key.code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                if !app.config.repositories.is_empty() {
                                    if app.repo_list_index
                                        >= app.config.repositories.len().saturating_sub(1)
                                    {
                                        app.repo_list_index = 0;
                                    } else {
                                        app.repo_list_index += 1;
                                    }
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                if !app.config.repositories.is_empty() {
                                    if app.repo_list_index == 0 {
                                        app.repo_list_index =
                                            app.config.repositories.len().saturating_sub(1);
                                    } else {
                                        app.repo_list_index -= 1;
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(repo) = app.config.repositories.get(app.repo_list_index)
                                {
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
                            KeyCode::Char('d') => {
                                if app.config.repositories.len() > 1 {
                                    let removed =
                                        app.config.repositories.remove(app.repo_list_index);
                                    if app.repo_list_index >= app.config.repositories.len() {
                                        app.repo_list_index =
                                            app.config.repositories.len().saturating_sub(1);
                                    }
                                    if app.config.active_repo_id == removed.id {
                                        app.config.active_repo_id =
                                            app.config.repositories[0].id.clone();
                                        app.load_repository();
                                    }
                                    let _ = config::save_config(&app.config);
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.state = AppState::Browsing;
                            }
                            _ => {}
                        },
                        AppState::AddingRepo => match key.code {
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
                        },
                        AppState::ErrorPopup(_) => match key.code {
                            KeyCode::Esc
                            | KeyCode::Enter
                            | KeyCode::Char('q')
                            | KeyCode::Char('s') => {
                                app.state = AppState::Browsing;
                            }
                            _ => {}
                        },
                        AppState::Loading => {}
                        _ => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                            _ => {}
                        },
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
