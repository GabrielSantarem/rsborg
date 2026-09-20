use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
};

use super::centered_rect;
use crate::app::App;
use crate::i18n::Translator;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render_confirm_delete(f: &mut Frame, t: &Translator, name: &str, screen_area: Rect) {
    let area = centered_rect(55, 30, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(t.confirm_delete_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(t.confirm_delete_question()),
            Span::styled(
                name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(t.confirm_delete_warning()),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                t.confirm_delete_btn_yes(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                t.confirm_delete_btn_cancel(),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let p = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(block);
    f.render_widget(p, area);
}

pub fn render_success(f: &mut Frame, t: &Translator, msg: &str, screen_area: Rect) {
    let area = centered_rect(60, 35, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(t.success_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightGreen));
    let p = Paragraph::new(msg)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .block(block)
        .style(Style::default().fg(Color::LightGreen));
    f.render_widget(p, area);
}

pub fn render_error(f: &mut Frame, t: &Translator, msg: &str, screen_area: Rect) {
    let area = centered_rect(65, 40, screen_area);
    f.render_widget(Clear, area);

    let is_lock = msg.contains("break-lock") || msg.contains("bloqueado") || msg.contains("locked");

    let block = Block::default()
        .title(t.warning_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(msg, Style::default().fg(Color::Red))),
        Line::from(""),
    ];

    if is_lock {
        lines.push(Line::from(vec![
            Span::styled(" [b] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(t.btn_break_lock(), Style::default().fg(Color::White)),
            Span::raw("    "),
            Span::styled(" [Esc] ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            Span::styled(t.confirm_delete_btn_cancel(), Style::default().fg(Color::Gray)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(" [Esc / Enter] ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            Span::styled(t.confirm_delete_btn_cancel(), Style::default().fg(Color::Gray)),
        ]));
    }

    let p = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .block(block);
    f.render_widget(p, area);
}

pub fn render_loading(f: &mut Frame, app: &App, screen_area: Rect) {
    let area = centered_rect(65, 45, screen_area);
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
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(6),
        ])
        .split(inner_area);

    let pulse_percent = ((app.loading_info.elapsed_secs * 15
        + app.loading_info.spinner_frame as u64 * 5)
        % 100) as u16;
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .percent(pulse_percent)
        .label(app.t.gauge_activity_fmt(app.loading_info.elapsed_secs));
    f.render_widget(gauge, layout[1]);

    let elapsed = app.loading_info.elapsed_secs;
    let elapsed_formatted = format!("{:02}:{:02}s", elapsed / 60, elapsed % 60);

    let details = vec![
        Line::from(vec![
            Span::styled(
                app.t.telemetry_elapsed(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(elapsed_formatted, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled(
                app.t.telemetry_files_count(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                app.loading_info.files_count.clone(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                app.t.telemetry_original_size(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                app.loading_info.original_size.clone(),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                app.t.telemetry_compressed_size(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                app.loading_info.compressed_size.clone(),
                Style::default().fg(Color::LightCyan),
            ),
            Span::styled(
                app.t.telemetry_deduplicated_size(),
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
                app.t.telemetry_current_file(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if app.loading_info.current_file.is_empty() {
                    app.t.telemetry_processing().to_string()
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

pub fn render_help_modal(f: &mut Frame, app: &App, screen_area: Rect) {
    let area = centered_rect(78, 80, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(app.t.help_title())
        .borders(Borders::ALL)
        .style(app.theme.block_active());

    let sec_style = Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD);
    let key_style = Style::default().fg(app.theme.secondary).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(app.theme.text);
    let hint_style = Style::default().fg(app.theme.text_muted);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(app.t.help_sec_nav(), sec_style)),
        Line::from(vec![
            Span::styled("    [j/k] ou [Setas]  ", key_style),
            Span::styled(app.t.help_nav_jk(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [Enter]           ", key_style),
            Span::styled(app.t.help_nav_enter(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [Esc] ou [q]      ", key_style),
            Span::styled(app.t.help_nav_esc(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [t] ou [T]        ", key_style),
            Span::styled(app.t.help_nav_theme(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [l]               ", key_style),
            Span::styled(app.t.help_nav_lang(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [L] ou [o]        ", key_style),
            Span::styled(app.t.help_act_logs(), desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(app.t.help_sec_actions(), sec_style)),
        Line::from(vec![
            Span::styled("    [c]               ", key_style),
            Span::styled(app.t.help_act_create(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [Enter]           ", key_style),
            Span::styled(app.t.help_act_inspect(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [x]               ", key_style),
            Span::styled(app.t.help_act_restore(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [m] / [u]         ", key_style),
            Span::styled(app.t.help_act_mount(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [f]               ", key_style),
            Span::styled(app.t.help_act_diff(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [v]               ", key_style),
            Span::styled(app.t.help_act_check(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [p]               ", key_style),
            Span::styled(app.t.help_act_prune(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [d]               ", key_style),
            Span::styled(app.t.help_act_delete(), desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(app.t.help_sec_profiles(), sec_style)),
        Line::from(vec![
            Span::styled("    [b]               ", key_style),
            Span::styled(app.t.help_prof_manage(), desc_style),
        ]),
        Line::from(vec![
            Span::styled("    [r]               ", key_style),
            Span::styled(app.t.help_prof_repos(), desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(app.t.help_close_hint(), hint_style)),
    ];

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}
