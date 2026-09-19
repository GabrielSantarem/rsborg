use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::centered_rect;
use crate::app::{App, CreateFocus};
use crate::browser::ItemStatus;

pub fn render(f: &mut Frame, app: &mut App, screen_area: Rect) {
    let area = centered_rect(85, 85, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Assistente de Backup / Backup Wizard ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(10),
        ])
        .split(inner_area);

    let (n_sty, b_sty) = match app.create_focus {
        CreateFocus::Name => (
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(Color::DarkGray),
        ),
        CreateFocus::Browser => (
            Style::default().fg(Color::DarkGray),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let n = Paragraph::new(app.new_backup_name.as_str())
        .style(n_sty)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("1. Nome do Backup (digite e aperte Tab para ir aos arquivos)"),
        );
    f.render_widget(n, input_chunks[0]);

    let legend = Line::from(vec![
        Span::styled("Legenda: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            "[+] Incluído ",
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("[✓] Herdado ", Style::default().fg(Color::Cyan)),
        Span::styled(
            "[-] Excluído ",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("[ ] Não selecionado ", Style::default().fg(Color::Gray)),
    ]);
    let legend_p = Paragraph::new(legend);
    f.render_widget(legend_p, input_chunks[1]);

    let browser_h = app.file_browser.current_dir.to_string_lossy();
    let b_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " 2. Selecionar Arquivos / Pastas  [{}] ",
            browser_h
        ))
        .style(b_sty);

    let items: Vec<ListItem> = app
        .file_browser
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected =
                i == app.file_browser.selected_index && app.create_focus == CreateFocus::Browser;

            let icon = if entry.is_dir { "📁" } else { "📄" };

            let (prefix, status_text, color) = if entry.is_parent_link {
                ("[ ⮥ ]", "", Color::Yellow)
            } else {
                match app.file_browser.get_status(&entry.path) {
                    ItemStatus::ExplicitInclude => ("[ + ]", " (Incluído)", Color::LightGreen),
                    ItemStatus::InheritedInclude => ("[ ✓ ]", " (Herdado do pai)", Color::Cyan),
                    ItemStatus::ExplicitExclude => ("[ - ]", " (Excluído)", Color::LightRed),
                    ItemStatus::InheritedExclude => ("[ x ]", " (Pai excluído)", Color::DarkGray),
                    ItemStatus::Neutral => ("[   ]", "", Color::White),
                }
            };

            let mut item_style = Style::default().fg(color);
            if is_selected {
                item_style = item_style
                    .bg(Color::Rgb(40, 40, 40))
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
            }

            let line = Line::from(vec![
                Span::styled(format!("{} {} ", prefix, icon), item_style),
                Span::styled(entry.name.clone(), item_style),
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(b_block);
    f.render_widget(list, input_chunks[2]);
}
