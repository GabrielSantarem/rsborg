use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use super::centered_rect;
use crate::app::{App, AppState, AutomationViewState, ProfileFocus};
use crate::browser::ItemStatus;

pub const COMPRESSION_OPTIONS: [&str; 5] = ["lz4", "zstd,3", "zstd,9", "zlib,6", "none"];
pub const SCHEDULE_OPTIONS: [(&str, &str); 4] = [
    ("daily", "Diário (Daily)"),
    ("weekly", "Semanal (Weekly)"),
    ("hourly", "A cada hora (Hourly)"),
    ("manual", "Manual (Sob demanda)"),
];

pub fn render_profiles_view(f: &mut Frame, app: &mut App, screen_area: Rect) {
    let area = centered_rect(85, 80, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(app.t.profiles_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let selected_idx = match app.state {
        AppState::ManagingProfiles { selected_index } => selected_index,
        _ => 0,
    };

    if app.config.profiles.is_empty() {
        let empty_p = Paragraph::new(app.t.profiles_empty())
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(empty_p, inner_area);
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner_area);

    // Left: Profile List
    let items: Vec<ListItem> = app
        .config
        .profiles
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let is_selected = idx == selected_idx;
            let bullet = if is_selected { "➔ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = format!("{}{} [#{}] ({})", bullet, p.name, p.counter + 1, p.schedule);
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.t.profiles_configured_title()),
        )
        .highlight_symbol("➔ ");

    let mut l_state = ListState::default();
    l_state.select(Some(selected_idx));
    f.render_stateful_widget(list, split[0], &mut l_state);

    // Right: Selected Profile Details
    if let Some(profile) = app.config.profiles.get(selected_idx) {
        let next_name = profile.next_archive_name(chrono::Local::now());

        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    app.t.profile_label_name(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &profile.name,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    app.t.profile_label_compression(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(&profile.compression, Style::default().fg(Color::Cyan)),
                Span::raw("   |   "),
                Span::styled(
                    app.t.profile_label_schedule(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(&profile.schedule, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled(
                    app.t.profile_label_runs(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{}", profile.counter)),
            ]),
            Line::from(vec![
                Span::styled(
                    app.t.profile_next_archive_fmt(&next_name),
                    Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                app.t.profile_included_paths_fmt(profile.paths.len()),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::LightCyan),
            )),
        ];

        for p in &profile.paths {
            lines.push(Line::from(format!("  [+] {}", p.display())));
        }

        if !profile.excludes.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                app.t.profile_excluded_paths_fmt(profile.excludes.len()),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::LightRed),
            )));
            for e in &profile.excludes {
                lines.push(Line::from(format!("  [-] {}", e.display())));
            }
        }

        let detail_p = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(app.t.profile_details_title()),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(detail_p, split[1]);
    }
}

pub fn render_profile_wizard(
    f: &mut Frame,
    app: &mut App,
    screen_area: Rect,
) {
    let wizard = if let AppState::CreatingProfile(ref w) = app.state {
        w.clone()
    } else {
        return;
    };
    let area = centered_rect(80, 85, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(app.t.profile_wizard_title())
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
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(inner_area);

    // 1. Name Input
    let name_border_color = if wizard.focus == ProfileFocus::Name {
        Color::Yellow
    } else {
        Color::Gray
    };
    let name_text = format!("{}_", wizard.name);
    let name_p = Paragraph::new(name_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(name_border_color))
            .title(app.t.profile_name_label()),
    );
    f.render_widget(name_p, layout[0]);

    // 2. Compression Options
    let comp_border_color = if wizard.focus == ProfileFocus::Compression {
        Color::Yellow
    } else {
        Color::Gray
    };
    let comp_spans: Vec<Span> = COMPRESSION_OPTIONS
        .iter()
        .enumerate()
        .map(|(idx, opt)| {
            let is_sel = idx == wizard.compression_idx;
            if is_sel {
                Span::styled(
                    format!(" [• {}] ", opt),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("  ( ) {}  ", opt),
                    Style::default().fg(Color::DarkGray),
                )
            }
        })
        .collect();

    let comp_p = Paragraph::new(Line::from(comp_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(comp_border_color))
            .title(app.t.profile_compression_label()),
    );
    f.render_widget(comp_p, layout[1]);

    // 3. Schedule Frequency
    let sched_border_color = if wizard.focus == ProfileFocus::Schedule {
        Color::Yellow
    } else {
        Color::Gray
    };
    let sched_spans: Vec<Span> = SCHEDULE_OPTIONS
        .iter()
        .enumerate()
        .map(|(idx, (_, label))| {
            let is_sel = idx == wizard.schedule_idx;
            if is_sel {
                Span::styled(
                    format!(" [• {}] ", label),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("  ( ) {}  ", label),
                    Style::default().fg(Color::DarkGray),
                )
            }
        })
        .collect();

    let sched_p = Paragraph::new(Line::from(sched_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(sched_border_color))
            .title(app.t.profile_schedule_label()),
    );
    f.render_widget(sched_p, layout[2]);

    // 4. File Browser
    let browser_border_color = if wizard.focus == ProfileFocus::Browser {
        Color::Yellow
    } else {
        Color::Gray
    };
    let browser_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(browser_border_color))
        .title(app.t.profile_paths_label());

    let browser_inner = browser_block.inner(layout[3]);
    f.render_widget(browser_block, layout[3]);

    let list_items: Vec<ListItem> = app
        .file_browser
        .entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = idx == app.file_browser.selected_index;
            let status = app.file_browser.get_status(&entry.path);

            let (check, check_color) = match status {
                ItemStatus::ExplicitInclude => ("[X] ", Color::Green),
                ItemStatus::InheritedInclude => ("[+] ", Color::LightGreen),
                ItemStatus::ExplicitExclude => ("[-] ", Color::Red),
                ItemStatus::InheritedExclude => ("[!] ", Color::LightRed),
                ItemStatus::Neutral => ("[ ] ", Color::DarkGray),
            };

            let prefix = if entry.is_dir { "[DIR] " } else { "[FILE] " };
            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(40, 50, 70))
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(check, Style::default().fg(check_color)),
                Span::raw(prefix),
                Span::raw(&entry.name),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(list_items);
    f.render_widget(list, browser_inner);

    // 5. Action Prompt
    let prompt = Paragraph::new(app.t.profile_save_prompt())
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(prompt, layout[4]);
}

pub fn render_automation_modal(
    f: &mut Frame,
    app: &App,
    state: &AutomationViewState,
    screen_area: Rect,
) {
    let area = centered_rect(80, 75, screen_area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(app.t.automation_modal_title())
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(inner_area);

    // Tabs
    let tab_titles = [
        app.t.automation_tab_service(),
        app.t.automation_tab_timer(),
        app.t.automation_tab_cron(),
    ];

    let tab_spans: Vec<Span> = tab_titles
        .iter()
        .enumerate()
        .map(|(idx, title)| {
            let is_sel = idx == state.active_tab;
            if is_sel {
                Span::styled(
                    *title,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(*title, Style::default().fg(Color::DarkGray))
            }
        })
        .collect();

    let tabs_p =
        Paragraph::new(Line::from(tab_spans)).block(Block::default().borders(Borders::ALL));
    f.render_widget(tabs_p, layout[0]);

    // Content
    let (repo_path, passphrase) = match app.get_active_repo() {
        Some(r) => (r.location.clone(), r.passphrase.clone()),
        None => (crate::config::get_default_repo_path(), None),
    };

    let content = match state.active_tab {
        0 => state
            .profile
            .generate_systemd_service(&repo_path, passphrase.as_deref()),
        1 => state.profile.generate_systemd_timer(),
        _ => state
            .profile
            .generate_crontab_line(&repo_path, passphrase.as_deref()),
    };

    let content_p = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.t.automation_generated_content_title()),
        )
        .style(Style::default().fg(Color::LightCyan));
    f.render_widget(content_p, layout[1]);

    // Footer prompt
    let prompt = Paragraph::new(app.t.automation_footer_prompt())
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Yellow));
    f.render_widget(prompt, layout[2]);
}
