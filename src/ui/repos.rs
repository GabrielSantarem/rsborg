use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::centered_rect;
use crate::app::App;

pub fn render_manage_repos(f: &mut Frame, app: &App, screen_area: Rect) {
    let area = centered_rect(70, 60, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Gerenciador de Repositórios ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = app
        .config
        .repositories
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let is_selected = i == app.repo_list_index;
            let is_active = repo.id == app.config.active_repo_id;

            let active_badge = if is_active { " [ATIVO] " } else { "         " };
            let enc_text = if repo.passphrase.is_some() {
                "🔒 Criptografado"
            } else {
                "🔓 Sem senha"
            };

            let mut style = Style::default();
            if is_selected {
                style = style.bg(Color::DarkGray).fg(Color::White);
            }
            if is_active {
                style = style.fg(Color::LightGreen).add_modifier(Modifier::BOLD);
            }

            let text = format!(
                "{}{} ({} | {})",
                active_badge, repo.name, repo.location, enc_text
            );
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Repositórios Cadastrados "),
    );
    f.render_widget(list, inner_area);
}

pub fn render_add_repo(f: &mut Frame, app: &App, screen_area: Rect) {
    let area = centered_rect(65, 50, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Adicionar Novo Repositório ")
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
            Constraint::Min(2),
        ])
        .split(inner_area);

    let act = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let inact = Style::default().fg(Color::DarkGray);

    let name_p = Paragraph::new(app.new_repo_name.as_str())
        .style(if app.add_repo_focus == 0 { act } else { inact })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("1. Nome Identificador (ex: HD Externo, Servidor Remoto)"),
        );
    f.render_widget(name_p, layout[0]);

    let loc_p = Paragraph::new(app.new_repo_location.as_str())
        .style(if app.add_repo_focus == 1 { act } else { inact })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("2. Localização (ex: /run/media/... ou ssh://user@host/repo)"),
        );
    f.render_widget(loc_p, layout[1]);

    let pass_display = "*".repeat(app.new_repo_passphrase.len());
    let pass_p = Paragraph::new(pass_display.as_str())
        .style(if app.add_repo_focus == 2 { act } else { inact })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("3. Senha / Passphrase (Opcional - deixe vazio se não tiver)"),
        );
    f.render_widget(pass_p, layout[2]);
}
