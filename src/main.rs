use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use glyphy::config::{Config, DEFAULT_RAMP};
use glyphy::render_to_terminal_with;

const MAX_DIMENSION: u32 = 2000;

/// Glyphy — Render images as glyphs in your terminal.
#[derive(Parser, Debug)]
#[command(name = "glyphy", version, about)]
struct Cli {
    /// Path to the image file (PNG, JPG, WEBP). Optional in TUI mode — opens file picker if omitted.
    #[arg(short, long)]
    input: Option<PathBuf>,

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

    /// Launch interactive TUI mode.
    #[arg(long, default_value_t = false)]
    tui: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::new(cli.width, cli.height, cli.ramp, cli.invert);

    if cli.tui {
        // TUI mode: if no input, start with file picker
        if let Some(input) = &cli.input {
            let mut app = glyphy::tui::App::new(input, config)?;
            app.run()?;
        } else {
            let mut app = glyphy::tui::App::new_picker(config)?;
            app.run()?;
        }
    } else {
        // Non-TUI mode requires input
        let input = cli.input.ok_or_else(|| {
            anyhow::anyhow!("Input image is required in non-TUI mode. Use -i <image> or --tui.")
        })?;
        render_to_terminal_with(&input, &config)?;
    }

    Ok(())
}
