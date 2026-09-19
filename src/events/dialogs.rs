use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::EventOutcome;
use crate::app::App;
use crate::app::state::{AppState, InspectState, RestoreRequest};

pub fn handle_confirm_delete(app: &mut App, key: KeyEvent, target_name: String) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.confirm_delete_archive(target_name);
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.state = AppState::Browsing;
        }
        _ => {}
    }
}

pub fn handle_confirm_restore(req: &mut RestoreRequest, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Enter => EventOutcome::Done,
        KeyCode::Esc => EventOutcome::ExitModal,
        KeyCode::Backspace => {
            req.destination_path.pop();
            EventOutcome::Continue
        }
        KeyCode::Char(c) => {
            req.destination_path.push(c);
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}

pub fn handle_inspect_archive(inspect: &mut InspectState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => EventOutcome::ExitModal,
        KeyCode::Down | KeyCode::Char('j') => {
            if !inspect.entries.is_empty() {
                inspect.selected_index =
                    (inspect.selected_index + 1).min(inspect.entries.len() - 1);
            }
            EventOutcome::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if inspect.selected_index > 0 {
                inspect.selected_index -= 1;
            }
            EventOutcome::Continue
        }
        KeyCode::PageDown => {
            if !inspect.entries.is_empty() {
                inspect.selected_index =
                    (inspect.selected_index + 15).min(inspect.entries.len() - 1);
            }
            EventOutcome::Continue
        }
        KeyCode::PageUp => {
            inspect.selected_index = inspect.selected_index.saturating_sub(15);
            EventOutcome::Continue
        }
        KeyCode::Char('x') => EventOutcome::Done,
        _ => EventOutcome::Continue,
    }
}
