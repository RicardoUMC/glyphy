use ratatui::style::{Color, Style, Stylize};

/// Available TUI chrome theme modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

/// Semantic styles used by the TUI chrome.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub mode: ThemeMode,
    pub chrome: Style,
    pub text: Style,
    pub accent: Style,
    pub muted: Style,
    pub error: Style,
    pub focused_border: Style,
    pub file_border: Style,
    pub directory: Style,
    pub selection: Style,
    pub picker_badge: Style,
    pub view_badge: Style,
    pub settings_on: Style,
    pub settings_off: Style,
    pub help_overlay: Style,
}

impl Theme {
    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }

    fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            chrome: Style::default().bg(Color::Rgb(24, 26, 33)),
            text: Style::default().fg(Color::Gray),
            accent: Style::default().fg(Color::Cyan).bold(),
            muted: Style::default().fg(Color::DarkGray),
            error: Style::default().fg(Color::Red),
            focused_border: Style::default().fg(Color::Cyan),
            file_border: Style::default().fg(Color::Green),
            directory: Style::default().fg(Color::Yellow),
            selection: Style::default().fg(Color::Black).bg(Color::Green),
            picker_badge: Style::default().fg(Color::Black).bg(Color::Green).bold(),
            view_badge: Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
            settings_on: Style::default().fg(Color::Yellow),
            settings_off: Style::default().fg(Color::DarkGray),
            help_overlay: Style::default().bg(Color::Rgb(24, 26, 33)).fg(Color::Gray),
        }
    }

    fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            chrome: Style::default().bg(Color::Rgb(232, 228, 218)),
            text: Style::default().fg(Color::Black),
            accent: Style::default().fg(Color::Blue).bold(),
            muted: Style::default().fg(Color::DarkGray),
            error: Style::default().fg(Color::Red),
            focused_border: Style::default().fg(Color::Blue),
            file_border: Style::default().fg(Color::Green),
            directory: Style::default().fg(Color::Rgb(128, 92, 0)),
            selection: Style::default().fg(Color::White).bg(Color::Blue),
            picker_badge: Style::default().fg(Color::White).bg(Color::Green).bold(),
            view_badge: Style::default().fg(Color::White).bg(Color::Blue).bold(),
            settings_on: Style::default().fg(Color::Rgb(128, 92, 0)),
            settings_off: Style::default().fg(Color::DarkGray),
            help_overlay: Style::default()
                .bg(Color::Rgb(248, 245, 236))
                .fg(Color::Black),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_theme_has_distinguishable_core_states() {
        let theme = Theme::from_mode(ThemeMode::Dark);

        assert_eq!(theme.mode, ThemeMode::Dark);
        assert_ne!(theme.text.fg, theme.muted.fg);
        assert_ne!(theme.selection.bg, theme.chrome.bg);
        assert_ne!(theme.error.fg, theme.accent.fg);
    }

    #[test]
    fn light_theme_uses_readable_selection_and_chrome() {
        let theme = Theme::from_mode(ThemeMode::Light);

        assert_eq!(theme.mode, ThemeMode::Light);
        assert_ne!(theme.text.fg, theme.muted.fg);
        assert_ne!(theme.selection.fg, theme.selection.bg);
        assert_ne!(theme.selection.bg, theme.chrome.bg);
    }

    #[test]
    fn theme_modes_use_different_chrome_palettes() {
        let dark = Theme::from_mode(ThemeMode::Dark);
        let light = Theme::from_mode(ThemeMode::Light);

        assert_ne!(dark.chrome.bg, light.chrome.bg);
        assert_ne!(dark.text.fg, light.text.fg);
    }
}
