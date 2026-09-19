use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};

use super::centered_rect;
use crate::app::{PrunePlanState, PrunePolicyState};
use crate::i18n::Translator;

pub fn render_policy_modal(
    f: &mut Frame,
    t: &Translator,
    policy_state: &PrunePolicyState,
    screen_area: Rect,
) {
    let area = centered_rect(70, 75, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(t.prune_policy_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .split(inner_area);

    let act = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let inact = Style::default().fg(Color::DarkGray);

    let fields = [
        (t.prune_keep_last(), policy_state.last_str.as_str(), 0),
        (t.prune_keep_daily(), policy_state.daily_str.as_str(), 1),
        (t.prune_keep_weekly(), policy_state.weekly_str.as_str(), 2),
        (t.prune_keep_monthly(), policy_state.monthly_str.as_str(), 3),
        (t.prune_keep_yearly(), policy_state.yearly_str.as_str(), 4),
        (t.prune_prefix(), policy_state.prefix_str.as_str(), 5),
    ];

    for (title, val, idx) in fields {
        let sty = if policy_state.focus_field == idx {
            act
        } else {
            inact
        };
        let p = Paragraph::new(val)
            .style(sty)
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(p, layout[idx]);
    }

    let hint = Paragraph::new(t.prune_hint()).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::ITALIC),
    );
    f.render_widget(hint, layout[6]);

    let prompt = Paragraph::new(t.prune_prompt_button())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(prompt, layout[7]);
}

pub fn render_plan_view(
    f: &mut Frame,
    t: &Translator,
    plan_state: &PrunePlanState,
    screen_area: Rect,
) {
    let area = centered_rect(85, 85, screen_area);
    f.render_widget(Clear, area);

    let to_keep = plan_state.items.iter().filter(|i| i.will_keep).count();
    let to_prune = plan_state.items.iter().filter(|i| !i.will_keep).count();

    let block = Block::default()
        .title(t.plan_title_fmt(to_keep, to_prune))
        .borders(Borders::ALL)
        .style(Style::default().fg(if to_prune > 0 {
            Color::Red
        } else {
            Color::Green
        }));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(inner_area);

    let summary = Line::from(vec![
        Span::styled(
            t.plan_total_evaluated_label(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}  |  ", plan_state.items.len())),
        Span::styled(
            t.plan_kept_fmt(to_keep),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            t.plan_to_prune_fmt(to_prune),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(summary), layout[0]);

    let rows = plan_state.items.iter().map(|item| {
        let (status_text, style) = if item.will_keep {
            (t.badge_keep(), Style::default().fg(Color::Green))
        } else {
            (
                t.badge_prune(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        };

        Row::new(vec![
            Cell::from(status_text).style(style),
            Cell::from(item.name.clone()),
            Cell::from(item.date_info.clone()),
            Cell::from(item.rule_info.clone()).style(Style::default().fg(Color::DarkGray)),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Percentage(40),
            Constraint::Length(26),
            Constraint::Percentage(30),
        ],
    )
    .header(
        Row::new(vec![
            Cell::from(t.col_action()).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(t.col_name()).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(t.col_date()).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(t.col_applied_rule()).style(Style::default().add_modifier(Modifier::BOLD)),
        ])
        .bottom_margin(1),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let mut t_state = TableState::default();
    t_state.select(Some(plan_state.selected_index));
    f.render_stateful_widget(table, layout[1], &mut t_state);

    let action_bar = if to_prune > 0 {
        Paragraph::new(vec![Line::from(vec![
            Span::styled(
                t.plan_confirm_button(),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                t.btn_cancel(),
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ),
        ])])
        .alignment(Alignment::Center)
    } else {
        Paragraph::new(t.plan_none_to_prune())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightGreen))
    };
    f.render_widget(action_bar, layout[2]);
}
