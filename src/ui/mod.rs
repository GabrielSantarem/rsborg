use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub mod backup;
pub mod browse;
pub mod check;
pub mod diff;
pub mod inspect;
pub mod popups;
pub mod profiles;
pub mod prune;
pub mod repos;
pub mod restore;
pub mod theme;

use crate::app::{App, AppState};

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

    // 1. Dashboard Header (Bordas retas e badges em texto limpo)
    let version_str = app
        .borg_version
        .as_deref()
        .unwrap_or_else(|| app.t.checking_borg());

    let (active_repo_name, active_repo_loc) = match app.get_active_repo() {
        Some(r) => (r.name.as_str(), r.location.as_str()),
        None => (app.t.default_local(), app.t.unknown()),
    };

    let status_badge = if app.borg_version.is_some() {
        Span::styled(
            " [● ONLINE] ",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [● OFFLINE] ",
            Style::default()
                .fg(app.theme.danger)
                .add_modifier(Modifier::BOLD),
        )
    };

    let header_text = vec![Line::from(vec![
        Span::styled(
            format!(" {} ", app.t.title()),
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        status_badge,
        Span::styled(
            format!(" [BORG {}] ", version_str),
            Style::default().fg(app.theme.info),
        ),
        Span::styled(
            format!(" [{}: {}] ", app.t.header_repo(), active_repo_name),
            Style::default()
                .fg(app.theme.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" [{}] ", active_repo_loc),
            Style::default().fg(app.theme.text_muted),
        ),
        Span::styled(
            format!(" [TEMA: {}] ", app.theme.mode.name()),
            Style::default().fg(app.theme.primary),
        ),
    ])];

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .style(app.theme.block_normal()),
    );
    f.render_widget(header, chunks[0]);

    // 2. Main Body
    match app.state {
        AppState::Initializing => {
            let p = Paragraph::new("Carregando / Initializing...")
                .style(Style::default().fg(app.theme.text_muted))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(app.theme.block_normal()),
                );
            f.render_widget(p, chunks[1]);
        }
        AppState::InitError(ref err) => {
            let p = Paragraph::new(format!("Erro ao inicializar BorgBackup:\n\n{}", err))
                .style(Style::default().fg(app.theme.danger))
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(app.theme.danger)),
                );
            f.render_widget(p, chunks[1]);
        }
        _ => {
            browse::render(f, app, chunks[1]);
        }
    }

    // 3. Footer com Badges Estilizados
    let footer_line = render_styled_footer(app);
    let footer = Paragraph::new(footer_line).block(
        Block::default()
            .borders(Borders::ALL)
            .style(app.theme.block_normal()),
    );
    f.render_widget(footer, chunks[2]);

    // 4. Overlays & Modais
    if app.state == AppState::CreatingBackup {
        backup::render(f, app, size);
    }

    if let AppState::ConfirmDelete(ref name) = app.state {
        popups::render_confirm_delete(f, &app.t, name, size);
    }

    if let AppState::ConfirmRestore(ref req) = app.state {
        restore::render(f, &app.t, req, size);
    }

    if let AppState::InspectArchive(ref inspect) = app.state {
        inspect::render(f, &app.t, inspect, size);
    }

    if let AppState::PruningPolicy(ref policy_state) = app.state {
        prune::render_policy_modal(f, &app.t, policy_state, size);
    }

    if let AppState::PrunePlanView(ref plan_state) = app.state {
        prune::render_plan_view(f, &app.t, plan_state, size);
    }

    if let AppState::CheckWizard(ref state) = app.state {
        check::render_check_wizard(f, &app.t, state, size);
    }

    if let AppState::CheckResultView(ref state) = app.state {
        check::render_check_result(f, &app.t, state, size);
    }

    if let AppState::DiffWizard(ref state) = app.state {
        diff::render_diff_wizard(f, &app.t, state, size);
    }

    if let AppState::DiffView(ref state) = app.state {
        diff::render_diff_view(f, &app.t, state, size);
    }

    if app.state == AppState::ManagingRepos {
        repos::render_manage_repos(f, app, size);
    }

    if app.state == AppState::AddingRepo {
        repos::render_add_repo(f, app, size);
    }

    if let AppState::ManagingProfiles { .. } = app.state {
        profiles::render_profiles_view(f, app, size);
    }

    if let AppState::CreatingProfile(_) = app.state {
        profiles::render_profile_wizard(f, app, size);
    }

    if let AppState::AutomationView(ref view_state) = app.state {
        profiles::render_automation_modal(f, app, view_state, size);
    }

    if let AppState::ErrorPopup(ref msg) = app.state {
        popups::render_error(f, &app.t, msg, size);
    }

    if let AppState::SuccessPopup(ref msg) = app.state {
        popups::render_success(f, &app.t, msg, size);
    }

    if app.state == AppState::Loading {
        popups::render_loading(f, app, size);
    }

    if app.state == AppState::HelpModal {
        popups::render_help_modal(f, app, size);
    }
}

fn render_styled_footer<'a>(app: &'a App) -> Line<'a> {
    let key_style = app.theme.key_badge_style();
    let desc_style = app.theme.desc_style();
    let sep = Span::styled(" | ", Style::default().fg(app.theme.border_normal));

    match app.state {
        AppState::Browsing => {
            let is_pt = app.t.lang == crate::i18n::Language::Pt;
            Line::from(vec![
                Span::styled(" [c] ", key_style),
                Span::styled(if is_pt { "Criar" } else { "Create" }, desc_style),
                sep.clone(),
                Span::styled(" [b] ", key_style),
                Span::styled(if is_pt { "Perfis" } else { "Profiles" }, desc_style),
                sep.clone(),
                Span::styled(" [Enter] ", key_style),
                Span::styled(if is_pt { "Inspecionar" } else { "Inspect" }, desc_style),
                sep.clone(),
                Span::styled(" [f] ", key_style),
                Span::styled("Diff", desc_style),
                sep.clone(),
                Span::styled(" [v] ", key_style),
                Span::styled(if is_pt { "Verificar" } else { "Verify" }, desc_style),
                sep.clone(),
                Span::styled(" [p] ", key_style),
                Span::styled(if is_pt { "Retenção" } else { "Prune" }, desc_style),
                sep.clone(),
                Span::styled(" [m/u] ", key_style),
                Span::styled(
                    if is_pt { "Montar/Desm." } else { "Mount/Unm." },
                    desc_style,
                ),
                sep.clone(),
                Span::styled(" [t] ", key_style),
                Span::styled(if is_pt { "Tema" } else { "Theme" }, desc_style),
                sep.clone(),
                Span::styled(" [l] ", key_style),
                Span::styled(if is_pt { "Idioma" } else { "Language" }, desc_style),
                sep.clone(),
                Span::styled(" [?] ", key_style),
                Span::styled(if is_pt { "Ajuda" } else { "Help" }, desc_style),
                sep,
                Span::styled(" [q] ", key_style),
                Span::styled(if is_pt { "Sair" } else { "Quit" }, desc_style),
            ])
        }
        AppState::HelpModal => {
            let is_pt = app.t.lang == crate::i18n::Language::Pt;
            Line::from(vec![
                Span::styled(" [Esc / q / ?] ", key_style),
                Span::styled(
                    if is_pt { "Fechar Ajuda" } else { "Close Help" },
                    desc_style,
                ),
            ])
        }
        _ => {
            let text = match app.state {
                AppState::CreatingBackup => app.t.footer_creating(),
                AppState::ConfirmDelete(_) => app.t.footer_confirm_delete(),
                AppState::ConfirmRestore(_) => app.t.footer_confirm_restore(),
                AppState::InspectArchive(_) => app.t.footer_inspect(),
                AppState::PruningPolicy(_) => app.t.footer_pruning_policy(),
                AppState::PrunePlanView(_) => app.t.footer_prune_plan(),
                AppState::CheckWizard(_) => app.t.footer_check_wizard(),
                AppState::CheckResultView(_) => app.t.footer_check_result(),
                AppState::DiffWizard(_) => app.t.footer_diff_wizard(),
                AppState::DiffView(_) => app.t.footer_diff_view(),
                AppState::ManagingRepos => app.t.footer_managing_repos(),
                AppState::AddingRepo => app.t.footer_adding_repo(),
                AppState::ManagingProfiles { .. } => app.t.footer_profiles(),
                AppState::CreatingProfile(_) => app.t.footer_profile_wizard(),
                AppState::AutomationView(_) => app.t.footer_automation_view(),
                AppState::ErrorPopup(_) | AppState::SuccessPopup(_) => app.t.footer_popup(),
                AppState::Loading => app.t.footer_loading(),
                _ => " [Esc] Voltar ",
            };
            Line::from(vec![Span::styled(text, desc_style)])
        }
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
