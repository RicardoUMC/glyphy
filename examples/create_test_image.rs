use image::{ImageBuffer, Rgb};

fn main() {
    let width = 32;
    let height = 16;

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let r = (x as f32 / (width - 1) as f32 * 255.0) as u8;
        let g = (y as f32 / (height - 1) as f32 * 255.0) as u8;
        let b = 128u8;
        Rgb([r, g, b])
    });

    img.save("test_gradient.png")
        .expect("Failed to save test image");
    println!("Created test_gradient.png ({}x{})", width, height);
}
