#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unknown_lints)]
#![allow(float_literal_f32_fallback)]

mod app;
mod i18n;
mod icons;
mod prefs;
mod qr;
mod security;
mod theme;
mod toast;
mod views;

use eframe::egui;

fn main() -> eframe::Result {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([550.0, 750.0])
        .with_min_inner_size([550.0, 750.0])
        .with_title("LiAuth")
        .with_icon(load_window_icon());

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "LiAuth",
        options,
        Box::new(|cc| Ok(Box::new(app::LiAuthApp::new(cc)))),
    )
}

fn load_window_icon() -> egui::IconData {
    let bytes = include_bytes!("../../branding/logo.png");
    if bytes.is_empty() {
        return placeholder_icon();
    }
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            }
        }
        Err(_) => placeholder_icon(),
    }
}

fn placeholder_icon() -> egui::IconData {
    let size = 64usize;
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - size as f32 / 2.0;
            let dy = y as f32 - size as f32 / 2.0;
            let inside = (dx * dx + dy * dy).sqrt() < size as f32 / 2.2;
            let offset = (y * size + x) * 4;
            let shade = if inside { 20 } else { 0 };
            let alpha = if inside { 255 } else { 0 };
            rgba[offset] = shade;
            rgba[offset + 1] = shade;
            rgba[offset + 2] = shade + 2;
            rgba[offset + 3] = alpha;
        }
    }
    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}
