use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::EventOutcome;
use crate::app::state::{PrunePlanState, PrunePolicyState};

pub fn handle_pruning_policy(policy_state: &mut PrunePolicyState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Esc => EventOutcome::ExitModal,
        KeyCode::Tab | KeyCode::Down => {
            policy_state.focus_field = (policy_state.focus_field + 1) % 6;
            EventOutcome::Continue
        }
        KeyCode::Up => {
            if policy_state.focus_field == 0 {
                policy_state.focus_field = 5;
            } else {
                policy_state.focus_field -= 1;
            }
            EventOutcome::Continue
        }
        KeyCode::Char('s') | KeyCode::Enter => EventOutcome::Done,
        KeyCode::Char(c) => {
            match policy_state.focus_field {
                0 if c.is_ascii_digit() => {
                    policy_state.last_str.push(c);
                }
                1 if c.is_ascii_digit() => {
                    policy_state.daily_str.push(c);
                }
                2 if c.is_ascii_digit() => {
                    policy_state.weekly_str.push(c);
                }
                3 if c.is_ascii_digit() => {
                    policy_state.monthly_str.push(c);
                }
                4 if c.is_ascii_digit() => {
                    policy_state.yearly_str.push(c);
                }
                5 => {
                    policy_state.prefix_str.push(c);
                }
                _ => {}
            }
            EventOutcome::Continue
        }
        KeyCode::Backspace => {
            match policy_state.focus_field {
                0 => {
                    policy_state.last_str.pop();
                }
                1 => {
                    policy_state.daily_str.pop();
                }
                2 => {
                    policy_state.weekly_str.pop();
                }
                3 => {
                    policy_state.monthly_str.pop();
                }
                4 => {
                    policy_state.yearly_str.pop();
                }
                5 => {
                    policy_state.prefix_str.pop();
                }
                _ => {}
            }
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}

pub fn handle_prune_plan_view(plan_state: &mut PrunePlanState, key: KeyEvent) -> EventOutcome {
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') => EventOutcome::ExitModal,
        KeyCode::Down | KeyCode::Char('j') => {
            if !plan_state.items.is_empty() {
                plan_state.selected_index =
                    (plan_state.selected_index + 1).min(plan_state.items.len() - 1);
            }
            EventOutcome::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if plan_state.selected_index > 0 {
                plan_state.selected_index -= 1;
            }
            EventOutcome::Continue
        }
        KeyCode::Char('y') | KeyCode::Enter => EventOutcome::Done,
        _ => EventOutcome::Continue,
    }
}
