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
    let area = centered_rect(55, 30, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(t.warning_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));
    let p = Paragraph::new(msg)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .block(block)
        .style(Style::default().fg(Color::Red));
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

    let is_pt = app.t.lang == crate::i18n::Language::Pt;
    let title = if is_pt { " [ AJUDA / ATALHOS DE TECLADO ] " } else { " [ HELP / KEYBOARD SHORTCUTS ] " };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(app.theme.block_active());

    let sec_style = Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD);
    let key_style = Style::default().fg(app.theme.secondary).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(app.theme.text);
    let hint_style = Style::default().fg(app.theme.text_muted);

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    let sec1_title = if is_pt { "  NAVEGAÇÃO GERAL" } else { "  GENERAL NAVIGATION" };
    lines.push(Line::from(Span::styled(sec1_title, sec_style)));
    lines.push(Line::from(vec![
        Span::styled("    [j/k] ou [Setas]  ", key_style),
        Span::styled(if is_pt { "Navegar pelas listas de backups ou arquivos" } else { "Navigate backup or file lists" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [Enter]           ", key_style),
        Span::styled(if is_pt { "Acessar diretório / Confirmar seleção" } else { "Enter directory / Confirm selection" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [Esc] ou [q]      ", key_style),
        Span::styled(if is_pt { "Voltar à tela anterior / Fechar modal / Sair" } else { "Go back / Close modal / Exit" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [t] ou [T]        ", key_style),
        Span::styled(if is_pt { "Alternar Tema Visual (Rust Oxide <-> Catppuccin Mocha)" } else { "Toggle Theme (Rust Oxide <-> Catppuccin Mocha)" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [l] ou [L]        ", key_style),
        Span::styled(if is_pt { "Alternar Idioma (Português <-> English)" } else { "Toggle Language (Português <-> English)" }, desc_style),
    ]));
    lines.push(Line::from(""));

    let sec2_title = if is_pt { "  AÇÕES EM SNAPSHOTS" } else { "  SNAPSHOT ACTIONS" };
    lines.push(Line::from(Span::styled(sec2_title, sec_style)));
    lines.push(Line::from(vec![
        Span::styled("    [c]               ", key_style),
        Span::styled(if is_pt { "Criar Novo Backup (Assistente com seletor de arquivos)" } else { "Create New Backup (File picker wizard)" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [Enter]           ", key_style),
        Span::styled(if is_pt { "Inspecionar arquivos dentro do snapshot selecionado" } else { "Inspect files inside selected snapshot" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [x]               ", key_style),
        Span::styled(if is_pt { "Restaurar snapshot selecionado para disco" } else { "Restore selected snapshot to disk" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [m] / [u]         ", key_style),
        Span::styled(if is_pt { "Montar snapshot via FUSE em ~/.rsborg/mnt / Desmontar" } else { "Mount snapshot via FUSE in ~/.rsborg/mnt / Unmount" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [f]               ", key_style),
        Span::styled(if is_pt { "Comparar Versões (Diff visual entre dois snapshots)" } else { "Compare Versions (Visual diff between two snapshots)" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [v]               ", key_style),
        Span::styled(if is_pt { "Verificar integridade do repositório (borg check)" } else { "Check repository integrity (borg check)" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [p]               ", key_style),
        Span::styled(if is_pt { "Política de Retenção & Poda com Simulação (borg prune dry-run)" } else { "Retention Policy & Pruning Simulation (borg prune dry-run)" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [d]               ", key_style),
        Span::styled(if is_pt { "Excluir snapshot permanentemente do repositório" } else { "Permanently delete snapshot from repository" }, desc_style),
    ]));
    lines.push(Line::from(""));

    let sec3_title = if is_pt { "  PERFIS & AUTOMAÇÃO" } else { "  PROFILES & AUTOMATION" };
    lines.push(Line::from(Span::styled(sec3_title, sec_style)));
    lines.push(Line::from(vec![
        Span::styled("    [b]               ", key_style),
        Span::styled(if is_pt { "Gerenciar Perfis de Backup e Gerador Systemd/Cron" } else { "Manage Backup Profiles and Systemd/Cron Generator" }, desc_style),
    ]));
    lines.push(Line::from(vec![
        Span::styled("    [r]               ", key_style),
        Span::styled(if is_pt { "Alternar entre Repositórios configurados ou adicionar novo" } else { "Switch configured Repositories or add new one" }, desc_style),
    ]));
    lines.push(Line::from(""));

    let close_hint = if is_pt { "  [ Pressione Esc, q ou ? para fechar este menu de ajuda ]" } else { "  [ Press Esc, q or ? to close this help menu ]" };
    lines.push(Line::from(Span::styled(close_hint, hint_style)));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}
