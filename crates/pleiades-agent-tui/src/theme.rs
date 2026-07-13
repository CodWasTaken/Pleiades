use std::collections::HashMap;

/// Terminal color scheme definition.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: HashMap<String, String>,
}

impl Theme {
    /// Load a theme by name from built-in themes.
    pub fn load(name: &str) -> Option<Self> {
        BUILTIN_THEMES.get(name).cloned()
    }

    /// Get a color value by key.
    pub fn color(&self, key: &str) -> Option<&String> {
        self.colors.get(key)
    }
}

/// Built-in themes collection.
pub static BUILTIN_THEMES: once_cell::sync::Lazy<HashMap<String, Theme>> =
    once_cell::sync::Lazy::new(|| {
        let mut themes = HashMap::new();

        themes.insert(
            "catppuccin-mocha".to_string(),
            Theme {
                name: "Catppuccin Mocha".to_string(),
                colors: HashMap::from([
                    ("background".to_string(), "#1e1e2e".to_string()),
                    ("foreground".to_string(), "#cdd6f4".to_string()),
                    ("accent".to_string(), "#89b4fa".to_string()),
                    ("success".to_string(), "#a6e3a1".to_string()),
                    ("warning".to_string(), "#f9e2af".to_string()),
                    ("error".to_string(), "#f38ba8".to_string()),
                    ("info".to_string(), "#89dceb".to_string()),
                    ("muted".to_string(), "#585b70".to_string()),
                    ("surface".to_string(), "#313244".to_string()),
                ]),
            },
        );

        themes.insert(
            "dracula".to_string(),
            Theme {
                name: "Dracula".to_string(),
                colors: HashMap::from([
                    ("background".to_string(), "#282a36".to_string()),
                    ("foreground".to_string(), "#f8f8f2".to_string()),
                    ("accent".to_string(), "#bd93f9".to_string()),
                    ("success".to_string(), "#50fa7b".to_string()),
                    ("warning".to_string(), "#ffb86c".to_string()),
                    ("error".to_string(), "#ff5555".to_string()),
                    ("info".to_string(), "#8be9fd".to_string()),
                    ("muted".to_string(), "#6272a4".to_string()),
                    ("surface".to_string(), "#44475a".to_string()),
                ]),
            },
        );

        themes.insert(
            "tokyo-night".to_string(),
            Theme {
                name: "Tokyo Night".to_string(),
                colors: HashMap::from([
                    ("background".to_string(), "#1a1b26".to_string()),
                    ("foreground".to_string(), "#a9b1d6".to_string()),
                    ("accent".to_string(), "#7aa2f7".to_string()),
                    ("success".to_string(), "#9ece6a".to_string()),
                    ("warning".to_string(), "#e0af68".to_string()),
                    ("error".to_string(), "#f7768e".to_string()),
                    ("info".to_string(), "#7dcfff".to_string()),
                    ("muted".to_string(), "#565f89".to_string()),
                    ("surface".to_string(), "#24283b".to_string()),
                ]),
            },
        );

        themes
    });
