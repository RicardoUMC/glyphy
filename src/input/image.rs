use std::path::Path;

use anyhow::{Context, Result};
use image::RgbImage;

use super::traits::ImageSource;

/// Image loader using the `image` crate.
///
/// Supports PNG, JPEG, WEBP, and other formats via the `image` crate.
pub struct ImageFileLoader;

impl ImageSource for ImageFileLoader {
    fn load(&self, path: &Path) -> Result<RgbImage> {
        let img = image::open(path)
            .with_context(|| format!("Failed to open image: {}", path.display()))?;
        Ok(img.to_rgb8())
    }
}
