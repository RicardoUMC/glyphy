use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
};
use image::GenericImageView;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

use crate::config::Config;
use crate::processing::GlyphBuffer;
use crate::tui::keys::KeyAction;

const MAX_DIMENSION: u32 = 2000;

/// Supported image file extensions for the file picker.
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, Show);
    }
}

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
    /// Whether size should follow the terminal dimensions.
    pub auto_size: bool,
    /// Whether the UI needs to be redrawn.
    pub dirty: bool,
    /// Most recent error message, displayed in the UI.
    pub last_error: Option<String>,
    /// Whether we're in file picker mode (no image loaded yet).
    pub picker_mode: bool,
    /// List of image files found in CWD for the file picker.
    pub picker_files: Vec<PathBuf>,
    /// Currently selected index in the file picker.
    pub picker_index: usize,
    /// Current focused panel: 'f'ile, 's'ettings, 'o'utput.
    pub focus: char,
}

impl App {
    /// Create a new `App` and load the initial image.
    ///
    /// Calls `process()` immediately to populate the buffer.
    /// Returns an error if the image cannot be loaded or processed.
    pub fn new(path: &Path, config: Config) -> Result<Self> {
        let auto_size = config.width.is_none() || config.height.is_none();
        let mut app = Self {
            config,
            buffer: None,
            image_path: path.to_path_buf(),
            running: true,
            show_help: false,
            auto_size,
            dirty: true,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
        };
        app.process()?;
        Ok(app)
    }

    /// Create a new `App` in file picker mode (no image loaded).
    ///
    /// Scans CWD for image files and displays the picker.
    pub fn new_picker(config: Config) -> Result<Self> {
        let files = Self::scan_cwd_images();
        let mut app = Self {
            config,
            buffer: None,
            image_path: PathBuf::new(),
            running: true,
            show_help: false,
            auto_size: true,
            dirty: true,
            last_error: None,
            picker_mode: true,
            picker_files: files,
            picker_index: 0,
            focus: 'f',
        };
        // Auto-select first file if available
        if !app.picker_files.is_empty() {
            app.image_path = app.picker_files[0].clone();
            let _ = app.process();
        }
        Ok(app)
    }

    /// Scan CWD for image files (png, jpg, jpeg, webp, gif, bmp).
    fn scan_cwd_images() -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                            files.push(path);
                        }
                    }
                }
            }
        }
        files.sort();
        files
    }

    /// Load the currently selected file from the picker.
    fn picker_select(&mut self) {
        if let Some(path) = self.picker_files.get(self.picker_index) {
            self.image_path = path.clone();
            self.picker_mode = false;
            self.auto_size = true;
            self.config.width = None;
            self.config.height = None;
            self.focus = 'o';
            let _ = self.process();
        }
    }

    /// (Re)load the image with the current config.
    ///
    /// Calls `glyphy::process_image` and caches the result.
    /// On failure, stores the error string in `last_error`.
    pub fn process(&mut self) -> Result<()> {
        let config = self.resolve_tui_config()?;
        match crate::process_image(&self.image_path, &config) {
            Ok(buffer) => {
                self.config.width = Some(buffer.width as u32);
                self.config.height = Some(buffer.height as u32);
                self.buffer = Some(buffer);
                self.last_error = None;
                self.dirty = true;
                Ok(())
            }
            Err(e) => {
                let msg = e.to_string();
                self.last_error = Some(msg.clone());
                self.dirty = true;
                Err(e)
            }
        }
    }

    fn resolve_tui_config(&self) -> Result<Config> {
        let mut config = self.config.clone();
        if !self.auto_size && config.width.is_some() && config.height.is_some() {
            return Ok(config);
        }

        let Ok((term_w, term_h)) = size() else {
            return Ok(config);
        };

        let area = crate::tui::render::image_inner_area(Rect::new(0, 0, term_w, term_h));
        if self.auto_size || config.width.is_none() {
            config.width = Some(u32::from(area.width).max(1));
        }

        if self.auto_size || config.height.is_none() {
            let image = image::open(&self.image_path)?;
            let (orig_w, orig_h) = image.dimensions();
            let target_w = config.width.unwrap_or(orig_w);
            let aspect = orig_h as f32 / orig_w as f32;
            let computed_h = (target_w as f32 * aspect * 0.5) as u32;
            config.height = Some(computed_h.min(u32::from(area.height)).max(1));
        }

        Ok(config)
    }

    /// Apply a key action, updating config and state.
    ///
    /// For config-changing actions, re-processes the image automatically.
    pub fn handle_action(&mut self, action: KeyAction) {
        self.dirty = true;

        let needs_reprocess = match action {
            KeyAction::Quit => {
                self.running = false;
                false
            }
            KeyAction::WidthUp => {
                if self.picker_mode {
                    return;
                }
                self.auto_size = false;
                let w = self
                    .config
                    .width
                    .unwrap_or(80)
                    .saturating_add(10)
                    .min(MAX_DIMENSION);
                self.config.width = Some(w);
                true
            }
            KeyAction::WidthDown => {
                if self.picker_mode {
                    return;
                }
                self.auto_size = false;
                let w = self.config.width.unwrap_or(80).saturating_sub(10).max(10);
                self.config.width = Some(w);
                true
            }
            KeyAction::HeightUp => {
                if self.picker_mode {
                    // In picker mode, move up in file list
                    if self.picker_index > 0 {
                        self.picker_index -= 1;
                        self.image_path = self.picker_files[self.picker_index].clone();
                        let _ = self.process();
                    }
                    return;
                }
                self.auto_size = false;
                let h = self
                    .config
                    .height
                    .unwrap_or(40)
                    .saturating_add(5)
                    .min(MAX_DIMENSION);
                self.config.height = Some(h);
                true
            }
            KeyAction::HeightDown => {
                if self.picker_mode {
                    // In picker mode, move down in file list
                    if self.picker_index + 1 < self.picker_files.len() {
                        self.picker_index += 1;
                        self.image_path = self.picker_files[self.picker_index].clone();
                        let _ = self.process();
                    }
                    return;
                }
                self.auto_size = false;
                let h = self.config.height.unwrap_or(40).saturating_sub(5).max(5);
                self.config.height = Some(h);
                true
            }
            KeyAction::CycleRamp => {
                if self.picker_mode {
                    return;
                }
                self.config.next_ramp();
                true
            }
            KeyAction::ToggleInvert => {
                if self.picker_mode {
                    return;
                }
                self.config.invert = !self.config.invert;
                true
            }
            KeyAction::ToggleHelp => {
                self.show_help = !self.show_help;
                false
            }
            KeyAction::NavConfirm => {
                if self.picker_mode && !self.picker_files.is_empty() {
                    self.picker_select();
                }
                false
            }
            KeyAction::FocusFile => {
                self.focus = 'f';
                false
            }
            KeyAction::FocusSettings => {
                self.focus = 's';
                false
            }
            KeyAction::FocusOutput => {
                self.focus = 'o';
                false
            }
            KeyAction::BackToPicker => {
                if !self.picker_mode && !self.picker_files.is_empty() {
                    self.picker_mode = true;
                    self.buffer = None;
                    self.focus = 'f';
                    self.auto_size = true;
                    self.config.width = None;
                    self.config.height = None;
                    // Re-scan CWD in case files changed
                    self.picker_files = Self::scan_cwd_images();
                    if !self.picker_files.is_empty() {
                        self.picker_index = 0;
                        self.image_path = self.picker_files[0].clone();
                        let _ = self.process();
                    }
                }
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
        let _guard = TerminalGuard;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        self.run_loop(&mut terminal)
    }

    /// Inner event loop: draw, poll, handle, repeat.
    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        while self.running {
            if self.dirty {
                terminal.draw(|f| crate::tui::render::render(f, self))?;
                self.dirty = false;
            }
            crate::tui::event::handle_events(self)?;
        }
        Ok(())
    }
}

/// Cycle to the next ramp preset, wrapping around.
#[cfg(test)]
fn next_ramp(current: &[char]) -> Vec<char> {
    let current_str: String = current.iter().collect();
    for (i, preset) in crate::config::RAMP_PRESETS.iter().enumerate() {
        if *preset == current_str {
            let next = crate::config::RAMP_PRESETS[(i + 1) % crate::config::RAMP_PRESETS.len()];
            return next.chars().collect();
        }
    }
    // Unknown ramp — reset to default.
    crate::config::RAMP_PRESETS[0].chars().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, RAMP_PRESETS};

    /// The default ramp at index 0 of RAMP_PRESETS.
    fn default_ramp_str() -> String {
        RAMP_PRESETS[0].chars().collect()
    }

    #[test]
    fn handle_action_quit_sets_running_false() {
        let config = Config::default();
        let mut app = App {
            config,
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
        };
        app.handle_action(KeyAction::Quit);
        assert!(!app.running);
    }

    #[test]
    fn handle_action_width_up() {
        let config = Config::default();
        let mut app = App {
            config: Config {
                width: Some(80),
                ..config
            },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
        };
        app.handle_action(KeyAction::WidthUp);
        assert_eq!(app.config.width, Some(90));
    }

    #[test]
    fn handle_action_width_down() {
        let config = Config::default();
        let mut app = App {
            config: Config {
                width: Some(80),
                ..config
            },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
        };
        app.handle_action(KeyAction::WidthDown);
        assert_eq!(app.config.width, Some(70));
    }

    #[test]
    fn handle_action_width_down_floor() {
        let config = Config::default();
        let mut app = App {
            config: Config {
                width: Some(10),
                ..config
            },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
        };
        app.handle_action(KeyAction::WidthDown);
        assert_eq!(app.config.width, Some(10));
    }

    #[test]
    fn handle_action_height_up() {
        let config = Config::default();
        let mut app = App {
            config: Config {
                height: Some(40),
                ..config
            },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
        };
        app.handle_action(KeyAction::HeightUp);
        assert_eq!(app.config.height, Some(45));
    }

    #[test]
    fn handle_action_height_down() {
        let config = Config::default();
        let mut app = App {
            config: Config {
                height: Some(40),
                ..config
            },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
        };
        app.handle_action(KeyAction::HeightDown);
        assert_eq!(app.config.height, Some(35));
    }

    #[test]
    fn handle_action_height_down_floor() {
        let config = Config::default();
        let mut app = App {
            config: Config {
                height: Some(5),
                ..config
            },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
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
            config: Config {
                ramp: ramp_chars,
                ..config
            },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
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
            config: Config {
                invert: false,
                ..config
            },
            buffer: None,
            image_path: PathBuf::from("test.png"),
            running: true,
            show_help: false,
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
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
            auto_size: false,
            dirty: false,
            last_error: None,
            picker_mode: false,
            picker_files: Vec::new(),
            picker_index: 0,
            focus: 'o',
        };
        app.handle_action(KeyAction::ToggleHelp);
        assert!(app.show_help);
        app.handle_action(KeyAction::ToggleHelp);
        assert!(!app.show_help);
    }

    #[test]
    fn cycle_ramp_wraps_around() {
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
