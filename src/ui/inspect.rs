use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Row, Table, TableState},
};

use super::centered_rect;
use crate::app::InspectState;

pub fn render(f: &mut Frame, inspect: &InspectState, screen_area: Rect) {
    let area = centered_rect(85, 85, screen_area);
    f.render_widget(Clear, area);

    let title = format!(
        " Conteúdo do Backup: {} ({} itens) - [x] Restaurar Item Selecionado ",
        inspect.archive_name,
        inspect.entries.len()
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let rows = inspect.entries.iter().map(|entry| {
        let icon = if entry.entry_type == "d" {
            "📁 "
        } else {
            "📄 "
        };
        let size_kb = format!("{:.1} KB", entry.size as f32 / 1024.0);
        Row::new(vec![
            Cell::from(entry.mode.clone()),
            Cell::from(size_kb),
            Cell::from(format!("{}{}", icon, entry.path)),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec![
            Cell::from("Permissões").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Tamanho").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Caminho do Arquivo").style(Style::default().add_modifier(Modifier::BOLD)),
        ])
        .bottom_margin(1),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let mut t_state = TableState::default();
    t_state.select(Some(inspect.selected_index));
    f.render_stateful_widget(table, inner_area, &mut t_state);
}
