pub mod app;
pub mod config;
pub mod input;
pub mod processing;
pub mod rendering;

use std::path::Path;

use anyhow::Result;

use crate::app::Pipeline;
use crate::config::Config;
use crate::input::{ImageFileLoader, ImageSource};
use crate::processing::{BrightnessProcessor, Processor};
use crate::rendering::TerminalRenderer;

/// Render an image to the terminal using default settings.
///
/// This is the simplest API — loads an image, processes it with brightness
/// mapping, and renders to stdout.
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use glyphy::render_to_terminal;
///
/// render_to_terminal(Path::new("image.png")).unwrap();
/// ```
pub fn render_to_terminal(path: &Path) -> Result<()> {
    let config = Config::default();
    let pipeline = Pipeline::new(ImageFileLoader, BrightnessProcessor, TerminalRenderer);
    pipeline.run(path, &config)
}

/// Render an image to the terminal with custom configuration.
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use glyphy::{render_to_terminal_with, Config};
///
/// let config = Config {
///     width: Some(80),
///     height: Some(40),
///     ..Default::default()
/// };
/// render_to_terminal_with(Path::new("image.png"), &config).unwrap();
/// ```
pub fn render_to_terminal_with(path: &Path, config: &Config) -> Result<()> {
    let pipeline = Pipeline::new(ImageFileLoader, BrightnessProcessor, TerminalRenderer);
    pipeline.run(path, config)
}

/// Process an image into a GlyphBuffer without rendering.
///
/// Use this when you want to consume the buffer in a custom way
/// (e.g., TUI widget, web response, Neovim plugin).
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use glyphy::{process_image, Config};
///
/// let buffer = process_image(Path::new("image.png"), &Config::default()).unwrap();
/// for row in &buffer.cells {
///     for cell in row {
///         print!("{}", cell.character);
///     }
///     println!();
/// }
/// ```
pub fn process_image(path: &Path, config: &Config) -> Result<crate::processing::GlyphBuffer> {
    let source = ImageFileLoader;
    let processor = BrightnessProcessor;
    let image = source.load(path)?;
    processor.process(&image, config)
}
