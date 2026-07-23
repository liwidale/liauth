use std::time::Instant;

use egui::epaint::{CornerRadius, Shadow};
use egui::Align2;

use crate::theme::Palette;

const DURATION_SECONDS: f32 = 2.6;

pub struct Toasts {
    items: Vec<Toast>,
}

struct Toast {
    message: String,
    created: Instant,
}

impl Toasts {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, message: impl Into<String>) {
        self.items.push(Toast {
            message: message.into(),
            created: Instant::now(),
        });
    }

    pub fn show(&mut self, ctx: &egui::Context, palette: &Palette) {
        self.items
            .retain(|t| t.created.elapsed().as_secs_f32() < DURATION_SECONDS);
        if self.items.is_empty() {
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        for (index, toast) in self.items.iter().enumerate() {
            egui::Area::new(egui::Id::new(("toast", index)))
                .anchor(
                    Align2::CENTER_BOTTOM,
                    egui::vec2(0.0, -24.0 - index as f32 * 46.0),
                )
                .interactable(false)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(palette.glass)
                        .stroke(egui::Stroke::new(1.0, palette.border))
                        .corner_radius(CornerRadius::same(crate::theme::RADIUS_CARD))
                        .inner_margin(egui::Margin::symmetric(16, 10))
                        .shadow(Shadow::NONE)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&toast.message)
                                    .font(crate::theme::semibold(13.0))
                                    .color(palette.text_primary),
                            );
                        });
                });
        }
    }
}
