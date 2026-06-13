/// Default character ramp from dark to bright.
///
/// This is the classic density ramp used in ASCII art.
/// Index 0 = darkest (space), last = brightest (█).
pub const DEFAULT_RAMP: &str = " .:-=+*#%@";

/// Configuration for the glyph rendering pipeline.
#[derive(Debug, Clone)]
pub struct Config {
    /// Target width in characters. If None, auto-detect from terminal.
    pub width: Option<u32>,
    /// Target height in characters. If None, auto-detect from terminal.
    pub height: Option<u32>,
    /// Character ramp from dark to bright. Defaults to `DEFAULT_RAMP`.
    pub ramp: Vec<char>,
    /// Whether to invert brightness (bright characters for dark areas).
    pub invert: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            ramp: DEFAULT_RAMP.chars().collect(),
            invert: false,
        }
    }
}

impl Config {
    pub fn new(width: Option<u32>, height: Option<u32>, ramp: String, invert: bool) -> Self {
        let ramp: Vec<char> = ramp.chars().collect();
        let ramp = if ramp.is_empty() {
            DEFAULT_RAMP.chars().collect()
        } else {
            ramp
        };

        Self {
            width,
            height,
            ramp,
            invert,
        }
    }

    /// Map a brightness value (0.0–1.0) to a character from the ramp.
    pub fn brightness_to_char(&self, brightness: f32) -> char {
        let brightness = if self.invert {
            1.0 - brightness
        } else {
            brightness
        };

        let len = self.ramp.len();
        if len == 0 {
            return ' ';
        }

        let index = (brightness * (len - 1) as f32).round() as usize;
        let index = index.min(len - 1);
        self.ramp[index]
    }
}
