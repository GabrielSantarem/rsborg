use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::app::state::AppState;

pub fn handle_archive_info(app: &mut App, key: KeyEvent) {
    if let AppState::ArchiveInfo(ref mut state) = app.state {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('i') => {
                app.state = AppState::Browsing;
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                state.active_tab = (state.active_tab + 1) % 2;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                state.active_tab = if state.active_tab == 0 { 1 } else { 0 };
            }
            KeyCode::Char('1') => {
                state.active_tab = 0;
            }
            KeyCode::Char('2') => {
                state.active_tab = 1;
            }
            _ => {}
        }
    }
}
