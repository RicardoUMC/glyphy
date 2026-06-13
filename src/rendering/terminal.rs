use std::io::{self, Write};

use anyhow::Result;
use crossterm::{cursor, execute, terminal};

use super::traits::Renderer;
use crate::processing::GlyphBuffer;

/// Terminal stdout renderer.
///
/// Clears the screen and renders the glyph buffer to stdout.
/// Uses a single print! call for performance (avoiding per-character output).
pub struct TerminalRenderer;

impl Renderer for TerminalRenderer {
    fn render(&self, buffer: &GlyphBuffer) -> Result<()> {
        let mut stdout = io::stdout();
        // Clear screen and move cursor to top-left
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout, cursor::MoveTo(0, 0))?;

        let output = buffer.to_string_output();
        print!("{output}");
        stdout.flush()?;

        Ok(())
    }
}
