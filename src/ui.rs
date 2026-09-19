use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{App, AppState, CreateFocus};
use crate::browser::ItemStatus;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(size);

    let repo_loc = app
        .repository
        .as_ref()
        .map(|r| r.location.as_str())
        .unwrap_or("");
    let version_str = app.borg_version.as_deref().unwrap_or("?");
    let header_text = vec![Line::from(vec![
        Span::styled(
            format!(" {} ", app.t.title()),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "| {} | {}: {} ",
            version_str,
            app.t.header_repo(),
            repo_loc
        )),
    ])];

    let header = Paragraph::new(header_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    match app.state {
        AppState::Initializing => {
            let p = Paragraph::new("...").block(Block::default().borders(Borders::ALL));
            f.render_widget(p, chunks[1]);
        }
        AppState::InitError(ref err) => {
            let p = Paragraph::new(format!("Error:\n\n{}", err))
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(p, chunks[1]);
        }
        AppState::Browsing
        | AppState::CreatingBackup
        | AppState::Loading
        | AppState::ErrorPopup(_) => {
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(chunks[1]);

            let selected_style = Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Green);
            let normal_style = Style::default().bg(Color::Reset);

            let n_col = app.t.col_name();
            let s_col = app.t.col_start();
            let cells = vec![
                Cell::from(n_col).style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from(s_col).style(Style::default().add_modifier(Modifier::BOLD)),
            ];
            let header_row = Row::new(cells).height(1).bottom_margin(1);

            let rows = app.archives.iter().map(|a| {
                let r_cells = vec![Cell::from(a.name.clone()), Cell::from(a.start.clone())];
                Row::new(r_cells).style(normal_style).height(1)
            });

            let table = Table::new(
                rows,
                [Constraint::Percentage(50), Constraint::Percentage(50)],
            )
            .header(header_row)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(app.t.table_title()),
            )
            .row_highlight_style(selected_style)
            .highlight_symbol(">> ");
            f.render_stateful_widget(table, body_chunks[0], &mut app.table_state);

            if let Some(i) = app.table_state.selected() {
                if let Some(archive) = app.archives.get(i) {
                    let md_text = format!(
                        "ID: {}\nNome: {}\nArquivo Original: {}\n\nData de Criação: {}\nFinalizado em: {}",
                        archive.id, archive.name, archive.archive, archive.start, archive.time
                    );
                    let p = Paragraph::new(md_text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(app.t.metadata_title()),
                    );
                    f.render_widget(p, body_chunks[1]);
                }
            } else {
                let p = Paragraph::new("").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(app.t.metadata_title()),
                );
                f.render_widget(p, body_chunks[1]);
            }
        }
    }

    let footer_text = match app.state {
        AppState::Browsing => app.t.footer_main(),
        AppState::CreatingBackup => {
            " [Tab] Alternar Foco | [Espaço] Incluir/Excluir | [Enter/→] Entrar | [BS/←] Subir | [s / Ctrl+S] Criar | [Esc] Cancelar"
        }
        AppState::ErrorPopup(_) => " [Esc/Enter] Voltar ",
        AppState::Loading => " Executando Borg em segundo plano... ",
        _ => " [Esc] Sair ",
    };
    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);

    if app.state == AppState::CreatingBackup {
        let area = centered_rect(85, 85, size);
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
                Constraint::Length(2), // Barra de ajuda de status
                Constraint::Min(8),
            ])
            .split(inner_area);

        let act = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let inact = Style::default().fg(Color::DarkGray);

        let n_sty = if app.create_focus == CreateFocus::Name {
            act
        } else {
            inact
        };
        let b_sty = if app.create_focus == CreateFocus::Browser {
            act
        } else {
            inact
        };

        let n = Paragraph::new(app.new_backup_name.as_str())
            .style(n_sty)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("1. Nome do Backup (digite e aperte Tab para ir aos arquivos)"),
            );
        f.render_widget(n, input_chunks[0]);

        // Legenda explicativa
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

        // File Browser
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
                let is_selected = i == app.file_browser.selected_index
                    && app.create_focus == CreateFocus::Browser;

                let icon = if entry.is_dir { "📁" } else { "📄" };

                let (prefix, status_text, color) = if entry.is_parent_link {
                    ("[ ⮥ ]", "", Color::Yellow)
                } else {
                    match app.file_browser.get_status(&entry.path) {
                        ItemStatus::ExplicitInclude => ("[ + ]", " (Incluído)", Color::LightGreen),
                        ItemStatus::InheritedInclude => ("[ ✓ ]", " (Herdado do pai)", Color::Cyan),
                        ItemStatus::ExplicitExclude => ("[ - ]", " (Excluído)", Color::LightRed),
                        ItemStatus::InheritedExclude => {
                            ("[ x ]", " (Pai excluído)", Color::DarkGray)
                        }
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

    if let AppState::ErrorPopup(ref msg) = app.state {
        let area = centered_rect(55, 30, size);
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(" Aviso / Warning ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Red));
        let p = Paragraph::new(msg.as_str())
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center)
            .block(block)
            .style(Style::default().fg(Color::Red));
        f.render_widget(p, area);
    }

    if app.state == AppState::Loading {
        let area = centered_rect(65, 45, size);
        f.render_widget(Clear, area);

        let spinner = SPINNER_FRAMES[app.loading_info.spinner_frame % SPINNER_FRAMES.len()];
        let title = format!(" [ {} ] {} ", spinner, app.loading_info.message);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Espaço
                Constraint::Length(2), // Activity bar
                Constraint::Length(1), // Espaço
                Constraint::Min(6),    // Details
            ])
            .split(inner_area);

        // Barra de atividade animada (pulse)
        let pulse_percent = ((app.loading_info.elapsed_secs * 15
            + app.loading_info.spinner_frame as u64 * 5)
            % 100) as u16;
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .percent(pulse_percent)
            .label(format!(
                "Atividade Borg: {}s",
                app.loading_info.elapsed_secs
            ));
        f.render_widget(gauge, layout[1]);

        let elapsed = app.loading_info.elapsed_secs;
        let elapsed_formatted = format!("{:02}:{:02}s", elapsed / 60, elapsed % 60);

        let details = vec![
            Line::from(vec![
                Span::styled(
                    "⏱️  Tempo decorrido: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(elapsed_formatted, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled(
                    "📦  Arquivos verificados: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    app.loading_info.files_count.clone(),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "📊  Tamanho Original: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    app.loading_info.original_size.clone(),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    "  |  Comprimido: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    app.loading_info.compressed_size.clone(),
                    Style::default().fg(Color::LightCyan),
                ),
                Span::styled(
                    "  |  Deduplicado: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    app.loading_info.deduplicated_size.clone(),
                    Style::default().fg(Color::LightGreen),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "📄  Arquivo atual: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if app.loading_info.current_file.is_empty() {
                        "Escaneando diretórios...".to_string()
                    } else {
                        app.loading_info.current_file.clone()
                    },
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ];

        let p = Paragraph::new(details).wrap(Wrap { trim: true });
        f.render_widget(p, layout[3]);
    }
}
