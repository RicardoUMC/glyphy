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

    /// Launch interactive TUI mode.
    #[arg(long, default_value_t = false)]
    tui: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = Config::new(cli.width, cli.height, cli.ramp, cli.invert);

    if cli.tui {
        let mut app = glyphy::tui::App::new(&cli.input, config)?;
        app.run()?;
    } else {
        render_to_terminal_with(&cli.input, &config)?;
    }

    Ok(())
}
