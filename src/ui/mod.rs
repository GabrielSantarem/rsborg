use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub mod backup;
pub mod browse;
pub mod inspect;
pub mod popups;
pub mod prune;
pub mod repos;
pub mod restore;

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

    // 1. Header
    let version_str = app
        .borg_version
        .as_deref()
        .unwrap_or_else(|| app.t.checking_borg());

    let (active_repo_name, active_repo_loc) = match app.get_active_repo() {
        Some(r) => (r.name.as_str(), r.location.as_str()),
        None => (app.t.default_local(), app.t.unknown()),
    };

    let header_text = vec![Line::from(vec![
        Span::styled(
            format!(" {} ", app.t.title()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "| {} | {}: {} [{}] ",
            version_str,
            app.t.header_repo(),
            active_repo_name,
            active_repo_loc
        )),
    ])];

    let header = Paragraph::new(header_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // 2. Main Body
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
        _ => {
            browse::render(f, app, chunks[1]);
        }
    }

    // 3. Footer
    let footer_text = match app.state {
        AppState::Browsing => app.t.footer_main(),
        AppState::CreatingBackup => app.t.footer_creating(),
        AppState::ConfirmDelete(_) => app.t.footer_confirm_delete(),
        AppState::ConfirmRestore(_) => app.t.footer_confirm_restore(),
        AppState::InspectArchive(_) => app.t.footer_inspect(),
        AppState::PruningPolicy(_) => app.t.footer_pruning_policy(),
        AppState::PrunePlanView(_) => app.t.footer_prune_plan(),
        AppState::ManagingRepos => app.t.footer_managing_repos(),
        AppState::AddingRepo => app.t.footer_adding_repo(),
        AppState::ErrorPopup(_) | AppState::SuccessPopup(_) => app.t.footer_popup(),
        AppState::Loading => app.t.footer_loading(),
        _ => " [Esc] ",
    };
    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);

    // 4. Overlays & Modals
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

    if app.state == AppState::ManagingRepos {
        repos::render_manage_repos(f, app, size);
    }

    if app.state == AppState::AddingRepo {
        repos::render_add_repo(f, app, size);
    }

    if let AppState::SuccessPopup(ref msg) = app.state {
        popups::render_success(f, &app.t, msg, size);
    }

    if let AppState::ErrorPopup(ref msg) = app.state {
        popups::render_error(f, &app.t, msg, size);
    }

    if app.state == AppState::Loading {
        popups::render_loading(f, app, size);
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
