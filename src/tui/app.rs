use std::fs;
use std::io::stdout;
use std::path::{Component, Path, PathBuf};

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
use crate::tui::theme::ThemeMode;

const MAX_DIMENSION: u32 = 2000;

/// Supported image file extensions for the file picker.
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];

/// An entry in the file picker: either a directory or an image file.
pub enum PickerEntry {
    Dir {
        path: PathBuf,
        is_parent: bool, // true for the ".." entry
    },
    File(PathBuf),
}

impl PickerEntry {
    /// Display name for the entry (dirs get trailing "/", parent shows "..").
    pub fn name(&self) -> String {
        match self {
            PickerEntry::Dir { path, is_parent } => {
                if *is_parent {
                    "../".to_string()
                } else {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    format!("{}/", name)
                }
            }
            PickerEntry::File(p) => p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string(),
        }
    }

    /// Returns true if this entry is a directory.
    pub fn is_dir(&self) -> bool {
        matches!(self, PickerEntry::Dir { .. })
    }

    /// Returns the full path of the entry.
    pub fn path(&self) -> &Path {
        match self {
            PickerEntry::Dir { path, .. } | PickerEntry::File(path) => path,
        }
    }
}

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
    /// List of entries (directories and image files) in CWD for the file picker.
    pub picker_entries: Vec<PickerEntry>,
    /// Current working directory for the file picker (virtual, NOT the process CWD).
    pub picker_cwd: PathBuf,
    /// Currently selected index in the file picker.
    pub picker_index: usize,
    /// Current focused panel: 'f'ile, 's'ettings, 'o'utput.
    pub focus: char,
    /// Current TUI chrome theme mode.
    pub theme_mode: ThemeMode,
}

impl App {
    /// Create a new `App` and load the initial image.
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
            picker_entries: Vec::new(),
            picker_cwd: normalize_path(std::env::current_dir().unwrap_or_default()),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
        };
        app.process()?;
        Ok(app)
    }

    /// Create a new `App` in file picker mode (no image loaded).
    pub fn new_picker(config: Config) -> Result<Self> {
        let cwd = normalize_path(std::env::current_dir().unwrap_or_default());
        let entries = Self::scan_cwd_entries(&cwd);
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
            picker_entries: entries,
            picker_cwd: cwd,
            picker_index: 0,
            focus: 'f',
            theme_mode: ThemeMode::Dark,
        };
        // Auto-select first file if available
        if let Some(PickerEntry::File(path)) = app.picker_entries.first() {
            app.image_path = path.clone();
            let _ = app.process();
        }
        Ok(app)
    }

    /// Scan the given directory for subdirectories and image files.
    /// Returns directories first (alphabetical), then files (alphabetical).
    /// Includes ".." entry when not at filesystem root.
    fn scan_cwd_entries(cwd: &Path) -> Vec<PickerEntry> {
        let cwd = normalize_path(cwd.to_path_buf());
        let mut entries = Vec::new();

        // Add ".." if not at root — resolve to actual parent path
        if let Some(parent) = cwd.parent() {
            entries.push(PickerEntry::Dir {
                path: normalize_path(parent.to_path_buf()),
                is_parent: true,
            });
        }

        if let Ok(dir_entries) = fs::read_dir(&cwd) {
            for entry in dir_entries.flatten() {
                let path = normalize_path(entry.path());
                if path.is_dir() {
                    entries.push(PickerEntry::Dir {
                        path,
                        is_parent: false,
                    });
                } else if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                            entries.push(PickerEntry::File(path));
                        }
                    }
                }
            }
        }

        // Sort: parent ("..") always first, then dirs (alphabetical), then files (alphabetical)
        entries.sort_by(|a, b| {
            let a_is_parent = matches!(
                a,
                PickerEntry::Dir {
                    is_parent: true,
                    ..
                }
            );
            let b_is_parent = matches!(
                b,
                PickerEntry::Dir {
                    is_parent: true,
                    ..
                }
            );
            if a_is_parent {
                return std::cmp::Ordering::Less;
            }
            if b_is_parent {
                return std::cmp::Ordering::Greater;
            }
            match (a.is_dir(), b.is_dir()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name().to_lowercase().cmp(&b.name().to_lowercase()),
            }
        });

        entries
    }

    /// Load the currently selected entry from the picker.
    fn picker_select(&mut self) {
        if let Some(entry) = self.picker_entries.get(self.picker_index) {
            match entry {
                PickerEntry::Dir { path, is_parent } => {
                    let target = if *is_parent {
                        path.clone()
                    } else if path.is_absolute() {
                        path.clone()
                    } else {
                        self.picker_cwd.join(path)
                    };

                    // Virtual path tracking only — do NOT call set_current_dir
                    self.picker_cwd = normalize_path(target);
                    self.picker_entries = Self::scan_cwd_entries(&self.picker_cwd);
                    self.picker_index = 0;
                    self.dirty = true;
                }
                PickerEntry::File(path) => {
                    self.image_path = path.clone();
                    self.picker_mode = false;
                    self.auto_size = true;
                    self.config.width = None;
                    self.config.height = None;
                    self.focus = 'o';
                    let _ = self.process();
                }
            }
        }
    }

    /// (Re)load the image with the current config.
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
                    if self.picker_index > 0 {
                        self.picker_index -= 1;
                        self.dirty = true;
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
                    if self.picker_index + 1 < self.picker_entries.len() {
                        self.picker_index += 1;
                        self.dirty = true;
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
                if self.picker_mode && !self.picker_entries.is_empty() {
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
                if !self.picker_mode && !self.picker_entries.is_empty() {
                    self.picker_mode = true;
                    self.buffer = None;
                    self.focus = 'f';
                    self.auto_size = true;
                    self.config.width = None;
                    self.config.height = None;
                    self.picker_entries = Self::scan_cwd_entries(&self.picker_cwd);
                    self.picker_index = 0;
                    self.dirty = true;
                }
                false
            }
        };

        if needs_reprocess {
            let _ = self.process();
        }
    }

    /// Run the TUI event loop.
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

/// Normalize a path lexically without touching the filesystem.
///
/// This removes `.` and resolves `..` components without using
/// `canonicalize()`, which can add Windows `\\?\` prefixes and requires
/// filesystem access.
fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
            picker_entries: Vec::new(),
            picker_cwd: PathBuf::from("."),
            picker_index: 0,
            focus: 'o',
            theme_mode: ThemeMode::Dark,
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
