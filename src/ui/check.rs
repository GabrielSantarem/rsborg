use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::centered_rect;
use crate::app::{CheckResultState, CheckWizardState};
use crate::borg::BorgCheckMode;
use crate::i18n::Translator;

pub fn render_check_wizard(
    f: &mut Frame,
    t: &Translator,
    state: &CheckWizardState,
    screen_area: Rect,
) {
    let area = centered_rect(75, 60, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(t.check_wizard_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(inner_area);

    // 1. Target Scope Section
    let target_entire = if !state.check_archive_only {
        format!("(•) {}", t.check_target_entire_repo())
    } else {
        format!("( ) {}", t.check_target_entire_repo())
    };

    let target_archive = match state.target_archive {
        Some(ref name) => {
            if state.check_archive_only {
                format!("(•) {}", t.check_target_archive_fmt(name))
            } else {
                format!("( ) {}", t.check_target_archive_fmt(name))
            }
        }
        None => format!("( ) {}", t.no_backup_selected()),
    };

    let target_lines = vec![
        Line::from(vec![Span::styled(
            target_entire,
            if !state.check_archive_only {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        )]),
        Line::from(vec![Span::styled(
            target_archive,
            if state.check_archive_only {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        )]),
    ];

    let target_p = Paragraph::new(target_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(t.check_target_title()),
    );
    f.render_widget(target_p, layout[0]);

    // 2. Mode Selection Section
    let modes = [
        (BorgCheckMode::RepositoryOnly, t.check_mode_quick()),
        (BorgCheckMode::Standard, t.check_mode_standard()),
        (BorgCheckMode::VerifyData, t.check_mode_verify_data()),
        (BorgCheckMode::Repair, t.check_mode_repair()),
    ];

    let items: Vec<ListItem> = modes
        .iter()
        .map(|(mode, desc)| {
            let is_selected = state.check_mode == *mode;
            let bullet = if is_selected { "(•) " } else { "( ) " };
            let style = if is_selected {
                if *mode == BorgCheckMode::Repair {
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(format!("{}{}", bullet, desc)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(t.check_mode_title()),
    );
    f.render_widget(list, layout[1]);

    // 3. Action Prompt
    let prompt = Paragraph::new(t.check_start_prompt())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(prompt, layout[2]);
}

pub fn render_check_result(
    f: &mut Frame,
    t: &Translator,
    state: &CheckResultState,
    screen_area: Rect,
) {
    let area = centered_rect(85, 85, screen_area);
    f.render_widget(Clear, area);

    let (badge_text, border_color) = if state.result.success {
        (t.check_status_healthy(), Color::LightGreen)
    } else if state.result.warnings {
        (t.check_status_warning(), Color::Yellow)
    } else {
        (t.check_status_corrupted(), Color::Red)
    };

    let block = Block::default()
        .title(t.check_result_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(border_color));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(8),
        ])
        .split(inner_area);

    // 1. Status Banner
    let banner = Paragraph::new(Line::from(vec![Span::styled(
        badge_text,
        Style::default()
            .fg(border_color)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(banner, layout[0]);

    // 2. Metadata line
    let meta = Line::from(vec![
        Span::styled("Alvo: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}  |  ", state.target_display)),
        Span::styled("Modo: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}  |  ", state.mode_display)),
        Span::styled("Linhas: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}", state.result.log_output.len())),
    ]);
    f.render_widget(Paragraph::new(meta), layout[1]);

    // 3. Log Output View with Scrolling
    let visible_height = layout[2].height.saturating_sub(2) as usize;
    let total_lines = state.result.log_output.len();

    let start_idx = state.log_scroll.min(total_lines.saturating_sub(1));
    let end_idx = (start_idx + visible_height).min(total_lines);

    let log_items: Vec<ListItem> = if total_lines == 0 {
        vec![ListItem::new("Nenhuma saída registrada do processo.")]
    } else {
        state.result.log_output[start_idx..end_idx]
            .iter()
            .map(|l| {
                let style = if l.contains("ERROR") || l.contains("Error") {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else if l.contains("WARNING") || l.contains("Warning") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(l.as_str()).style(style)
            })
            .collect()
    };

    let log_title = format!(
        "{} (Linhas {}-{} de {})",
        t.check_logs_header(),
        start_idx + 1,
        end_idx,
        total_lines
    );
    let log_list =
        List::new(log_items).block(Block::default().borders(Borders::ALL).title(log_title));
    f.render_widget(log_list, layout[2]);
}
