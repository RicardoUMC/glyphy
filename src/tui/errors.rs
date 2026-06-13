/// TUI-specific error types.
///
/// These wrap image load and processing failures for display
/// in the TUI's error state (`last_error`).
#[derive(Debug)]
pub enum TuiError {
    /// Image file could not be loaded at the given path.
    ImageLoad(String),
    /// Image processing (resize, brightness, etc.) failed.
    Processing(String),
}

impl std::fmt::Display for TuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImageLoad(msg) => write!(f, "Failed to load image: {msg}"),
            Self::Processing(msg) => write!(f, "Failed to process image: {msg}"),
        }
    }
}

impl std::error::Error for TuiError {}
