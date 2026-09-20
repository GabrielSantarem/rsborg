use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::centered_rect;
use crate::app::App;
use crate::app::state::{LogFilterLevel, LogViewerState};

pub fn render_log_viewer(f: &mut Frame, app: &App, state: &LogViewerState, screen_area: Rect) {
    let area = centered_rect(90, 85, screen_area);
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filters & Info Header
            Constraint::Min(5),    // Log Content
            Constraint::Length(3), // Footer / Shortcuts
        ])
        .split(area);

    // 1. Header with Filters and Counts
    let all_count = state.all_lines.len();
    let info_count = state
        .all_lines
        .iter()
        .filter(|l| l.contains("[INFO]"))
        .count();
    let warn_count = state
        .all_lines
        .iter()
        .filter(|l| l.contains("[WARN]"))
        .count();
    let error_count = state
        .all_lines
        .iter()
        .filter(|l| l.contains("[ERROR]"))
        .count();

    let tab_style = |active: bool| -> Style {
        if active {
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(app.theme.text_muted)
        }
    };

    let filter_line = Line::from(vec![
        Span::raw(" Filtros: "),
        Span::styled(
            format!(" [1] {} ({}) ", app.t.logs_filter_all(), all_count),
            tab_style(state.filter == LogFilterLevel::All),
        ),
        Span::raw(" "),
        Span::styled(
            format!(" [2] {} ({}) ", app.t.logs_filter_info(), info_count),
            tab_style(state.filter == LogFilterLevel::Info),
        ),
        Span::raw(" "),
        Span::styled(
            format!(" [3] {} ({}) ", app.t.logs_filter_warn(), warn_count),
            tab_style(state.filter == LogFilterLevel::Warn),
        ),
        Span::raw(" "),
        Span::styled(
            format!(" [4] {} ({}) ", app.t.logs_filter_error(), error_count),
            tab_style(state.filter == LogFilterLevel::Error),
        ),
    ]);

    let header_block = Block::default()
        .borders(Borders::ALL)
        .title(app.t.logs_title())
        .title_alignment(Alignment::Center)
        .style(app.theme.block_active());

    f.render_widget(Paragraph::new(filter_line).block(header_block), chunks[0]);

    // 2. Log Content Body
    let filtered_indices = state.filtered_indices();
    let total_filtered = filtered_indices.len();
    let inner_height = chunks[1].height.saturating_sub(2) as usize;

    let start_idx = state.scroll.min(total_filtered);
    let end_idx = (start_idx + inner_height).min(total_filtered);

    let visible_indices = if total_filtered > 0 {
        &filtered_indices[start_idx..end_idx]
    } else {
        &[]
    };

    let mut log_spans: Vec<Line> = Vec::new();

    if total_filtered == 0 {
        log_spans.push(Line::from(""));
        log_spans.push(Line::from(vec![Span::styled(
            format!("   {}", app.t.logs_empty()),
            Style::default().fg(app.theme.text_muted),
        )]));
    } else {
        for &line_idx in visible_indices {
            if let Some(raw_line) = state.all_lines.get(line_idx) {
                log_spans.push(format_log_line(raw_line, app));
            }
        }
    }

    let line_info = if total_filtered > 0 {
        format!(
            " Linhas {}-{} de {} ",
            start_idx + 1,
            end_idx,
            total_filtered
        )
    } else {
        " 0 linhas ".to_string()
    };

    let content_block = Block::default()
        .borders(Borders::ALL)
        .title(line_info)
        .title_alignment(Alignment::Right)
        .style(app.theme.block_normal());

    f.render_widget(Paragraph::new(log_spans).block(content_block), chunks[1]);

    // 3. Footer Bar
    let footer_p = Paragraph::new(app.t.logs_footer())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(app.theme.block_normal()),
        );
    f.render_widget(footer_p, chunks[2]);
}

fn format_log_line<'a>(line: &'a str, app: &'a App) -> Line<'a> {
    // Expected format: [2026-09-19 22:45:00.123] [LEVEL] message
    if line.starts_with('[') && line.len() > 30
        && let Some(first_bracket_end) = line.find(']')
    {
        let timestamp_str = &line[0..=first_bracket_end];
        let rest = line[first_bracket_end + 1..].trim_start();

        if rest.starts_with('[')
            && let Some(second_bracket_end) = rest.find(']')
        {
            let level_str = &rest[0..=second_bracket_end];
            let msg = rest[second_bracket_end + 1..].trim_start();

                    let level_style = if level_str.contains("ERROR") {
                        Style::default()
                            .fg(app.theme.danger)
                            .add_modifier(Modifier::BOLD)
                    } else if level_str.contains("WARN") {
                        Style::default()
                            .fg(app.theme.warning)
                            .add_modifier(Modifier::BOLD)
                    } else if level_str.contains("DEBUG") {
                        Style::default().fg(app.theme.secondary)
                    } else {
                        Style::default().fg(app.theme.success)
                    };

                    return Line::from(vec![
                        Span::styled(timestamp_str, Style::default().fg(app.theme.text_muted)),
                        Span::raw(" "),
                        Span::styled(level_str, level_style),
                        Span::raw(" "),
                        Span::styled(msg, Style::default().fg(Color::White)),
                    ]);
        }
    }

    // Fallback for wrapped details, stack traces, or command output
    let fallback_style =
        if line.contains("Error") || line.contains("failed") || line.contains("Exception") {
            Style::default().fg(app.theme.danger)
        } else {
            Style::default().fg(app.theme.text_muted)
        };

    Line::from(vec![Span::raw("   "), Span::styled(line, fallback_style)])
}
