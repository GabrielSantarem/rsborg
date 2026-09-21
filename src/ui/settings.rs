use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::centered_rect;
use crate::app::App;
use crate::app::state::SettingsState;
use crate::i18n::Language;
use crate::ui::theme::ThemeMode;

pub fn render_settings(f: &mut Frame, app: &App, state: &SettingsState, screen_area: Rect) {
    let area = centered_rect(68, 62, screen_area);
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Options list
            Constraint::Length(4), // Helpful description for selected option
            Constraint::Length(3), // Footer
        ])
        .split(area);

    let is_pt = app.t.lang == Language::Pt;

    // Values for badges
    let lang_badge = match app.t.lang {
        Language::Pt => "Português (Brasil)",
        Language::En => "English (US)",
    };

    let theme_badge = match app.theme.mode {
        ThemeMode::Rust => "Rust Oxide (Laranja & Carvão)",
        ThemeMode::Catppuccin => "Catppuccin Mocha",
    };

    let repos_count = app.config.repositories.len();
    let repos_badge = if is_pt {
        format!("{} configurado(s) ->", repos_count)
    } else {
        format!("{} configured ->", repos_count)
    };

    let stats_badge = if is_pt {
        "Consultar deduplicação ->"
    } else {
        "Query deduplication ->"
    };

    let logs_badge = if is_pt {
        "Visualizar rsborg.log ->"
    } else {
        "View rsborg.log ->"
    };

    let restore_badge = "~/Restaurados".to_string();

    let borg_version = app.borg_version.as_deref().unwrap_or("Indisponível");
    let core_badge = format!("Borg v{} (OK)", borg_version);

    let items = [
        (app.t.settings_item_lang(), lang_badge.to_string(), true),
        (app.t.settings_item_theme(), theme_badge.to_string(), true),
        (app.t.settings_item_repos(), repos_badge, true),
        (app.t.settings_item_stats(), stats_badge.to_string(), true),
        (app.t.settings_item_logs(), logs_badge.to_string(), true),
        (app.t.settings_item_restore(), restore_badge, false),
        (app.t.settings_item_core(), core_badge, false),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    for (idx, (label, value, interactive)) in items.iter().enumerate() {
        let is_selected = state.selected_index == idx;

        let cursor_span = if is_selected {
            Span::styled(
                " [>] ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(" [ ] ", Style::default().fg(app.theme.border_normal))
        };

        let label_style = if is_selected {
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text)
        };

        let value_style = if is_selected {
            Style::default()
                .fg(app.theme.secondary)
                .add_modifier(Modifier::BOLD)
        } else if *interactive {
            Style::default().fg(app.theme.secondary)
        } else {
            Style::default().fg(app.theme.text_muted)
        };

        lines.push(Line::from(vec![
            cursor_span,
            Span::styled(format!("{:<28}", label), label_style),
            Span::styled(format!(" [ {} ]", value), value_style),
        ]));
        lines.push(Line::from(""));
    }

    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(app.t.settings_title())
        .title_alignment(Alignment::Center)
        .style(app.theme.block_active());

    f.render_widget(Paragraph::new(lines).block(main_block), chunks[0]);

    // Description text for selected option
    let desc_text = match state.selected_index {
        0 => {
            if is_pt {
                "Altere instantaneamente o idioma de toda a aplicação entre Português e English."
            } else {
                "Instantly switch the application language between Portuguese and English."
            }
        }
        1 => {
            if is_pt {
                "Alterne a paleta de cores entre o tema Rust Oxide e Catppuccin Mocha."
            } else {
                "Switch color scheme between Rust Oxide and Catppuccin Mocha themes."
            }
        }
        2 => {
            if is_pt {
                "Gerencie repositórios locais ou remotos SSH, adicione novos ou remova existentes."
            } else {
                "Manage local or remote SSH Borg repositories, add new or remove existing ones."
            }
        }
        3 => {
            if is_pt {
                "Consulte o total deduplicado, chunks únicos e economia real em disco (borg info)."
            } else {
                "Query total deduplication savings, unique chunks, and disk usage (borg info)."
            }
        }
        4 => {
            if is_pt {
                "Abra o visualizador integrado com rolagem, filtros de erro e histórico de comandos."
            } else {
                "Open integrated log viewer with scrolling, error filters, and command history."
            }
        }
        5 => {
            if is_pt {
                "Pasta no disco onde os arquivos restaurados de snapshots são salvos por padrão."
            } else {
                "Default local folder where extracted snapshot files are saved."
            }
        }
        _ => {
            if is_pt {
                "Informações de versão e conectividade com o executável BorgBackup no sistema operacional."
            } else {
                "Version and connectivity status with the system BorgBackup executable."
            }
        }
    };

    let desc_p = Paragraph::new(format!("  \u{2022} {}", desc_text))
        .style(Style::default().fg(app.theme.text_muted))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(app.theme.block_normal()),
        );
    f.render_widget(desc_p, chunks[1]);

    // Footer
    let footer_p = Paragraph::new(app.t.settings_footer())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(app.theme.block_normal()),
        );
    f.render_widget(footer_p, chunks[2]);
}
