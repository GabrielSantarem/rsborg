use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::centered_rect;
use crate::app::App;
use crate::app::state::ArchiveInfoState;
use crate::borg::{format_bytes, format_duration_secs};

pub fn render_archive_info(f: &mut Frame, app: &App, state: &ArchiveInfoState, screen_area: Rect) {
    let area = centered_rect(80, 78, screen_area);
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab navigation bar
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // --- Tab Navigation Bar ---
    let tab0_active = state.active_tab == 0;
    let tab1_active = state.active_tab == 1;

    let tab0_style = if tab0_active {
        Style::default()
            .fg(app.theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.text_muted)
    };

    let tab1_style = if tab1_active {
        Style::default()
            .fg(app.theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(app.theme.text_muted)
    };

    let tab_line = Line::from(vec![
        Span::styled(if tab0_active { " [>] " } else { " [ ] " }, tab0_style),
        Span::styled(app.t.info_tab_snapshot(), tab0_style),
        Span::styled("   |   ", Style::default().fg(app.theme.border_normal)),
        Span::styled(if tab1_active { " [>] " } else { " [ ] " }, tab1_style),
        Span::styled(app.t.info_tab_repo(), tab1_style),
    ]);

    let tabs_block = Block::default()
        .borders(Borders::ALL)
        .title(app.t.info_title())
        .title_alignment(Alignment::Center)
        .style(app.theme.block_active());
    f.render_widget(
        Paragraph::new(tab_line)
            .alignment(Alignment::Center)
            .block(tabs_block),
        chunks[0],
    );

    // --- Content by Tab ---
    let mut lines = Vec::new();
    lines.push(Line::from(""));

    let lbl_style = Style::default()
        .fg(app.theme.secondary)
        .add_modifier(Modifier::BOLD);
    let val_style = Style::default().fg(app.theme.text);
    let sec_style = Style::default()
        .fg(app.theme.primary)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    let highlight_style = Style::default()
        .fg(app.theme.success)
        .add_modifier(Modifier::BOLD);

    if state.active_tab == 0 {
        // Tab 0: Selected Snapshot Info
        let archive_details = state.info.archives.first();

        if let Some(archive) = archive_details {
            // Metadata Section
            lines.push(Line::from(vec![
                Span::styled("  :: ", lbl_style),
                Span::styled(app.t.info_sec_metadata(), sec_style),
            ]));
            lines.push(Line::from(""));

            lines.push(Line::from(vec![
                Span::styled(format!("    {:<26} ", app.t.info_lbl_name()), lbl_style),
                Span::styled(&archive.name, val_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled(format!("    {:<26} ", app.t.info_lbl_id()), lbl_style),
                Span::styled(&archive.id, Style::default().fg(app.theme.text_muted)),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_timestamp()),
                    lbl_style,
                ),
                Span::styled(&archive.start, val_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled(format!("    {:<26} ", app.t.info_lbl_duration()), lbl_style),
                Span::styled(format_duration_secs(archive.duration), val_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_host_user()),
                    lbl_style,
                ),
                Span::styled(
                    format!("{} / {}", archive.hostname, archive.username),
                    val_style,
                ),
            ]));

            if !archive.command_line.is_empty() {
                let cmd_str = archive.command_line.join(" ");
                lines.push(Line::from(vec![
                    Span::styled(format!("    {:<26} ", app.t.info_lbl_command()), lbl_style),
                    Span::styled(cmd_str, Style::default().fg(app.theme.text_muted)),
                ]));
            }

            lines.push(Line::from(""));

            // Stats Section
            lines.push(Line::from(vec![
                Span::styled("  :: ", lbl_style),
                Span::styled(app.t.info_sec_stats(), sec_style),
            ]));
            lines.push(Line::from(""));

            if let Some(ref stats) = archive.stats {
                lines.push(Line::from(vec![
                    Span::styled(format!("    {:<26} ", app.t.info_lbl_files()), lbl_style),
                    Span::styled(format!("{} arquivos", stats.nfiles), val_style),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("    {:<26} ", app.t.info_lbl_original_size()),
                        lbl_style,
                    ),
                    Span::styled(format_bytes(stats.original_size), val_style),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("    {:<26} ", app.t.info_lbl_compressed_size()),
                        lbl_style,
                    ),
                    Span::styled(format_bytes(stats.compressed_size), val_style),
                ]));

                let comp_ratio =
                    if stats.original_size > 0 && stats.original_size >= stats.compressed_size {
                        let saved = stats.original_size - stats.compressed_size;
                        let pct = (saved as f64 / stats.original_size as f64) * 100.0;
                        format!("{:.1}% (economizou {})", pct, format_bytes(saved))
                    } else {
                        "0.0%".to_string()
                    };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("    {:<26} ", app.t.info_lbl_compression_ratio()),
                        lbl_style,
                    ),
                    Span::styled(comp_ratio, highlight_style),
                ]));

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("    {:<26} ", app.t.info_lbl_dedup_size()),
                        lbl_style,
                    ),
                    Span::styled(format_bytes(stats.deduplicated_size), highlight_style),
                ]));
            } else {
                lines.push(Line::from(vec![Span::styled(
                    "    Estatísticas não registradas para este snapshot.",
                    Style::default().fg(app.theme.text_muted),
                )]));
            }
        } else {
            lines.push(Line::from(vec![Span::styled(
                "    Nenhum detalhe encontrado para o snapshot selecionado.",
                Style::default().fg(app.theme.text_muted),
            )]));
        }
    } else {
        // Tab 1: Global Repository & Deduplication Info
        lines.push(Line::from(vec![
            Span::styled("  :: ", lbl_style),
            Span::styled(app.t.info_sec_repo(), sec_style),
        ]));
        lines.push(Line::from(""));

        if let Some(ref repo) = state.info.repository {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_repo_location()),
                    lbl_style,
                ),
                Span::styled(&repo.location, val_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled(format!("    {:<26} ", app.t.info_lbl_repo_id()), lbl_style),
                Span::styled(&repo.id, Style::default().fg(app.theme.text_muted)),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_timestamp()),
                    lbl_style,
                ),
                Span::styled(&repo.last_modified, val_style),
            ]));
        }

        if let Some(ref enc) = state.info.encryption {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_encryption()),
                    lbl_style,
                ),
                Span::styled(&enc.mode, val_style),
            ]));
        }

        lines.push(Line::from(""));

        // Global Deduplication Section
        lines.push(Line::from(vec![
            Span::styled("  :: ", lbl_style),
            Span::styled(app.t.info_sec_dedup(), sec_style),
        ]));
        lines.push(Line::from(""));

        if let Some(ref cache) = state.info.cache
            && let Some(ref stats) = cache.stats
        {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_repo_original()),
                    lbl_style,
                ),
                Span::styled(format_bytes(stats.total_size), val_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_repo_compressed()),
                    lbl_style,
                ),
                Span::styled(format_bytes(stats.total_csize), val_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_repo_dedup()),
                    lbl_style,
                ),
                Span::styled(format_bytes(stats.unique_csize), highlight_style),
            ]));

            let dedup_savings = stats.total_csize.saturating_sub(stats.unique_csize);
            let savings_pct = if stats.total_csize > 0 {
                (dedup_savings as f64 / stats.total_csize as f64) * 100.0
            } else {
                0.0
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_repo_savings()),
                    lbl_style,
                ),
                Span::styled(
                    format!(
                        "{} ({:.1}% de economia total)",
                        format_bytes(dedup_savings),
                        savings_pct
                    ),
                    highlight_style,
                ),
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(format!("    {:<26} ", app.t.info_lbl_chunks()), lbl_style),
                Span::styled(format!("{} chunks", stats.total_chunks), val_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:<26} ", app.t.info_lbl_unique_chunks()),
                    lbl_style,
                ),
                Span::styled(
                    format!(
                        "{} chunks únicos ({:.1}% deduplicação)",
                        stats.total_unique_chunks,
                        if stats.total_chunks > 0 {
                            (1.0 - (stats.total_unique_chunks as f64 / stats.total_chunks as f64))
                                * 100.0
                        } else {
                            0.0
                        }
                    ),
                    val_style,
                ),
            ]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "    Estatísticas de cache do repositório indisponíveis.",
                Style::default().fg(app.theme.text_muted),
            )]));
        }
    }

    let content_block = Block::default()
        .borders(Borders::ALL)
        .style(app.theme.block_normal());
    f.render_widget(Paragraph::new(lines).block(content_block), chunks[1]);

    // --- Footer ---
    let footer_p = Paragraph::new(app.t.info_footer())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(app.theme.block_normal()),
        );
    f.render_widget(footer_p, chunks[2]);
}
