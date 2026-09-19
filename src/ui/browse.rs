use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

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
        let is_mounted = app.mounted_archives.contains_key(&a.name);
        let name_display = if is_mounted {
            format!("{} [FUSE 📂]", a.name)
        } else {
            a.name.clone()
        };

        let name_cell = if is_mounted {
            Cell::from(name_display).style(Style::default().fg(Color::Magenta))
        } else {
            Cell::from(name_display)
        };

        let r_cells = vec![name_cell, Cell::from(a.start.clone())];
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
            let mount_status = match app.mounted_archives.get(&archive.name) {
                Some(path) => app.t.fuse_mounted_fmt(&path.display().to_string()),
                None => app.t.fuse_not_mounted().to_string(),
            };

            let md_text = app.t.metadata_content_fmt(
                &archive.id,
                &archive.name,
                &archive.archive,
                &archive.start,
                &archive.time,
                &mount_status,
            );
            let p = Paragraph::new(md_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(app.t.metadata_title()),
            );
            f.render_widget(p, body_chunks[1]);
        }
    } else {
        let p = Paragraph::new(app.t.no_backup_selected()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.t.metadata_title()),
        );
        f.render_widget(p, body_chunks[1]);
    }
}
