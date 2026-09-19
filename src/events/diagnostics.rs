use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::EventOutcome;
use crate::app::state::{CheckResultState, CheckWizardState, DiffViewState, DiffWizardState};
use crate::borg::BorgCheckMode;

pub fn handle_check_wizard(wizard_state: &mut CheckWizardState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Esc => EventOutcome::ExitModal,
        KeyCode::Tab => {
            if wizard_state.target_archive.is_some() {
                wizard_state.check_archive_only = !wizard_state.check_archive_only;
            }
            EventOutcome::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            wizard_state.check_mode = match wizard_state.check_mode {
                BorgCheckMode::RepositoryOnly => BorgCheckMode::Standard,
                BorgCheckMode::Standard => BorgCheckMode::VerifyData,
                BorgCheckMode::VerifyData => BorgCheckMode::Repair,
                BorgCheckMode::Repair => BorgCheckMode::RepositoryOnly,
            };
            EventOutcome::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            wizard_state.check_mode = match wizard_state.check_mode {
                BorgCheckMode::RepositoryOnly => BorgCheckMode::Repair,
                BorgCheckMode::Standard => BorgCheckMode::RepositoryOnly,
                BorgCheckMode::VerifyData => BorgCheckMode::Standard,
                BorgCheckMode::Repair => BorgCheckMode::VerifyData,
            };
            EventOutcome::Continue
        }
        KeyCode::Enter => EventOutcome::Done,
        _ => EventOutcome::Continue,
    }
}

pub fn handle_check_result_view(
    result_state: &mut CheckResultState,
    key: KeyEvent,
) -> EventOutcome {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => EventOutcome::ExitModal,
        KeyCode::Down | KeyCode::Char('j') => {
            if !result_state.result.log_output.is_empty() {
                result_state.log_scroll = (result_state.log_scroll + 1)
                    .min(result_state.result.log_output.len().saturating_sub(1));
            }
            EventOutcome::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            result_state.log_scroll = result_state.log_scroll.saturating_sub(1);
            EventOutcome::Continue
        }
        KeyCode::PageDown => {
            if !result_state.result.log_output.is_empty() {
                result_state.log_scroll = (result_state.log_scroll + 15)
                    .min(result_state.result.log_output.len().saturating_sub(1));
            }
            EventOutcome::Continue
        }
        KeyCode::PageUp => {
            result_state.log_scroll = result_state.log_scroll.saturating_sub(15);
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}

pub fn handle_diff_wizard(wizard_state: &mut DiffWizardState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Esc => EventOutcome::ExitModal,
        KeyCode::Down | KeyCode::Char('j') => {
            if !wizard_state.candidates.is_empty() {
                wizard_state.selected_candidate_idx = (wizard_state.selected_candidate_idx + 1)
                    .min(wizard_state.candidates.len() - 1);
            }
            EventOutcome::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            wizard_state.selected_candidate_idx =
                wizard_state.selected_candidate_idx.saturating_sub(1);
            EventOutcome::Continue
        }
        KeyCode::Tab => {
            wizard_state.content_only = !wizard_state.content_only;
            EventOutcome::Continue
        }
        KeyCode::Enter => EventOutcome::Done,
        _ => EventOutcome::Continue,
    }
}

pub fn handle_diff_view(view_state: &mut DiffViewState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => EventOutcome::ExitModal,
        KeyCode::Down | KeyCode::Char('j') => {
            if !view_state.entries.is_empty() {
                view_state.selected_index =
                    (view_state.selected_index + 1).min(view_state.entries.len() - 1);
            }
            EventOutcome::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            view_state.selected_index = view_state.selected_index.saturating_sub(1);
            EventOutcome::Continue
        }
        KeyCode::PageDown => {
            if !view_state.entries.is_empty() {
                view_state.selected_index =
                    (view_state.selected_index + 15).min(view_state.entries.len() - 1);
            }
            EventOutcome::Continue
        }
        KeyCode::PageUp => {
            view_state.selected_index = view_state.selected_index.saturating_sub(15);
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}
