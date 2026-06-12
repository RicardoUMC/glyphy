use anyhow::Result;
use image::RgbImage;

use crate::config::Config;
use crate::processing::GlyphBuffer;

/// Trait for image-to-glyph processing.
///
/// Each implementation defines how pixels are mapped to characters.
/// The processor receives a raw image and config, returns a GlyphBuffer
/// ready for any renderer.
pub trait Processor {
    /// Process an RGB image into a glyph buffer.
    fn process(&self, image: &RgbImage, config: &Config) -> Result<GlyphBuffer>;
}
