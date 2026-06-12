use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use glyphy::app::Pipeline;
use glyphy::config::{Config, DEFAULT_RAMP};
use glyphy::input::ImageSource;
use glyphy::processing::{GlyphBuffer, Processor};
use glyphy::rendering::Renderer;

const MAX_DIMENSION: u32 = 2000;

/// Glyphy — Render images as glyphs in your terminal.
#[derive(Parser, Debug)]
#[command(name = "glyphy", version, about)]
struct Cli {
    /// Path to the image file (PNG, JPG, WEBP).
    #[arg(short, long)]
    input: PathBuf,

    /// Target width in characters.
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..=MAX_DIMENSION as i64))]
    width: Option<u32>,

    /// Target height in characters.
    #[arg(short = 'H', long, value_parser = clap::value_parser!(u32).range(1..=MAX_DIMENSION as i64))]
    height: Option<u32>,

    /// Character ramp from dark to bright (e.g. " .:-=+*#%@").
    #[arg(short, long, default_value = DEFAULT_RAMP)]
    ramp: String,

    /// Invert brightness mapping.
    #[arg(short = 'n', long, default_value_t = false)]
    invert: bool,
}

// --- Concrete implementations for MVP ---

/// Image loader using the `image` crate.
struct ImageFileLoader;

impl ImageSource for ImageFileLoader {
    fn load(&self, path: &std::path::Path) -> Result<image::RgbImage> {
        let img = image::open(path)
            .with_context(|| format!("Failed to open image: {}", path.display()))?;
        Ok(img.to_rgb8())
    }
}

/// Brightness-based glyph processor.
struct BrightnessProcessor;

impl Processor for BrightnessProcessor {
    fn process(&self, image: &image::RgbImage, config: &Config) -> Result<GlyphBuffer> {
        let (orig_w, orig_h) = image.dimensions();

        // Calculate target dimensions preserving aspect ratio.
        // Terminal characters are ~2:1 (height:width), so we double the width.
        let target_w = config.width.unwrap_or_else(|| {
            crossterm::terminal::size()
                .map(|(width, _)| u32::from(width))
                .unwrap_or(orig_w)
        });
        let target_h = config.height.unwrap_or_else(|| {
            let aspect = orig_h as f32 / orig_w as f32;
            // Factor of 0.5 compensates for non-square terminal characters
            let computed_h = (target_w as f32 * aspect * 0.5) as u32;

            if config.width.is_none() {
                if let Ok((_, height)) = crossterm::terminal::size() {
                    return computed_h.min(u32::from(height));
                }
            }

            computed_h
        });

        let target_w = target_w.max(1) as usize;
        let target_h = target_h.max(1) as usize;

        // Resize the image
        let resized = image::imageops::resize(
            image,
            target_w as u32,
            target_h as u32,
            image::imageops::FilterType::Triangle,
        );

        // Convert to glyph buffer
        let mut buffer = GlyphBuffer::new(target_w, target_h);
        for (y, row) in buffer.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let pixel = resized.get_pixel(x as u32, y as u32);
                let [r, g, b] = pixel.0;
                // Perceived brightness formula (ITU-R BT.601)
                let brightness = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
                cell.character = config.brightness_to_char(brightness);
                cell.brightness = brightness;
            }
        }

        Ok(buffer)
    }
}

/// Terminal stdout renderer.
struct TerminalRenderer;

impl Renderer for TerminalRenderer {
    fn render(&self, buffer: &GlyphBuffer) -> Result<()> {
        use crossterm::execute;
        use std::io::{self, Write};

        let mut stdout = io::stdout();
        // Clear screen and move cursor to top-left
        execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

        let output = buffer.to_string_output();
        print!("{output}");
        stdout.flush()?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::new(cli.width, cli.height, cli.ramp, cli.invert);

    let pipeline = Pipeline::new(ImageFileLoader, BrightnessProcessor, TerminalRenderer);
    pipeline.run(&cli.input, &config)?;

    Ok(())
}
