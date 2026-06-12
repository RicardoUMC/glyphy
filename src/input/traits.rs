use std::path::Path;

use anyhow::Result;
use image::RgbImage;

/// Trait for loading images from various sources.
///
/// Implementations handle format-specific loading (PNG, JPEG, WEBP).
/// The pipeline is agnostic to the source — it only sees an `RgbImage`.
pub trait ImageSource {
    /// Load an image from the given path and return it as an RGB image.
    fn load(&self, path: &Path) -> Result<RgbImage>;
}
