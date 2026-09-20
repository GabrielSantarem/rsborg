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

    // Se não houver backups, renderiza o Empty State amigável
    if app.archives.is_empty() {
        let empty_box = Block::default()
            .borders(Borders::ALL)
            .title(app.t.browse_empty_title())
            .style(app.theme.block_normal());

        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                app.t.browse_empty_header(),
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                app.t.browse_empty_desc(),
                Style::default().fg(app.theme.text),
            )),
            Line::from(""),
            Line::from(Span::styled(
                app.t.browse_empty_actions(),
                Style::default()
                    .fg(app.theme.secondary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  * ", Style::default().fg(app.theme.primary)),
                Span::styled(
                    app.t.browse_press(),
                    Style::default().fg(app.theme.text_muted),
                ),
                Span::styled("[c]", app.theme.key_badge_style()),
                Span::styled(
                    app.t.browse_empty_act1(),
                    Style::default().fg(app.theme.text),
                ),
            ]),
            Line::from(vec![
                Span::styled("  * ", Style::default().fg(app.theme.primary)),
                Span::styled(
                    app.t.browse_press(),
                    Style::default().fg(app.theme.text_muted),
                ),
                Span::styled("[b]", app.theme.key_badge_style()),
                Span::styled(
                    app.t.browse_empty_act2(),
                    Style::default().fg(app.theme.text),
                ),
            ]),
            Line::from(vec![
                Span::styled("  * ", Style::default().fg(app.theme.primary)),
                Span::styled(
                    app.t.browse_press(),
                    Style::default().fg(app.theme.text_muted),
                ),
                Span::styled("[r]", app.theme.key_badge_style()),
                Span::styled(
                    app.t.browse_empty_act3(),
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
                app.t.browse_fuse_mounted_fmt(&path.to_string_lossy()),
                Style::default()
                    .fg(app.theme.info)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                app.t.browse_fuse_unmounted(),
                Style::default().fg(app.theme.text_muted),
            )
        };

        let key_style = Style::default()
            .fg(app.theme.secondary)
            .add_modifier(Modifier::BOLD);
        let val_style = Style::default().fg(app.theme.text);

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(app.t.browse_meta_snapshot(), key_style),
                Span::styled(
                    &archive.name,
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(app.t.browse_meta_id(), key_style),
                Span::styled(&archive.id, val_style),
            ]),
            Line::from(vec![
                Span::styled(app.t.browse_meta_start(), key_style),
                Span::styled(&archive.start, val_style),
            ]),
            Line::from(vec![
                Span::styled(app.t.browse_meta_duration(), key_style),
                Span::styled(&archive.time, val_style),
            ]),
            Line::from(vec![
                Span::styled(app.t.browse_meta_fuse(), key_style),
                mount_line,
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  --------------------------------------",
                Style::default().fg(app.theme.border_normal),
            )),
            Line::from(""),
            Line::from(vec![Span::styled(
                app.t.browse_quick_actions(),
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![
                Span::styled("    [Enter] ", app.theme.key_badge_style()),
                Span::styled(
                    app.t.browse_act_explore(),
                    Style::default().fg(app.theme.text_muted),
                ),
            ]),
            Line::from(vec![
                Span::styled("    [m / u] ", app.theme.key_badge_style()),
                Span::styled(
                    app.t.browse_act_mount(),
                    Style::default().fg(app.theme.text_muted),
                ),
            ]),
            Line::from(vec![
                Span::styled("    [x]     ", app.theme.key_badge_style()),
                Span::styled(
                    app.t.browse_act_restore(),
                    Style::default().fg(app.theme.text_muted),
                ),
            ]),
            Line::from(vec![
                Span::styled("    [d]     ", app.theme.key_badge_style()),
                Span::styled(
                    app.t.browse_act_delete(),
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
