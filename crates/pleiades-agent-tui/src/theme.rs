//! Typed visual design system for the Pleiades terminal workspace.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Symbols {
    pub agent: &'static str,
    pub suggestion: &'static str,
    pub context: &'static str,
    pub running: &'static str,
    pub success: &'static str,
    pub failure: &'static str,
    pub paused: &'static str,
    pub tool: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub background: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub foreground: Color,
    pub muted: Color,
    pub primary: Color,
    pub info: Color,
    pub starlight: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub diff_add: Color,
    pub diff_remove: Color,
    pub border: Color,
    pub symbols: Symbols,
}

const UNICODE_SYMBOLS: Symbols = Symbols {
    agent: "✦",
    suggestion: "✧",
    context: "⋆",
    running: "◌",
    success: "✓",
    failure: "×",
    paused: "◇",
    tool: "⊹",
};

const ASCII_SYMBOLS: Symbols = Symbols {
    agent: "*",
    suggestion: "+",
    context: ".",
    running: "o",
    success: "+",
    failure: "x",
    paused: "-",
    tool: ">",
};

impl Theme {
    /// Load a built-in theme. Previous theme names remain compatible.
    pub fn load(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "seven-sisters" | "catppuccin-mocha" => Some(Self::seven_sisters()),
            "andromeda" | "dracula" => Some(Self::andromeda()),
            "orion" | "tokyo-night" => Some(Self::orion()),
            "event-horizon" => Some(Self::event_horizon()),
            "solar-wind" => Some(Self::solar_wind()),
            "high-contrast" => Some(Self::high_contrast(false)),
            "ascii" => Some(Self::high_contrast(true)),
            _ => None,
        }
    }

    pub fn seven_sisters() -> Self {
        Self {
            name: "seven-sisters",
            background: Color::Rgb(7, 10, 24),
            surface: Color::Rgb(15, 20, 42),
            surface_alt: Color::Rgb(23, 29, 57),
            foreground: Color::Rgb(226, 232, 255),
            muted: Color::Rgb(119, 131, 166),
            primary: Color::Rgb(142, 132, 255),
            info: Color::Rgb(91, 210, 238),
            starlight: Color::Rgb(255, 221, 160),
            success: Color::Rgb(116, 224, 174),
            warning: Color::Rgb(246, 190, 102),
            error: Color::Rgb(255, 112, 132),
            diff_add: Color::Rgb(96, 200, 144),
            diff_remove: Color::Rgb(239, 103, 125),
            border: Color::Rgb(63, 73, 112),
            symbols: UNICODE_SYMBOLS,
        }
    }

    fn andromeda() -> Self {
        Self {
            primary: Color::Rgb(203, 135, 255),
            info: Color::Rgb(119, 221, 255),
            ..Self::seven_sisters()
        }
    }

    fn orion() -> Self {
        Self {
            primary: Color::Rgb(104, 158, 255),
            starlight: Color::Rgb(255, 184, 120),
            ..Self::seven_sisters()
        }
    }

    fn event_horizon() -> Self {
        Self {
            background: Color::Black,
            surface: Color::Rgb(20, 20, 20),
            primary: Color::Rgb(190, 120, 255),
            ..Self::seven_sisters()
        }
    }

    fn solar_wind() -> Self {
        Self {
            primary: Color::Rgb(255, 176, 74),
            info: Color::Rgb(96, 220, 205),
            starlight: Color::Rgb(255, 231, 150),
            ..Self::seven_sisters()
        }
    }

    fn high_contrast(ascii: bool) -> Self {
        Self {
            name: if ascii { "ascii" } else { "high-contrast" },
            background: Color::Black,
            surface: Color::Black,
            surface_alt: Color::DarkGray,
            foreground: Color::White,
            muted: Color::Gray,
            primary: Color::Cyan,
            info: Color::LightCyan,
            starlight: Color::Yellow,
            success: Color::LightGreen,
            warning: Color::LightYellow,
            error: Color::LightRed,
            diff_add: Color::Green,
            diff_remove: Color::Red,
            border: Color::Gray,
            symbols: if ascii {
                ASCII_SYMBOLS
            } else {
                UNICODE_SYMBOLS
            },
        }
    }

    pub fn base(self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }
    pub fn title(self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }
    pub fn muted(self) -> Style {
        Style::default().fg(self.muted)
    }
    pub fn focused_border(self) -> Style {
        Style::default().fg(self.primary)
    }
    pub fn border(self) -> Style {
        Style::default().fg(self.border)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::seven_sisters()
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn default_is_seven_sisters() {
        assert_eq!(Theme::default().name, "seven-sisters");
    }

    #[test]
    fn legacy_theme_names_still_load() {
        assert!(Theme::load("dracula").is_some());
        assert!(Theme::load("tokyo-night").is_some());
    }
}
