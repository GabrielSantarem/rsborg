use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::app::state::{AppState, LogFilterLevel};

pub fn handle_log_viewer(app: &mut App, key: KeyEvent) {
    if let AppState::LogViewer(ref mut state) = app.state {
        // Approximate visible height for scrolling calculations (safe fallback)
        let visible_height = 25;

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('L') | KeyCode::Char('o') => {
                app.state = AppState::Browsing;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.scroll_down(visible_height);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.scroll_up();
            }
            KeyCode::PageDown => {
                state.scroll_page_down(10, visible_height);
            }
            KeyCode::PageUp => {
                state.scroll_page_up(10);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.scroll_page_down(10, visible_height);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.scroll_page_up(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                state.scroll_top();
            }
            KeyCode::End | KeyCode::Char('G') => {
                state.scroll_bottom(visible_height);
            }
            KeyCode::Char('1') => {
                state.set_filter(LogFilterLevel::All);
            }
            KeyCode::Char('2') => {
                state.set_filter(LogFilterLevel::Info);
            }
            KeyCode::Char('3') => {
                state.set_filter(LogFilterLevel::Warn);
            }
            KeyCode::Char('4') => {
                state.set_filter(LogFilterLevel::Error);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                let lines = crate::logger::read_log_entries();
                state.all_lines = lines;
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                let _ = crate::logger::clear_log_file();
                state.all_lines = crate::logger::read_log_entries();
                state.scroll = 0;
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                app.toggle_theme();
            }
            KeyCode::Char('l') => {
                app.toggle_language();
            }
            _ => {}
        }
    }
}
