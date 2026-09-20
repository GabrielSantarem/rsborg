use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    Rust,
    Catppuccin,
}

impl ThemeMode {
    pub fn toggle(&self) -> Self {
        match self {
            ThemeMode::Rust => ThemeMode::Catppuccin,
            ThemeMode::Catppuccin => ThemeMode::Rust,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::Rust => "Rust Oxide",
            ThemeMode::Catppuccin => "Catppuccin Mocha",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub mode: ThemeMode,
    pub primary: Color,
    pub secondary: Color,
    pub text: Color,
    pub text_muted: Color,
    pub border_normal: Color,
    pub border_active: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
    pub bg_highlight: Color,
}

impl Theme {
    pub fn new(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Rust => Self {
                mode,
                // Paleta inspirada nos tons clássicos do Rust (ferrugem, âmbar, ardósia)
                primary: Color::Rgb(222, 107, 43), // Rust Orange #DE6B2B
                secondary: Color::Rgb(235, 169, 74), // Amber Glow #EBA94A
                text: Color::Rgb(244, 235, 217),   // Warm Cream #F4EBD9
                text_muted: Color::Rgb(168, 153, 132), // Stone Gray #A89984
                border_normal: Color::Rgb(80, 73, 69), // Dark Slate #504945
                border_active: Color::Rgb(222, 107, 43), // Rust Orange
                success: Color::Rgb(142, 192, 124), // Sage Green #8EC07C
                warning: Color::Rgb(250, 189, 47), // Sunburst Yellow #FABD2F
                danger: Color::Rgb(251, 73, 52),   // Red Oxide #FB4934
                info: Color::Rgb(131, 165, 152),   // Sea Foam Teal #83A598
                bg_highlight: Color::Rgb(60, 56, 54), // Deep Charcoal #3C3836
            },
            ThemeMode::Catppuccin => Self {
                mode,
                // Paleta oficial Catppuccin Mocha
                primary: Color::Rgb(203, 166, 247), // Mauve #cba6f7
                secondary: Color::Rgb(137, 180, 250), // Blue #89b4fa
                text: Color::Rgb(205, 214, 244),    // Text #cdd6f4
                text_muted: Color::Rgb(147, 153, 178), // Subtext0 #9399b2
                border_normal: Color::Rgb(69, 71, 90), // Surface1 #45475a
                border_active: Color::Rgb(203, 166, 247), // Mauve
                success: Color::Rgb(166, 227, 161), // Green #a6e3a1
                warning: Color::Rgb(249, 226, 175), // Yellow #f9e2af
                danger: Color::Rgb(243, 139, 168),  // Red #f38ba8
                info: Color::Rgb(148, 226, 213),    // Teal #94e2d5
                bg_highlight: Color::Rgb(49, 50, 68), // Surface0 #313244
            },
        }
    }

    pub fn row_selected_style(&self) -> Style {
        Style::default()
            .bg(self.bg_highlight)
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn block_normal(&self) -> Style {
        Style::default().fg(self.border_normal)
    }

    pub fn block_active(&self) -> Style {
        Style::default().fg(self.border_active)
    }

    pub fn key_badge_style(&self) -> Style {
        Style::default()
            .fg(self.secondary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn desc_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }
}
