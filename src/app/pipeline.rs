use anyhow::Result;
use image::RgbImage;

use crate::config::Config;
use crate::input::ImageSource;
use crate::processing::{GlyphBuffer, Processor};
use crate::rendering::Renderer;

/// The rendering pipeline: input → processing → output.
///
/// This is the core orchestration layer. It wires together an image source,
/// a processor, and a renderer without coupling them to each other.
pub struct Pipeline<I: ImageSource, P: Processor, R: Renderer> {
    source: I,
    processor: P,
    renderer: R,
}

impl<I: ImageSource, P: Processor, R: Renderer> Pipeline<I, P, R> {
    pub fn new(source: I, processor: P, renderer: R) -> Self {
        Self {
            source,
            processor,
            renderer,
        }
    }

    /// Run the full pipeline: load → process → render.
    pub fn run(&self, path: &std::path::Path, config: &Config) -> Result<()> {
        let image: RgbImage = self.source.load(path)?;
        let buffer: GlyphBuffer = self.processor.process(&image, config)?;
        self.renderer.render(&buffer)?;
        Ok(())
    }
}
