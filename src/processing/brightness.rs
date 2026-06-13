use anyhow::Result;
use image::RgbImage;

use super::glyph_buffer::GlyphBuffer;
use super::traits::Processor;
use crate::config::Config;

/// Brightness-based glyph processor.
///
/// Converts each pixel to a character based on perceived brightness.
/// Uses ITU-R BT.601 formula: 0.299R + 0.587G + 0.114B.
pub struct BrightnessProcessor;

impl Processor for BrightnessProcessor {
    fn process(&self, image: &RgbImage, config: &Config) -> Result<GlyphBuffer> {
        let (orig_w, orig_h) = image.dimensions();

        // Calculate target dimensions preserving aspect ratio.
        // Terminal characters are ~2:1 (height:width), so we double the width.
        let target_w = config.width.unwrap_or_else(|| {
            crossterm::terminal::size()
                .map(|(width, _)| u32::from(width))
                .unwrap_or(orig_w)
        });
        let target_h = config.height.unwrap_or_else(|| {
            let aspect = orig_h as f32 / orig_w as f32;
            // Factor of 0.5 compensates for non-square terminal characters
            let computed_h = (target_w as f32 * aspect * 0.5) as u32;

            if config.width.is_none() {
                if let Ok((_, height)) = crossterm::terminal::size() {
                    return computed_h.min(u32::from(height));
                }
            }

            computed_h
        });

        let target_w = target_w.max(1) as usize;
        let target_h = target_h.max(1) as usize;

        // Resize the image
        let resized = image::imageops::resize(
            image,
            target_w as u32,
            target_h as u32,
            image::imageops::FilterType::Triangle,
        );

        // Convert to glyph buffer
        let mut buffer = GlyphBuffer::new(target_w, target_h);
        for (y, row) in buffer.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let pixel = resized.get_pixel(x as u32, y as u32);
                let [r, g, b] = pixel.0;
                // Perceived brightness formula (ITU-R BT.601)
                let brightness = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
                cell.character = config.brightness_to_char(brightness);
                cell.brightness = brightness;
            }
        }

        Ok(buffer)
    }
}
