use anyhow::Result;

use crate::processing::GlyphBuffer;

/// Trait for rendering a GlyphBuffer to an output destination.
///
/// Renderers are output-agnostic: terminal stdout, TUI widget, file, etc.
/// The renderer receives a fully processed GlyphBuffer and displays it.
pub trait Renderer {
    /// Render the glyph buffer to the output.
    fn render(&self, buffer: &GlyphBuffer) -> Result<()>;
}
