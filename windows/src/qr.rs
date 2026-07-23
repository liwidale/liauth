use egui::{Color32, ColorImage};

pub fn decode_image_bytes(bytes: &[u8]) -> Option<String> {
    let img = image::load_from_memory(bytes).ok()?.to_luma8();
    decode_luma(img)
}

pub fn decode_rgba(width: usize, height: usize, rgba: &[u8]) -> Option<String> {
    let mut luma = image::GrayImage::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let offset = (y * width + x) * 4;
            let value = (u32::from(rgba[offset]) * 299
                + u32::from(rgba[offset + 1]) * 587
                + u32::from(rgba[offset + 2]) * 114)
                / 1000;
            luma.put_pixel(x as u32, y as u32, image::Luma([value as u8]));
        }
    }
    decode_luma(luma)
}

fn decode_luma(img: image::GrayImage) -> Option<String> {
    let mut prepared = rqrr::PreparedImage::prepare(img);
    for grid in prepared.detect_grids() {
        if let Ok((_, content)) = grid.decode() {
            if !content.is_empty() {
                return Some(content);
            }
        }
    }
    None
}

pub fn render(content: &str, dark: bool) -> Option<ColorImage> {
    let code = qrcode::QrCode::new(content.as_bytes()).ok()?;
    let width = code.width();
    let quiet = 2usize;
    let size = width + quiet * 2;
    let (foreground, background) = if dark {
        (Color32::from_rgb(245, 245, 247), Color32::from_rgb(18, 18, 20))
    } else {
        (Color32::from_rgb(18, 18, 20), Color32::WHITE)
    };
    let mut pixels = vec![background; size * size];
    for y in 0..width {
        for x in 0..width {
            if code[(x, y)] == qrcode::Color::Dark {
                pixels[(y + quiet) * size + (x + quiet)] = foreground;
            }
        }
    }
    Some(ColorImage {
        size: [size, size],
        pixels,
        source_size: egui::vec2(size as f32, size as f32),
    })
}
