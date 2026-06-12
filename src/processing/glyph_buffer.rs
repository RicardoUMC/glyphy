/// A single glyph cell in the output grid.
#[derive(Debug, Clone)]
pub struct GlyphCell {
    /// The character to display (e.g. ' ', '.', '#', '@').
    pub character: char,
    /// Brightness value (0.0 = black, 1.0 = white) for color rendering later.
    pub brightness: f32,
}

/// Intermediate buffer between processing and rendering.
///
/// Holds a 2D grid of `GlyphCell`s. This is the contract between
/// the processor and the renderer — processors produce it, renderers consume it.
#[derive(Debug, Clone)]
pub struct GlyphBuffer {
    /// Grid width in characters.
    pub width: usize,
    /// Grid height in characters.
    pub height: usize,
    /// Row-major grid of glyph cells.
    pub cells: Vec<Vec<GlyphCell>>,
}

impl GlyphBuffer {
    /// Create a new empty buffer with the given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![
            vec![
                GlyphCell {
                    character: ' ',
                    brightness: 0.0,
                };
                width
            ];
            height
        ];
        Self {
            width,
            height,
            cells,
        }
    }

    /// Render the buffer to a single String for terminal output.
    ///
    /// Each row is joined, and rows are separated by newlines.
    /// This approach avoids per-character print! calls which are extremely slow.
    pub fn to_string_output(&self) -> String {
        self.cells
            .iter()
            .map(|row| row.iter().map(|c| c.character).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
