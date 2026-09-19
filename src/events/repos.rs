use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::app::state::AppState;
use crate::config;

pub fn handle_managing_repos(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') if !app.config.repositories.is_empty() => {
            app.repo_list_index =
                (app.repo_list_index + 1).min(app.config.repositories.len().saturating_sub(1));
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.repo_list_index = app.repo_list_index.saturating_sub(1);
        }
        KeyCode::Enter => {
            if let Some(repo) = app.config.repositories.get(app.repo_list_index) {
                let repo_id = repo.id.clone();
                app.switch_active_repo(repo_id);
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
            if app.config.repositories.len() > 1
                && app.repo_list_index < app.config.repositories.len() =>
        {
            let removed = app.config.repositories.remove(app.repo_list_index);
            if app.repo_list_index >= app.config.repositories.len() {
                app.repo_list_index = app.config.repositories.len().saturating_sub(1);
            }
            if app.config.active_repo_id == removed.id
                && let Some(first) = app.config.repositories.first()
            {
                app.config.active_repo_id = first.id.clone();
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

pub fn handle_adding_repo(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.state = AppState::ManagingRepos;
        }
        KeyCode::Tab => {
            app.add_repo_focus = (app.add_repo_focus + 1) % 3;
        }
        KeyCode::Enter => {
            app.save_new_repository();
        }
        KeyCode::Char(c) => match app.add_repo_focus {
            0 => app.new_repo_name.push(c),
            1 => app.new_repo_location.push(c),
            2 => app.new_repo_passphrase.push(c),
            _ => {}
        },
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
        _ => {}
    }
}
