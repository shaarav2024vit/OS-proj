use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

/// The 4 iconic Linux ricing color palettes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    CatppuccinMocha,
    TokyoNight,
    GruvboxDark,
    Nord,
}

impl ThemeName {
    /// Cycle to the next theme in order
    pub fn next(self) -> Self {
        match self {
            ThemeName::CatppuccinMocha => ThemeName::TokyoNight,
            ThemeName::TokyoNight => ThemeName::GruvboxDark,
            ThemeName::GruvboxDark => ThemeName::Nord,
            ThemeName::Nord => ThemeName::CatppuccinMocha,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeName::CatppuccinMocha => "☕ Catppuccin Mocha",
            ThemeName::TokyoNight => "🌃 Tokyo Night",
            ThemeName::GruvboxDark => "🌲 Gruvbox Dark",
            ThemeName::Nord => "❄️  Nord",
        }
    }
}

/// Polished Linux-rice color palette with full accent set
pub struct Theme {
    pub name: ThemeName,

    pub bg_base: Color,
    pub bg_mantle: Color,
    pub bg_crust: Color,
    pub surface0: Color,
    pub surface1: Color,
    pub overlay: Color,
    pub subtext: Color,
    pub text: Color,

    pub mauve: Color,
    pub blue: Color,
    pub sapphire: Color,
    pub teal: Color,
    pub green: Color,
    pub yellow: Color,
    pub peach: Color,
    pub red: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_name(ThemeName::CatppuccinMocha)
    }
}

impl Theme {
    pub const BORDER_TYPE: BorderType = BorderType::Rounded;

    /// Create a theme from one of the 4 iconic palettes
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::CatppuccinMocha => Self {
                name,
                bg_base:   Color::Rgb(30, 30, 46),       // #1E1E2E
                bg_mantle: Color::Rgb(24, 24, 37),       // #181825
                bg_crust:  Color::Rgb(17, 17, 27),       // #11111B
                surface0:  Color::Rgb(49, 50, 68),       // #313244
                surface1:  Color::Rgb(69, 71, 90),       // #45475A
                overlay:   Color::Rgb(108, 112, 134),    // #6C7086
                subtext:   Color::Rgb(166, 173, 200),    // #A6ADC8
                text:      Color::Rgb(205, 214, 244),    // #CDD6F4
                mauve:     Color::Rgb(203, 166, 247),    // #CBA6F7
                blue:      Color::Rgb(137, 180, 250),    // #89B4FA
                sapphire:  Color::Rgb(116, 199, 236),    // #74C7EC
                teal:      Color::Rgb(148, 226, 213),    // #94E2D5
                green:     Color::Rgb(166, 227, 161),    // #A6E3A1
                yellow:    Color::Rgb(249, 226, 175),    // #F9E2AF
                peach:     Color::Rgb(250, 179, 135),    // #FAB387
                red:       Color::Rgb(243, 139, 168),    // #F38BA8
            },
            ThemeName::TokyoNight => Self {
                name,
                bg_base:   Color::Rgb(26, 27, 38),       // #1A1B26
                bg_mantle: Color::Rgb(22, 22, 30),       // #16161E
                bg_crust:  Color::Rgb(18, 18, 24),       // #121218
                surface0:  Color::Rgb(41, 46, 66),       // #292E42
                surface1:  Color::Rgb(55, 59, 82),       // #373B52
                overlay:   Color::Rgb(86, 95, 137),      // #565F89
                subtext:   Color::Rgb(169, 177, 214),    // #A9B1D6
                text:      Color::Rgb(192, 202, 245),    // #C0CAF5
                mauve:     Color::Rgb(187, 154, 247),    // #BB9AF7 (purple)
                blue:      Color::Rgb(125, 174, 247),    // #7DAEF7
                sapphire:  Color::Rgb(125, 207, 255),    // #7DCFFF (cyan)
                teal:      Color::Rgb(115, 218, 202),    // #73DACA
                green:     Color::Rgb(158, 206, 106),    // #9ECE6A
                yellow:    Color::Rgb(224, 175, 104),    // #E0AF68
                peach:     Color::Rgb(255, 158, 100),    // #FF9E64
                red:       Color::Rgb(247, 118, 142),    // #F7768E
            },
            ThemeName::GruvboxDark => Self {
                name,
                bg_base:   Color::Rgb(40, 40, 40),       // #282828
                bg_mantle: Color::Rgb(29, 32, 33),       // #1D2021
                bg_crust:  Color::Rgb(24, 24, 24),       // #181818
                surface0:  Color::Rgb(60, 56, 54),       // #3C3836
                surface1:  Color::Rgb(80, 73, 69),       // #504945
                overlay:   Color::Rgb(146, 131, 116),    // #928374
                subtext:   Color::Rgb(189, 174, 147),    // #BDAE93
                text:      Color::Rgb(235, 219, 178),    // #EBDBB2
                mauve:     Color::Rgb(211, 134, 155),    // #D3869B (purple)
                blue:      Color::Rgb(131, 165, 152),    // #83A598 (aqua-blue)
                sapphire:  Color::Rgb(131, 165, 152),    // #83A598
                teal:      Color::Rgb(142, 192, 124),    // #8EC07C (aqua)
                green:     Color::Rgb(184, 187, 38),     // #B8BB26
                yellow:    Color::Rgb(250, 189, 47),     // #FABD2F
                peach:     Color::Rgb(254, 128, 25),     // #FE8019
                red:       Color::Rgb(251, 73, 52),      // #FB4934
            },
            ThemeName::Nord => Self {
                name,
                bg_base:   Color::Rgb(46, 52, 64),       // #2E3440
                bg_mantle: Color::Rgb(41, 46, 56),       // #292E38
                bg_crust:  Color::Rgb(36, 40, 48),       // #242830
                surface0:  Color::Rgb(59, 66, 82),       // #3B4252
                surface1:  Color::Rgb(67, 76, 94),       // #434C5E
                overlay:   Color::Rgb(76, 86, 106),      // #4C566A
                subtext:   Color::Rgb(216, 222, 233),    // #D8DEE9
                text:      Color::Rgb(236, 239, 244),    // #ECEFF4
                mauve:     Color::Rgb(180, 142, 173),    // #B48EAD (purple)
                blue:      Color::Rgb(129, 161, 193),    // #81A1C1
                sapphire:  Color::Rgb(136, 192, 208),    // #88C0D0 (frost)
                teal:      Color::Rgb(143, 188, 187),    // #8FBCBB
                green:     Color::Rgb(163, 190, 140),    // #A3BE8C
                yellow:    Color::Rgb(235, 203, 139),    // #EBCB8B
                peach:     Color::Rgb(208, 135, 112),    // #D08770
                red:       Color::Rgb(191, 97, 106),     // #BF616A
            },
        }
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.mauve)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.surface1)
    }

    pub fn border_active_style(&self) -> Style {
        Style::default().fg(self.mauve)
    }

    pub fn directory_style(&self) -> Style {
        Style::default()
            .fg(self.blue)
            .add_modifier(Modifier::BOLD)
    }

    pub fn file_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn encrypted_style(&self) -> Style {
        Style::default()
            .fg(self.peach)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_row_style(&self) -> Style {
        Style::default()
            .bg(self.surface0)
            .fg(self.text)
            .add_modifier(Modifier::BOLD)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.overlay)
    }

    pub fn badge_style(&self, bg: Color, fg: Color) -> Style {
        Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD)
    }

    /// Generates smooth, proportioned usage bar with gradient color
    pub fn format_usage_bar(&self, ratio: f64, width: usize) -> (String, Style) {
        let clamped = ratio.clamp(0.0, 1.0);
        let filled_chars = (clamped * width as f64).round() as usize;
        let empty_chars = width.saturating_sub(filled_chars);

        // Blocks: Smooth full and empty characters
        let bar_filled: String = "█".repeat(filled_chars);
        let bar_empty: String = "░".repeat(empty_chars);
        let bar = format!("{}{}", bar_filled, bar_empty);

        let color = if clamped > 0.75 {
            self.red
        } else if clamped > 0.45 {
            self.peach
        } else if clamped > 0.20 {
            self.yellow
        } else {
            self.teal
        };

        (bar, Style::default().fg(color))
    }

    /// Color scale based on human file sizes
    pub fn size_color(&self, bytes: u64) -> Color {
        const GB: u64 = 1024 * 1024 * 1024;
        const MB: u64 = 1024 * 1024;

        if bytes > 10 * GB {
            self.red
        } else if bytes > 1 * GB {
            self.peach
        } else if bytes > 100 * MB {
            self.yellow
        } else if bytes > 1 * MB {
            self.teal
        } else {
            self.subtext
        }
    }
}
