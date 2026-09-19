use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::centered_rect;
use crate::app::RestoreRequest;
use crate::i18n::Translator;

pub fn render(f: &mut Frame, t: &Translator, req: &RestoreRequest, screen_area: Rect) {
    let area = centered_rect(65, 45, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(t.restore_wizard_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .split(inner_area);

    let what_text = if req.paths_to_extract.is_empty() {
        t.restore_all_files().to_string()
    } else {
        t.restore_specific_item_fmt(&req.paths_to_extract[0])
    };

    let origin_content = t.restore_origin_content_fmt(&req.archive_name, &what_text);
    let origin_p = Paragraph::new(origin_content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(t.restore_selected_backup_title()),
    );
    f.render_widget(origin_p, layout[0]);

    let dest_p = Paragraph::new(req.destination_path.as_str())
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.restore_destination_dir_title()),
        );
    f.render_widget(dest_p, layout[1]);

    let prompt = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                t.btn_start_extraction(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(
                t.btn_cancel(),
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ),
        ]),
    ];
    let prompt_p = Paragraph::new(prompt).alignment(Alignment::Center);
    f.render_widget(prompt_p, layout[2]);
}
