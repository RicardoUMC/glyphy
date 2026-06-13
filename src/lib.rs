pub mod app;
pub mod config;
pub mod input;
pub mod processing;
pub mod rendering;
pub mod tui;

use std::path::Path;

use anyhow::Result;

use crate::app::Pipeline;
use crate::input::{ImageFileLoader, ImageSource};
use crate::processing::{BrightnessProcessor, Processor};
use crate::rendering::TerminalRenderer;

pub use crate::config::Config;

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
    let source = ImageFileLoader;
    let image = source.load(path)?;
    let config = resolve_terminal_size(&config, image.dimensions());
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
    let source = ImageFileLoader;
    let image = source.load(path)?;
    let config = resolve_terminal_size(config, image.dimensions());
    let pipeline = Pipeline::new(ImageFileLoader, BrightnessProcessor, TerminalRenderer);
    pipeline.run(path, &config)
}

/// Process an image into a GlyphBuffer without rendering.
///
/// Use this when you want to consume the buffer in a custom way
/// (e.g., TUI widget or web response).
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
    let config = resolve_terminal_size(config, image.dimensions());
    processor.process(&image, &config)
}

fn resolve_terminal_size(config: &Config, (orig_w, orig_h): (u32, u32)) -> Config {
    let Ok((term_w, term_h)) = crossterm::terminal::size() else {
        return config.clone();
    };

    let mut config = config.clone();
    let width_was_auto = config.width.is_none();

    if config.width.is_none() {
        config.width = Some(u32::from(term_w));
    }

    if width_was_auto && config.height.is_none() {
        let target_w = config.width.unwrap_or(orig_w);
        let aspect = orig_h as f32 / orig_w as f32;
        let computed_h = (target_w as f32 * aspect * 0.5) as u32;
        config.height = Some(computed_h.min(u32::from(term_h)));
    }

    config
}
