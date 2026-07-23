use egui::epaint::{CornerRadius, PathShape, Shadow};
use egui::{Color32, Pos2, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::{self, Palette};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Plus,
    Sync,
    Settings,
    Trash,
    Refresh,
    Close,
    Check,
    Square,
    Eye,
    EyeOff,
}

impl Icon {
    pub fn paint(self, painter: &egui::Painter, rect: Rect, color: Color32) {
        let stroke = Stroke::new(1.6, color);
        let rect = rect.shrink(rect.width() * 0.24);
        let (left, right, top, bottom) = (rect.left(), rect.right(), rect.top(), rect.bottom());
        let center = rect.center();
        match self {
            // Filled cross made of two solid bars.
            Icon::Plus => {
                let bar = rect.width() * 0.18;
                painter.rect_filled(
                    Rect::from_center_size(center, Vec2::new(bar, rect.height())),
                    CornerRadius::same(1),
                    color,
                );
                painter.rect_filled(
                    Rect::from_center_size(center, Vec2::new(rect.width(), bar)),
                    CornerRadius::same(1),
                    color,
                );
            }
            // Two solid bars with filled arrow heads.
            Icon::Sync => {
                let bar = rect.height() * 0.14;
                let head = rect.width() * 0.32;
                let y1 = rect.top() + rect.height() * 0.26;
                let y2 = rect.bottom() - rect.height() * 0.26;
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(left, y1 - bar / 2.0),
                        Pos2::new(right - head, y1 + bar / 2.0),
                    ),
                    CornerRadius::same(1),
                    color,
                );
                painter.add(PathShape::convex_polygon(
                    vec![
                        Pos2::new(right, y1),
                        Pos2::new(right - head, y1 - head * 0.7),
                        Pos2::new(right - head, y1 + head * 0.7),
                    ],
                    color,
                    Stroke::NONE,
                ));
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(left + head, y2 - bar / 2.0),
                        Pos2::new(right, y2 + bar / 2.0),
                    ),
                    CornerRadius::same(1),
                    color,
                );
                painter.add(PathShape::convex_polygon(
                    vec![
                        Pos2::new(left, y2),
                        Pos2::new(left + head, y2 - head * 0.7),
                        Pos2::new(left + head, y2 + head * 0.7),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
            // Filled slider rows with solid round knobs.
            Icon::Settings => {
                let bar = rect.height() * 0.12;
                let rows = [rect.height() * 0.18, rect.height() * 0.5, rect.height() * 0.82];
                let knobs = [0.72, 0.28, 0.55];
                for (offset, knob) in rows.iter().zip(knobs) {
                    let y = top + offset;
                    painter.rect_filled(
                        Rect::from_min_max(Pos2::new(left, y - bar / 2.0), Pos2::new(right, y + bar / 2.0)),
                        CornerRadius::same(1),
                        color,
                    );
                    let x = left + rect.width() * knob;
                    painter.circle_filled(Pos2::new(x, y), rect.width() * 0.13, color);
                }
            }
            // Filled trash can: lid, handle and tapered body.
            Icon::Trash => {
                let bar = rect.height() * 0.12;
                let lid_y = top + rect.height() * 0.18;
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(left, lid_y - bar / 2.0),
                        Pos2::new(right, lid_y + bar / 2.0),
                    ),
                    CornerRadius::same(1),
                    color,
                );
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(center.x - rect.width() * 0.18, top),
                        Pos2::new(center.x + rect.width() * 0.18, lid_y),
                    ),
                    CornerRadius::same(1),
                    color,
                );
                painter.add(PathShape::convex_polygon(
                    vec![
                        Pos2::new(left + rect.width() * 0.1, lid_y + bar),
                        Pos2::new(right - rect.width() * 0.1, lid_y + bar),
                        Pos2::new(right - rect.width() * 0.2, bottom),
                        Pos2::new(left + rect.width() * 0.2, bottom),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
            Icon::Refresh => {
                let radius = rect.width() * 0.5;
                let segments = 24;
                let start = -std::f32::consts::FRAC_PI_2;
                let sweep = std::f32::consts::TAU * 0.78;
                let points: Vec<Pos2> = (0..=segments)
                    .map(|i| {
                        let angle = start + sweep * (i as f32 / segments as f32);
                        Pos2::new(center.x + radius * angle.cos(), center.y + radius * angle.sin())
                    })
                    .collect();
                let end = points[points.len() - 1];
                painter.add(PathShape::line(points, stroke));
                let head = rect.width() * 0.24;
                painter.line_segment([end, Pos2::new(end.x + head, end.y - head * 0.2)], stroke);
                painter.line_segment([end, Pos2::new(end.x - head * 0.2, end.y - head)], stroke);
            }
            Icon::Close => {
                painter.line_segment([rect.left_top(), rect.right_bottom()], stroke);
                painter.line_segment([rect.right_top(), rect.left_bottom()], stroke);
            }
            Icon::Check => {
                let points = vec![
                    Pos2::new(left, center.y + rect.height() * 0.1),
                    Pos2::new(left + rect.width() * 0.35, bottom - rect.height() * 0.05),
                    Pos2::new(right, top + rect.height() * 0.1),
                ];
                painter.add(PathShape::line(points, stroke));
            }
            Icon::Square => {
                painter.rect_stroke(rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);
            }
            Icon::Eye | Icon::EyeOff => {
                let half_w = rect.width() * 0.5;
                let half_h = rect.height() * 0.28;
                let segments = 16;
                let mut top = Vec::with_capacity(segments + 1);
                let mut bottom = Vec::with_capacity(segments + 1);
                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let x = center.x - half_w + rect.width() * t;
                    let bow = (1.0 - (2.0 * t - 1.0).powi(2)) * half_h;
                    top.push(Pos2::new(x, center.y - bow));
                    bottom.push(Pos2::new(x, center.y + bow));
                }
                painter.add(PathShape::line(top, stroke));
                painter.add(PathShape::line(bottom, stroke));
                painter.circle_stroke(center, rect.height() * 0.16, stroke);
                if self == Icon::EyeOff {
                    painter.line_segment([rect.left_top(), rect.right_bottom()], stroke);
                }
            }
        }
    }
}

/// Elevated panel used by the lock/onboarding screens. Same construction
/// as a modal: opaque surface, hairline border, 12px radius.
pub fn glass_frame(palette: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.glass)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(theme::RADIUS_MODAL))
        .inner_margin(egui::Margin::same(24))
        .shadow(Shadow::NONE)
}

pub fn card_frame(palette: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .shadow(Shadow::NONE)
}

const BUTTON_HEIGHT: f32 = 36.0;

/// Primary action: white on black (inverts in the light theme).
pub fn primary_button(ui: &mut Ui, palette: &Palette, text: &str, enabled: bool) -> Response {
    let rich = RichText::new(text)
        .font(theme::semibold(14.0))
        .color(palette.accent_text);
    let button = egui::Button::new(rich)
        .fill(palette.accent)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(theme::RADIUS_CONTROL))
        .min_size(Vec2::new(ui.available_width().min(320.0), BUTTON_HEIGHT));
    ui.add_enabled(enabled, button)
}

/// Secondary action: raised surface with a hairline border.
pub fn secondary_button(ui: &mut Ui, palette: &Palette, text: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .font(theme::semibold(14.0))
            .color(palette.text_primary),
    )
    .fill(palette.surface_raised)
    .stroke(Stroke::new(1.0, palette.border))
    .corner_radius(CornerRadius::same(theme::RADIUS_CONTROL))
    .min_size(Vec2::new(0.0, BUTTON_HEIGHT));
    ui.add(button)
}

/// Destructive action: ghost with a red outline, never filled.
pub fn danger_button(ui: &mut Ui, palette: &Palette, text: &str, enabled: bool) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .font(theme::semibold(14.0))
            .color(palette.danger),
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::new(1.0, palette.danger))
    .corner_radius(CornerRadius::same(theme::RADIUS_CONTROL))
    .min_size(Vec2::new(0.0, BUTTON_HEIGHT));
    ui.add_enabled(enabled, button)
}

/// Ghost action: no background, no border.
pub fn quiet_button(ui: &mut Ui, palette: &Palette, text: &str) -> Response {
    let button = egui::Button::new(RichText::new(text).color(palette.text_secondary))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(theme::RADIUS_CONTROL));
    ui.add(button)
}

pub fn icon_button(ui: &mut Ui, palette: &Palette, icon: Icon, tooltip: &str) -> Response {
    let size = Vec2::splat(BUTTON_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(theme::RADIUS_CONTROL),
        if hovered {
            palette.hover
        } else {
            Color32::TRANSPARENT
        },
        Stroke::new(
            1.0,
            if hovered {
                palette.border_strong
            } else {
                palette.border
            },
        ),
        StrokeKind::Inside,
    );
    icon.paint(painter, rect, palette.text_primary);
    response.on_hover_text(tooltip)
}

pub fn text_field(
    ui: &mut Ui,
    palette: &Palette,
    value: &mut String,
    hint: &str,
    password: bool,
) -> Response {
    let width = ui.available_width();
    let reveal_id = ui.make_persistent_id(("reveal", hint));
    let mut revealed = ui.data(|d| d.get_temp::<bool>(reveal_id).unwrap_or(false));

    let output = egui::Frame::new()
        .fill(palette.surface)
        .corner_radius(CornerRadius::same(theme::RADIUS_CONTROL))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let field_width = if password { width - 62.0 } else { width - 28.0 };
                let response = ui.add(
                    egui::TextEdit::singleline(value)
                        .hint_text(RichText::new(hint).color(palette.text_tertiary))
                        .password(password && !revealed)
                        .frame(false)
                        .desired_width(field_width),
                );
                if password {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (rect, toggle) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::click());
                        let icon = if revealed { Icon::EyeOff } else { Icon::Eye };
                        icon.paint(ui.painter(), rect, palette.text_secondary);
                        if toggle.clicked() {
                            revealed = !revealed;
                        }
                    });
                }
                response
            })
            .inner
        });

    // Focus ring: a thin white border, painted after the content so the
    // state is known. No glow.
    let response = output.inner;
    let border = if response.has_focus() {
        palette.text_primary
    } else {
        palette.border
    };
    ui.painter().rect_stroke(
        output.response.rect,
        CornerRadius::same(theme::RADIUS_CONTROL),
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    if password {
        ui.data_mut(|d| d.insert_temp(reveal_id, revealed));
    }
    response
}

pub fn section_label(ui: &mut Ui, palette: &Palette, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .font(theme::semibold(12.0))
            .color(palette.text_tertiary)
            .extra_letter_spacing(0.8),
    );
}

pub fn countdown_bar(ui: &mut Ui, palette: &Palette, fraction: f32, width: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 2.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(1), palette.border);
    let filled = Rect::from_min_size(
        rect.min,
        Vec2::new(rect.width() * fraction.clamp(0.0, 1.0), rect.height()),
    );
    // The bar shifts to the warning tone when the code is about to expire.
    let fill = if fraction < 0.2 {
        palette.warning
    } else {
        palette.text_primary
    };
    painter.rect_filled(filled, CornerRadius::same(1), fill);
}

pub fn avatar(ui: &mut Ui, palette: &Palette, title: &str, size: f32, branded: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let brand = if branded { theme::brand_color(title) } else { None };

    // With a bundled Simple Icons glyph, show the real logo on the brand
    // background instead of the monogram.
    if branded {
        if let Some(slug) = crate::icons::slug_for_issuer(title) {
            if let Some(texture) = crate::icons::texture(ui.ctx(), slug) {
                let painter = ui.painter();
                let background = brand.unwrap_or(Color32::from_rgb(28, 28, 28));
                painter.rect(
                    rect,
                    CornerRadius::same(theme::RADIUS_CONTROL),
                    background,
                    Stroke::new(1.0, background),
                    StrokeKind::Inside,
                );
                let inset = rect.shrink(rect.width() * 0.2);
                painter.image(
                    texture.id(),
                    inset,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                return;
            }
        }
    }

    let painter = ui.painter();
    let (fill, stroke_color, text_color) = match brand {
        Some(color) => {
            // Keep the initial legible on any brand background.
            let luminance = 0.299 * color.r() as f32 + 0.587 * color.g() as f32 + 0.114 * color.b() as f32;
            let text = if luminance > 150.0 {
                Color32::from_rgb(0, 0, 0)
            } else {
                Color32::from_rgb(255, 255, 255)
            };
            (color, color, text)
        }
        None => (palette.surface_raised, palette.border, palette.text_primary),
    };
    painter.rect(
        rect,
        CornerRadius::same(theme::RADIUS_CONTROL),
        fill,
        Stroke::new(1.0, stroke_color),
        StrokeKind::Inside,
    );
    let initial = title
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        initial,
        theme::semibold(size * 0.4),
        text_color,
    );
}

/// Renders `text` with the characters at `indices` (char positions, as
/// produced by the fuzzy matcher) highlighted.
pub fn highlighted_label(
    ui: &mut Ui,
    palette: &Palette,
    text: &str,
    indices: &[u32],
    font: egui::FontId,
    color: Color32,
) {
    if indices.is_empty() {
        ui.label(RichText::new(text).font(font).color(color));
        return;
    }
    let matched: std::collections::HashSet<u32> = indices.iter().copied().collect();
    let mut job = egui::text::LayoutJob::default();
    let normal = egui::TextFormat {
        font_id: font.clone(),
        color,
        ..Default::default()
    };
    let highlight = egui::TextFormat {
        font_id: font,
        color: palette.accent_text,
        background: palette.accent,
        ..Default::default()
    };
    for (i, ch) in text.chars().enumerate() {
        let format = if matched.contains(&(i as u32)) {
            highlight.clone()
        } else {
            normal.clone()
        };
        job.append(&ch.to_string(), 0.0, format);
    }
    ui.label(job);
}

/// Square selection checkbox used by the batch-selection mode.
pub fn selection_box(ui: &mut Ui, palette: &Palette, selected: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(theme::RADIUS_SMALL),
        if selected { palette.accent } else { palette.surface },
        Stroke::new(
            1.0,
            if selected {
                palette.accent
            } else {
                palette.border_strong
            },
        ),
        StrokeKind::Inside,
    );
    if selected {
        Icon::Check.paint(painter, rect.shrink(3.0), palette.accent_text);
    }
    response
}

pub fn pin_marker(ui: &mut Ui, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(6.0), Sense::hover());
    ui.painter()
        .circle_filled(rect.center(), 3.0, palette.text_secondary);
}

/// Switch control: pill track, round knob, animated 150ms slide.
pub fn toggle(ui: &mut Ui, palette: &Palette, on: &mut bool) -> Response {
    let size = Vec2::new(40.0, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let progress = ui.ctx().animate_bool(response.id, *on);
    let painter = ui.painter();
    let radius = rect.height() / 2.0;
    let track = if *on {
        palette.accent
    } else {
        palette.surface_raised
    };
    painter.rect(
        rect,
        CornerRadius::same(radius as u8),
        track,
        Stroke::new(1.0, if *on { track } else { palette.border }),
        StrokeKind::Inside,
    );
    let knob_radius = radius - 4.0;
    let knob_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), progress);
    let knob_color = if *on {
        palette.accent_text
    } else {
        palette.text_secondary
    };
    painter.circle_filled(Pos2::new(knob_x, rect.center().y), knob_radius, knob_color);
    response
}

pub fn modal_frame(palette: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.glass)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(theme::RADIUS_MODAL))
        .inner_margin(egui::Margin::same(24))
        .shadow(Shadow::NONE)
}

pub fn format_code(code: &str) -> String {
    match code.len() {
        6 => format!("{} {}", &code[..3], &code[3..]),
        8 => format!("{} {}", &code[..4], &code[4..]),
        _ => code.to_string(),
    }
}
