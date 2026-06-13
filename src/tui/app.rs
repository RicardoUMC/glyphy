use std::io::stdout;
use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::config::Config;
use crate::processing::GlyphBuffer;
use crate::tui::keys::KeyAction;

/// Character ramp presets for cycling.
///
/// Index 0 is the default from `Config`.
const RAMP_PRESETS: &[&str] = &[
    " .:-=+*#%@",
    " ░▒▓█",
    "@%#*+=-:. ",
    "█▓▒░ ",
];

/// TUI application state.
///
/// Owns the current `Config`, a cached `GlyphBuffer`, and UI flags.
/// The event loop (in Phase 2) calls `handle_action` on each key
/// press, then draws the buffer via ratatui widgets.
pub struct App {
    /// Current rendering configuration.
    pub config: Config,
    /// Cached glyph buffer from the last successful `process()` call.
    pub buffer: Option<GlyphBuffer>,
    /// Path to the image file being viewed.
    pub image_path: PathBuf,
    /// Whether the event loop should keep running.
    pub running: bool,
    /// Whether the help dialog overlay is visible.
    pub show_help: bool,
    /// Most recent error message, displayed in the UI.
    pub last_error: Option<String>,
}

impl App {
    /// Create a new `App` and load the initial image.
    ///
    /// Calls `process()` immediately to populate the buffer.
    /// Returns an error if the image cannot be loaded or processed.
    pub fn new(path: &Path, config: Config) -> Result<Self> {
        let mut app = Self {
            config,
            buffer: None,
            image_path: path.to_path_buf(),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.process()?;
        Ok(app)
    }

    /// (Re)load the image with the current config.
    ///
    /// Calls `glyphy::process_image` and caches the result.
    /// On failure, stores the error string in `last_error`.
    pub fn process(&mut self) -> Result<()> {
        match crate::process_image(&self.image_path, &self.config) {
            Ok(buffer) => {
                self.buffer = Some(buffer);
                self.last_error = None;
                Ok(())
            }
            Err(e) => {
                let msg = e.to_string();
                self.last_error = Some(msg.clone());
                Err(e)
            }
        }
    }

    /// Apply a key action, updating config and state.
    ///
    /// For config-changing actions, re-processes the image automatically.
    pub fn handle_action(&mut self, action: KeyAction) {
        let needs_reprocess = match action {
            KeyAction::Quit => {
                self.running = false;
                false
            }
            KeyAction::WidthUp => {
                let w = self.config.width.unwrap_or(80).saturating_add(10);
                self.config.width = Some(w);
                true
            }
            KeyAction::WidthDown => {
                let w = self.config.width.unwrap_or(80).saturating_sub(10).max(10);
                self.config.width = Some(w);
                true
            }
            KeyAction::HeightUp => {
                let h = self.config.height.unwrap_or(40).saturating_add(5);
                self.config.height = Some(h);
                true
            }
            KeyAction::HeightDown => {
                let h = self.config.height.unwrap_or(40).saturating_sub(5).max(5);
                self.config.height = Some(h);
                true
            }
            KeyAction::CycleRamp => {
                self.config.ramp = next_ramp(&self.config.ramp);
                true
            }
            KeyAction::ToggleInvert => {
                self.config.invert = !self.config.invert;
                true
            }
            KeyAction::ToggleHelp => {
                self.show_help = !self.show_help;
                false
            }
        };

        if needs_reprocess {
            // Ignore errors here — store in last_error for UI display.
            let _ = self.process();
        }
    }

    /// Run the TUI event loop.
    ///
    /// Enables raw mode, enters the alternate screen, and starts a
    /// draw–poll loop that continues until the app signals quit.
    /// Terminal state is always restored on exit (success or error).
    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal);

        // Restore terminal state regardless of errors.
        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = terminal.show_cursor();

        result
    }

    /// Inner event loop: draw, poll, handle, repeat.
    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        while self.running {
            terminal.draw(|f| crate::tui::render::render(f, self))?;
            crate::tui::event::handle_events(self)?;
        }
        Ok(())
    }
}

/// Cycle to the next ramp preset, wrapping around.
fn next_ramp(current: &[char]) -> Vec<char> {
    let current_str: String = current.iter().collect();
    for (i, preset) in RAMP_PRESETS.iter().enumerate() {
        if *preset == current_str {
            let next = RAMP_PRESETS[(i + 1) % RAMP_PRESETS.len()];
            return next.chars().collect();
        }
    }
    // Unknown ramp — reset to default.
    RAMP_PRESETS[0].chars().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    /// The default ramp at index 0 of RAMP_PRESETS.
    fn default_ramp_str() -> String {
        RAMP_PRESETS[0].chars().collect()
    }

    #[test]
    fn handle_action_quit_sets_running_false() {
        let config = Config::default();
        // We can't construct App::new without a real image, so test
        // the handle_action logic directly by constructing manually.
        let mut app = App {
            config,
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::Quit);
        assert!(!app.running);
    }

    #[test]
    fn handle_action_width_up() {
        let config = Config::default();
        let mut app = App {
            config: Config { width: Some(80), ..config },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::WidthUp);
        assert_eq!(app.config.width, Some(90));
    }

    #[test]
    fn handle_action_width_down() {
        let config = Config::default();
        let mut app = App {
            config: Config { width: Some(80), ..config },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::WidthDown);
        assert_eq!(app.config.width, Some(70));
    }

    #[test]
    fn handle_action_width_down_floor() {
        let config = Config::default();
        let mut app = App {
            config: Config { width: Some(10), ..config },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::WidthDown);
        assert_eq!(app.config.width, Some(10));
    }

    #[test]
    fn handle_action_height_up() {
        let config = Config::default();
        let mut app = App {
            config: Config { height: Some(40), ..config },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::HeightUp);
        assert_eq!(app.config.height, Some(45));
    }

    #[test]
    fn handle_action_height_down() {
        let config = Config::default();
        let mut app = App {
            config: Config { height: Some(40), ..config },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::HeightDown);
        assert_eq!(app.config.height, Some(35));
    }

    #[test]
    fn handle_action_height_down_floor() {
        let config = Config::default();
        let mut app = App {
            config: Config { height: Some(5), ..config },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::HeightDown);
        assert_eq!(app.config.height, Some(5));
    }

    #[test]
    fn handle_action_cycle_ramp() {
        let config = Config::default();
        let ramp = default_ramp_str();
        let ramp_chars: Vec<char> = ramp.chars().collect();
        let mut app = App {
            config: Config { ramp: ramp_chars, ..config },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::CycleRamp);
        let expected: String = RAMP_PRESETS[1].chars().collect();
        let actual: String = app.config.ramp.iter().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn handle_action_toggle_invert() {
        let config = Config::default();
        let mut app = App {
            config: Config { invert: false, ..config },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::ToggleInvert);
        assert!(app.config.invert);
        app.handle_action(KeyAction::ToggleInvert);
        assert!(!app.config.invert);
    }

    #[test]
    fn handle_action_toggle_help() {
        let config = Config::default();
        let mut app = App {
            config,
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            last_error: None,
        };
        app.handle_action(KeyAction::ToggleHelp);
        assert!(app.show_help);
        app.handle_action(KeyAction::ToggleHelp);
        assert!(!app.show_help);
    }

    #[test]
    fn cycle_ramp_wraps_around() {
        // Start from the last preset and cycle — should return to first.
        let last_preset = RAMP_PRESETS[RAMP_PRESETS.len() - 1];
        let current: Vec<char> = last_preset.chars().collect();
        let next = next_ramp(&current);
        let expected: String = RAMP_PRESETS[0].chars().collect();
        let actual: String = next.iter().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn cycle_ramp_unknown_resets_to_default() {
        let unknown: Vec<char> = "abc".chars().collect();
        let next = next_ramp(&unknown);
        let expected: String = RAMP_PRESETS[0].chars().collect();
        let actual: String = next.iter().collect();
        assert_eq!(actual, expected);
    }
}
