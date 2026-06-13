/// TUI module for interactive image rendering with ratatui.
///
/// This module provides the ratatui application state, keyboard handling,
/// widgets, rendering, event loop, and error types that power the `--tui`
/// interactive mode.
pub mod app;
pub mod errors;
pub mod event;
pub mod keys;
pub mod render;
pub mod widgets;

pub use app::App;
pub use errors::TuiError;
pub use keys::KeyAction;
