use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::app::state::AppState;

pub const SETTINGS_ITEMS_COUNT: usize = 6;

pub fn handle_settings(app: &mut App, key: KeyEvent) {
    if let AppState::Settings(ref mut state) = app.state {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('s') | KeyCode::Char('S') => {
                app.state = AppState::Browsing;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.selected_index = (state.selected_index + 1) % SETTINGS_ITEMS_COUNT;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.selected_index = if state.selected_index == 0 {
                    SETTINGS_ITEMS_COUNT - 1
                } else {
                    state.selected_index - 1
                };
            }
            KeyCode::Enter | KeyCode::Char(' ') => match state.selected_index {
                0 => app.toggle_language(),
                1 => app.toggle_theme(),
                2 => app.state = AppState::ManagingRepos,
                3 => app.open_log_viewer(),
                _ => {}
            },
            KeyCode::Char('l') => {
                app.toggle_language();
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                app.toggle_theme();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                app.state = AppState::ManagingRepos;
            }
            KeyCode::Char('L') | KeyCode::Char('o') => {
                app.open_log_viewer();
            }
            _ => {}
        }
    }
}
