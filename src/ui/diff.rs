use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState,
    },
};

use super::centered_rect;
use crate::app::{DiffViewState, DiffWizardState};
use crate::borg::DiffKind;
use crate::i18n::Translator;

pub fn render_diff_wizard(
    f: &mut Frame,
    t: &Translator,
    state: &DiffWizardState,
    screen_area: Rect,
) {
    let area = centered_rect(70, 60, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(t.diff_wizard_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner_area);

    // 1. Base Archive (Source)
    let base_text = format!("  ➔ {}", state.base_archive);
    let base_p = Paragraph::new(base_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(t.diff_base_title()),
    );
    f.render_widget(base_p, layout[0]);

    // 2. Candidate Archives to compare against (Stateful with scroll)
    let items: Vec<ListItem> = state
        .candidates
        .iter()
        .map(|name| ListItem::new(format!("  {}", name)).style(Style::default().fg(Color::White)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.diff_target_title()),
        )
        .highlight_symbol("➔ ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let mut l_state = ListState::default();
    l_state.select(Some(state.selected_candidate_idx));
    f.render_stateful_widget(list, layout[1], &mut l_state);

    // 3. Content-only option
    let opt_text = state.content_only;
    let opt_line = Line::from(vec![Span::styled(
        t.diff_option_content_only(opt_text),
        if opt_text {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        },
    )]);
    let opt_p = Paragraph::new(opt_line).block(Block::default().borders(Borders::NONE));
    f.render_widget(opt_p, layout[2]);

    // 4. Action Prompt
    let prompt = Paragraph::new(t.diff_start_prompt())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(prompt, layout[3]);
}

pub fn render_diff_view(f: &mut Frame, t: &Translator, state: &DiffViewState, screen_area: Rect) {
    let area = centered_rect(90, 85, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(t.diff_view_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(inner_area);

    // 1. Header & Counts Summary
    let mut added = 0;
    let mut removed = 0;
    let mut modified = 0;
    let mut metadata = 0;

    for entry in &state.entries {
        match entry.kind() {
            DiffKind::Added => added += 1,
            DiffKind::Removed => removed += 1,
            DiffKind::Modified => modified += 1,
            DiffKind::Metadata => metadata += 1,
        }
    }

    let summary_line = Line::from(vec![
        Span::styled(
            format!(" {} ➔ {} ", state.archive1, state.archive2),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            t.diff_summary_fmt(added, removed, modified, metadata),
            Style::default().fg(Color::White),
        ),
    ]);
    let header_p = Paragraph::new(summary_line).block(Block::default().borders(Borders::ALL));
    f.render_widget(header_p, layout[0]);

    // 2. Main Differences Table (Stateful with auto-scroll)
    if state.entries.is_empty() {
        let empty_p = Paragraph::new(t.diff_no_changes())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(empty_p, layout[1]);
    } else {
        let rows: Vec<Row> = state
            .entries
            .iter()
            .map(|entry| {
                let (badge, badge_color) = match entry.kind() {
                    DiffKind::Added => (t.diff_badge_added(), Color::LightGreen),
                    DiffKind::Removed => (t.diff_badge_removed(), Color::LightRed),
                    DiffKind::Modified => (t.diff_badge_modified(), Color::Yellow),
                    DiffKind::Metadata => (t.diff_badge_metadata(), Color::Cyan),
                };

                Row::new(vec![
                    Span::styled(badge, Style::default().fg(badge_color)),
                    Span::raw(entry.formatted_change()),
                    Span::raw(&entry.path),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(16),
            Constraint::Length(22),
            Constraint::Min(20),
        ];

        let table = Table::new(rows, widths)
            .header(
                Row::new(vec![
                    t.diff_col_type(),
                    t.diff_col_change(),
                    t.diff_col_path(),
                ])
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .row_highlight_style(
                Style::default()
                    .bg(Color::Rgb(30, 45, 70))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
            .block(Block::default().borders(Borders::ALL));

        let mut t_state = TableState::default();
        t_state.select(Some(state.selected_index));
        f.render_stateful_widget(table, layout[1], &mut t_state);
    }

    // 3. Detailed Change Panel for currently selected item
    let detail_text = if let Some(entry) = state.entries.get(state.selected_index) {
        format!(
            "Caminho: {}\nAlteração: {}",
            entry.path,
            entry.formatted_change()
        )
    } else {
        String::new()
    };

    let detail_p = Paragraph::new(detail_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Detalhes da Seleção"),
    );
    f.render_widget(detail_p, layout[2]);
}
