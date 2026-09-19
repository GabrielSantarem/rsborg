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

    let repo_path = config::get_default_repo();
    let mut app = App::new(repo_path);
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
                    match app.state {
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
                            _ => {}
                        },
                        AppState::CreatingBackup => {
                            // Atalho universal para Submeter: Ctrl + S
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
                                CreateFocus::Browser => {
                                    match key.code {
                                        KeyCode::Char('j') | KeyCode::Down => {
                                            app.file_browser.next();
                                        }
                                        KeyCode::Char('k') | KeyCode::Up => {
                                            app.file_browser.previous();
                                        }
                                        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                                            // Se estiver sobre o item "..", sobe o diretório; se for pasta normal, entra nela
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
                                    }
                                }
                            }
                        }
                        AppState::ErrorPopup(_) => match key.code {
                            KeyCode::Esc
                            | KeyCode::Enter
                            | KeyCode::Char('q')
                            | KeyCode::Char('s') => {
                                app.state = AppState::CreatingBackup;
                            }
                            _ => {}
                        },
                        AppState::Loading => {
                            // Durante o loading, o app processa em background
                        }
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
