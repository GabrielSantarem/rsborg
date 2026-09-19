use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::centered_rect;
use crate::app::RestoreRequest;

pub fn render(f: &mut Frame, req: &RestoreRequest, screen_area: Rect) {
    let area = centered_rect(65, 45, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Assistente de Restauração / Extract ")
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
        "Restauração Total (Todos os arquivos do backup)".to_string()
    } else {
        format!("Item específico: {}", req.paths_to_extract[0])
    };

    let origin_p = Paragraph::new(format!(
        "Origem: {}\nConteúdo: {}",
        req.archive_name, what_text
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Backup Selecionado"),
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
                .title("Diretório de Destino (digite para alterar)"),
        );
    f.render_widget(dest_p, layout[1]);

    let prompt = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [Enter] Iniciar Extração ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(
                " [Esc] Cancelar ",
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ),
        ]),
    ];
    let prompt_p = Paragraph::new(prompt).alignment(Alignment::Center);
    f.render_widget(prompt_p, layout[2]);
}
