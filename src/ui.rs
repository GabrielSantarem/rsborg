use ratatui::{
    crossterm::style::style,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, CurrentScreen, CurrentlyEditing};

pub fn ui(f: &mut ratatui::Frame<'_>, app: &mut App) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Create new JSON",
        Style::default().fg(ratatui::style::Color::Green),
    ))
    .block(title_block);

    f.render_widget(title, chunks[0]);

    let mut list_items = Vec::<ListItem>::new();

    for key in app.pairs.keys() {
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!("{: <25} : {}", key, app.pairs[key],),
            Style::default().fg(ratatui::style::Color::Yellow),
        ))));
    }

    let list = List::new(list_items);

    f.render_widget(list, chunks[1]);

    let current_navigation_text = vec![
        match app.current_screen {
            CurrentScreen::Main => Span::styled(
                "Normal mode ",
                Style::default().fg(ratatui::style::Color::Green),
            ),
            CurrentScreen::Editing => Span::styled(
                "Editing mode",
                Style::default().fg(ratatui::style::Color::Yellow),
            ),
            CurrentScreen::Exiting => Span::styled(
                "Exiting",
                Style::default().fg(ratatui::style::Color::LightRed),
            ),
        }
        .to_owned(),
        Span::styled(" | ", Style::default().fg(ratatui::style::Color::White)),
        {
            if let Some(editing) = &app.currently_editing {
                match editing {
                    CurrentlyEditing::Key => Span::styled(
                        "Editng json key",
                        Style::default().fg(ratatui::style::Color::Green),
                    ),
                    CurrentlyEditing::Value => Span::styled(
                        "Editing Json Value",
                        Style::default().fg(ratatui::style::Color::LightGreen),
                    ),
                }
            } else {
                Span::styled(
                    "Not editng anything",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                )
            }
        },
    ];

    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::all()));

    let current_keys_hint = {
        match app.current_screen {
            CurrentScreen::Main => Span::styled(
                "(q) to quit / (e) to make new pair",
                Style::default().fg(ratatui::style::Color::Red),
            ),
            CurrentScreen::Editing => Span::styled(
                "(ESC) to cancel/(Tab) to switch boxes/enter to complete",
                Style::default().fg(ratatui::style::Color::Red),
            ),
            CurrentScreen::Exiting => Span::styled(
                "(q) to quit / (e) to make new pair",
                Style::default().fg(ratatui::style::Color::Red),
            ),
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    f.render_widget(mode_footer, footer_chunks[0]);
    f.render_widget(key_notes_footer, footer_chunks[1]);

    if let Some(edting) = &app.currently_editing {
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(ratatui::style::Color::DarkGray));

        let area = centered_rect(60, 25, f.area());
        f.render_widget(popup_block, area);

        let popup_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let mut key_block = Block::default().title("Key").borders(Borders::ALL);
        let mut value_block = Block::default().title("Value").borders(Borders::ALL);

        let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

        match edting {
            CurrentlyEditing::Key => key_block = key_block.style(active_style),
            CurrentlyEditing::Value => value_block = value_block.style(active_style),
        }

        let key_text = Paragraph::new(app.key_input.clone()).block(key_block);
        f.render_widget(key_text, popup_chunks[0]);

        let value_text = Paragraph::new(app.value_input.clone()).block(value_block);
        f.render_widget(value_text, popup_chunks[1]);
    }

    if let CurrentScreen::Exiting = app.current_screen {
        f.render_widget(Clear, f.area());

        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exited_text = Text::styled(
            "Would you like to output the buffer as json? (y/n)",
            Style::default().fg(Color::Red),
        );

        let exit_paragraph = Paragraph::new(exited_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, f.area());

        f.render_widget(exit_paragraph, area);
    }
}
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
