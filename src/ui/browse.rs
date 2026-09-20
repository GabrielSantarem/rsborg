use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let is_pt = app.t.lang == crate::i18n::Language::Pt;

    // Se não houver backups, renderiza o Empty State amigável
    if app.archives.is_empty() {
        let empty_title = if is_pt {
            " [ LISTA DE BACKUPS ] "
        } else {
            " [ BACKUP LIST ] "
        };
        let empty_box = Block::default()
            .borders(Borders::ALL)
            .title(empty_title)
            .style(app.theme.block_normal());

        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                if is_pt {
                    " [ NENHUM BACKUP ENCONTRADO ] "
                } else {
                    " [ NO ARCHIVES FOUND ] "
                },
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                if is_pt {
                    "O repositório ativo está pronto, mas ainda não possui nenhum snapshot arquivado."
                } else {
                    "The active repository is ready, but contains no archived snapshots yet."
                },
                Style::default().fg(app.theme.text),
            )),
            Line::from(""),
            Line::from(Span::styled(
                if is_pt {
                    "Ações recomendadas:"
                } else {
                    "Recommended actions:"
                },
                Style::default()
                    .fg(app.theme.secondary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  * ", Style::default().fg(app.theme.primary)),
                Span::styled(
                    if is_pt { "Pressione " } else { "Press " },
                    Style::default().fg(app.theme.text_muted),
                ),
                Span::styled("[c]", app.theme.key_badge_style()),
                Span::styled(
                    if is_pt {
                        " para criar seu primeiro backup interativo"
                    } else {
                        " to create your first interactive backup"
                    },
                    Style::default().fg(app.theme.text),
                ),
            ]),
            Line::from(vec![
                Span::styled("  * ", Style::default().fg(app.theme.primary)),
                Span::styled(
                    if is_pt { "Pressione " } else { "Press " },
                    Style::default().fg(app.theme.text_muted),
                ),
                Span::styled("[b]", app.theme.key_badge_style()),
                Span::styled(
                    if is_pt {
                        " para configurar um perfil automatizado com systemd/cron"
                    } else {
                        " to configure an automated profile with systemd/cron"
                    },
                    Style::default().fg(app.theme.text),
                ),
            ]),
            Line::from(vec![
                Span::styled("  * ", Style::default().fg(app.theme.primary)),
                Span::styled(
                    if is_pt { "Pressione " } else { "Press " },
                    Style::default().fg(app.theme.text_muted),
                ),
                Span::styled("[r]", app.theme.key_badge_style()),
                Span::styled(
                    if is_pt {
                        " para alternar para outro repositório com backups existentes"
                    } else {
                        " to switch to another repository with existing backups"
                    },
                    Style::default().fg(app.theme.text),
                ),
            ]),
        ];

        let p = Paragraph::new(empty_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(empty_box);
        f.render_widget(p, body_chunks[0]);
    } else {
        // Tabela de Backups com tema limpo e bordas retas
        let n_col = app.t.col_name();
        let s_col = app.t.col_start();
        let header_style = Style::default()
            .fg(app.theme.secondary)
            .add_modifier(Modifier::BOLD);

        let cells = vec![
            Cell::from(format!(" [ {} ]", n_col)).style(header_style),
            Cell::from(format!(" [ {} ]", s_col)).style(header_style),
        ];
        let header_row = Row::new(cells).height(1).bottom_margin(1);

        let rows = app.archives.iter().map(|a| {
            let is_mounted = app.mounted_archives.contains_key(&a.name);
            let name_display = if is_mounted {
                format!("{} [FUSE]", a.name)
            } else {
                a.name.clone()
            };

            let name_cell = if is_mounted {
                Cell::from(name_display).style(
                    Style::default()
                        .fg(app.theme.info)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Cell::from(name_display).style(Style::default().fg(app.theme.text))
            };

            let date_cell =
                Cell::from(a.start.clone()).style(Style::default().fg(app.theme.text_muted));
            Row::new(vec![name_cell, date_cell]).height(1)
        });

        let table = Table::new(
            rows,
            [Constraint::Percentage(55), Constraint::Percentage(45)],
        )
        .header(header_row)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", app.t.table_title()))
                .style(app.theme.block_normal()),
        )
        .row_highlight_style(app.theme.row_selected_style())
        .highlight_symbol(">> ");

        f.render_stateful_widget(table, body_chunks[0], &mut app.table_state);
    }

    // Painel Lateral de Metadados / Detalhes
    let meta_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", app.t.metadata_title()))
        .style(app.theme.block_normal());

    if let Some(i) = app.table_state.selected()
        && let Some(archive) = app.archives.get(i)
    {
            let is_mounted = app.mounted_archives.contains_key(&archive.name);
            let mount_line = if is_mounted {
                let path = app.mounted_archives.get(&archive.name).unwrap();
                Span::styled(
                    format!(" [MONTADO em {}]", path.display()),
                    Style::default()
                        .fg(app.theme.info)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(" [NÃO MONTADO]", Style::default().fg(app.theme.text_muted))
            };

            let key_style = Style::default()
                .fg(app.theme.secondary)
                .add_modifier(Modifier::BOLD);
            let val_style = Style::default().fg(app.theme.text);

            let lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Snapshot:  ", key_style),
                    Span::styled(
                        &archive.name,
                        Style::default()
                            .fg(app.theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  ID:        ", key_style),
                    Span::styled(&archive.id, val_style),
                ]),
                Line::from(vec![
                    Span::styled("  Início:    ", key_style),
                    Span::styled(&archive.start, val_style),
                ]),
                Line::from(vec![
                    Span::styled("  Duração:   ", key_style),
                    Span::styled(&archive.time, val_style),
                ]),
                Line::from(vec![Span::styled("  FUSE:      ", key_style), mount_line]),
                Line::from(""),
                Line::from(Span::styled(
                    "  --------------------------------------",
                    Style::default().fg(app.theme.border_normal),
                )),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "  Ações Rápidas:",
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                    Span::styled("    [Enter] ", app.theme.key_badge_style()),
                    Span::styled(
                        if is_pt {
                            "Explorar conteúdo interno"
                        } else {
                            "Explore internal files"
                        },
                        Style::default().fg(app.theme.text_muted),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("    [m / u] ", app.theme.key_badge_style()),
                    Span::styled(
                        if is_pt {
                            "Montar / Desmontar FUSE"
                        } else {
                            "Mount / Unmount FUSE"
                        },
                        Style::default().fg(app.theme.text_muted),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("    [x]     ", app.theme.key_badge_style()),
                    Span::styled(
                        if is_pt {
                            "Restaurar este backup"
                        } else {
                            "Restore this backup"
                        },
                        Style::default().fg(app.theme.text_muted),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("    [d]     ", app.theme.key_badge_style()),
                    Span::styled(
                        if is_pt {
                            "Excluir permanentemente"
                        } else {
                            "Delete permanently"
                        },
                        Style::default().fg(app.theme.danger),
                    ),
                ]),
            ];

            let p = Paragraph::new(lines).block(meta_block);
            f.render_widget(p, body_chunks[1]);
            return;
    }

    let p = Paragraph::new(format!("\n  {}", app.t.no_backup_selected()))
        .style(Style::default().fg(app.theme.text_muted))
        .block(meta_block);
    f.render_widget(p, body_chunks[1]);
}
