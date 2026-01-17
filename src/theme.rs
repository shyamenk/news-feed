use ratatui::style::Color;

#[allow(dead_code)]
pub trait Theme {
    fn base(&self) -> Color;
    fn mantle(&self) -> Color;
    fn text(&self) -> Color;
    fn subtext(&self) -> Color;
    fn overlay(&self) -> Color;
    fn accent_primary(&self) -> Color;
    fn accent_secondary(&self) -> Color;
    fn highlight(&self) -> Color;
    fn surface(&self) -> Color;
    fn warning(&self) -> Color;
    fn success(&self) -> Color;
}

// Claude Code Theme - Deep navy with amber/indigo accents
pub struct ClaudeCodeTheme;

impl Theme for ClaudeCodeTheme {
    fn base(&self) -> Color {
        Color::Rgb(26, 26, 46) // #1a1a2e - deep navy background
    }

    fn mantle(&self) -> Color {
        Color::Rgb(22, 22, 31) // #16161f - darker accents
    }

    fn text(&self) -> Color {
        Color::Rgb(228, 228, 231) // #e4e4e7 - soft white
    }

    fn subtext(&self) -> Color {
        Color::Rgb(161, 161, 170) // #a1a1aa - muted text
    }

    fn overlay(&self) -> Color {
        Color::Rgb(113, 113, 122) // #71717a - dimmed text
    }

    fn accent_primary(&self) -> Color {
        Color::Rgb(217, 119, 6) // #d97706 - amber/orange (Claude brand)
    }

    fn accent_secondary(&self) -> Color {
        Color::Rgb(99, 102, 241) // #6366f1 - indigo
    }

    fn highlight(&self) -> Color {
        Color::Rgb(55, 65, 81) // #374151 - dark gray selection
    }

    fn surface(&self) -> Color {
        Color::Rgb(39, 39, 42) // #27272a - elevated surface
    }

    fn warning(&self) -> Color {
        Color::Rgb(245, 158, 11) // #f59e0b - amber warning
    }

    fn success(&self) -> Color {
        Color::Rgb(16, 185, 129) // #10b981 - emerald green
    }
}

// Catppuccin Mocha Theme - Green accent version (matching quit tracker)
pub struct CatppuccinMochaTheme;

impl Theme for CatppuccinMochaTheme {
    fn base(&self) -> Color {
        Color::Rgb(30, 30, 46) // #1e1e2e - dark background
    }

    fn mantle(&self) -> Color {
        Color::Rgb(24, 24, 37) // #181825 - darker background
    }

    fn text(&self) -> Color {
        Color::Rgb(205, 214, 244) // #cdd6f4 - main text
    }

    fn subtext(&self) -> Color {
        Color::Rgb(186, 194, 222) // #bac2de - muted text
    }

    fn overlay(&self) -> Color {
        Color::Rgb(127, 132, 156) // #7f849c - dimmed text
    }

    fn accent_primary(&self) -> Color {
        Color::Rgb(166, 227, 161) // #a6e3a1 - GREEN (primary accent!)
    }

    fn accent_secondary(&self) -> Color {
        Color::Rgb(250, 179, 135) // #fab387 - peach (active highlights)
    }

    fn highlight(&self) -> Color {
        Color::Rgb(49, 50, 68) // #313244 - surface0 (selection background)
    }

    fn surface(&self) -> Color {
        Color::Rgb(49, 50, 68) // #313244 - surface0
    }

    fn warning(&self) -> Color {
        Color::Rgb(137, 220, 235) // #89dceb - cyan (for numbers/stats)
    }

    fn success(&self) -> Color {
        Color::Rgb(166, 227, 161) // #a6e3a1 - green (success/progress)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ThemeVariant {
    ClaudeCode,
    CatppuccinMocha,
}

#[allow(dead_code)]
impl ThemeVariant {
    pub fn get_theme(&self) -> Box<dyn Theme> {
        match self {
            ThemeVariant::ClaudeCode => Box::new(ClaudeCodeTheme),
            ThemeVariant::CatppuccinMocha => Box::new(CatppuccinMochaTheme),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "claude-code" => ThemeVariant::ClaudeCode,
            "catppuccin-mocha" => ThemeVariant::CatppuccinMocha,
            _ => ThemeVariant::ClaudeCode, // Default to Claude Code
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ThemeVariant::ClaudeCode => "claude-code",
            ThemeVariant::CatppuccinMocha => "catppuccin-mocha",
        }
    }
}
