use egui::epaint::{CornerRadius, Shadow};
use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke, TextStyle, Visuals};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

/// Vercel-style design tokens. Dark values follow the reference palette
/// exactly; light values are the equivalent tones for the light theme.
#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    /// Opaque overlay surface (modals, toasts). No translucency.
    pub glass: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub hover: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_tertiary: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub warning: Color32,
}

pub const DARK: Palette = Palette {
    background: Color32::from_rgb(0x00, 0x00, 0x00),
    surface: Color32::from_rgb(0x09, 0x09, 0x09),
    surface_raised: Color32::from_rgb(0x11, 0x11, 0x11),
    glass: Color32::from_rgb(0x09, 0x09, 0x09),
    border: Color32::from_rgb(0x26, 0x26, 0x26),
    border_strong: Color32::from_rgb(0x3F, 0x3F, 0x46),
    hover: Color32::from_rgb(0x18, 0x18, 0x1B),
    text_primary: Color32::from_rgb(0xFA, 0xFA, 0xFA),
    text_secondary: Color32::from_rgb(0xA1, 0xA1, 0xAA),
    text_tertiary: Color32::from_rgb(0x71, 0x71, 0x7A),
    accent: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    accent_text: Color32::from_rgb(0x00, 0x00, 0x00),
    danger: Color32::from_rgb(0xDC, 0x26, 0x26),
    success: Color32::from_rgb(0x16, 0xA3, 0x4A),
    warning: Color32::from_rgb(0xF5, 0x9E, 0x0B),
};

pub const LIGHT: Palette = Palette {
    background: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    surface_raised: Color32::from_rgb(0xFA, 0xFA, 0xFA),
    glass: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    border: Color32::from_rgb(0xEA, 0xEA, 0xEA),
    border_strong: Color32::from_rgb(0xD4, 0xD4, 0xD8),
    hover: Color32::from_rgb(0xF4, 0xF4, 0xF5),
    text_primary: Color32::from_rgb(0x0A, 0x0A, 0x0A),
    text_secondary: Color32::from_rgb(0x52, 0x52, 0x5B),
    text_tertiary: Color32::from_rgb(0xA1, 0xA1, 0xAA),
    accent: Color32::from_rgb(0x0A, 0x0A, 0x0A),
    accent_text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    danger: Color32::from_rgb(0xDC, 0x26, 0x26),
    success: Color32::from_rgb(0x16, 0xA3, 0x4A),
    warning: Color32::from_rgb(0xF5, 0x9E, 0x0B),
};

/// Corner radii of the design system: 4 for small controls, 6 for buttons
/// and inputs, 8 for cards, 12 for modals.
pub const RADIUS_SMALL: u8 = 4;
pub const RADIUS_CONTROL: u8 = 6;
pub const RADIUS_CARD: u8 = 8;
pub const RADIUS_MODAL: u8 = 12;

/// Inter weights: the helper names are kept for call-site stability, but
/// they now map to Inter 500 / 600 / 700 (never heavier than 700).
pub const FAMILY_SEMIBOLD: &str = "inter-medium";
pub const FAMILY_BOLD: &str = "inter-semibold";
pub const FAMILY_EXTRABOLD: &str = "inter-bold";
pub const FAMILY_CODE: &str = "mono-semibold";

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let faces: [(&str, &[u8]); 5] = [
        (
            "inter-regular",
            include_bytes!("../../assets/fonts/Inter-Regular.ttf"),
        ),
        (
            "inter-medium",
            include_bytes!("../../assets/fonts/Inter-Medium.ttf"),
        ),
        (
            "inter-semibold",
            include_bytes!("../../assets/fonts/Inter-SemiBold.ttf"),
        ),
        ("inter-bold", include_bytes!("../../assets/fonts/Inter-Bold.ttf")),
        (
            "mono-semibold",
            include_bytes!("../../assets/fonts/JetBrainsMono-SemiBold.ttf"),
        ),
    ];
    for (name, bytes) in faces {
        fonts
            .font_data
            .insert(name.to_owned(), std::sync::Arc::new(FontData::from_static(bytes)));
    }
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter-regular".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "mono-semibold".to_owned());
    for family in [FAMILY_SEMIBOLD, FAMILY_BOLD, FAMILY_EXTRABOLD, FAMILY_CODE] {
        fonts.families.insert(
            FontFamily::Name(family.into()),
            vec![family.to_owned(), "inter-regular".to_owned()],
        );
    }

    // Onest has no CJK glyphs; fall back to a system font so Chinese (and
    // other CJK) localizations render instead of showing boxes.
    if let Some(cjk) = load_system_cjk_font() {
        fonts
            .font_data
            .insert("system-cjk".to_owned(), std::sync::Arc::new(cjk));
        for family in fonts.families.values_mut() {
            family.push("system-cjk".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}

/// Loads a CJK-capable font shipped with the operating system.
fn load_system_cjk_font() -> Option<FontData> {
    let candidates: [(&str, u32); 4] = [
        ("C:\\Windows\\Fonts\\msyh.ttc", 0),   // Microsoft YaHei
        ("C:\\Windows\\Fonts\\msyhl.ttc", 0),  // Microsoft YaHei Light
        ("C:\\Windows\\Fonts\\simhei.ttf", 0), // SimHei
        ("C:\\Windows\\Fonts\\simsun.ttc", 0), // SimSun
    ];
    for (path, index) in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut data = FontData::from_owned(bytes);
            data.index = index;
            return Some(data);
        }
    }
    None
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FAMILY_SEMIBOLD.into()))
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FAMILY_BOLD.into()))
}

pub fn extrabold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FAMILY_EXTRABOLD.into()))
}

pub fn code_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FAMILY_CODE.into()))
}

pub fn apply(ctx: &egui::Context, dark: bool, animations: bool) {
    let palette = if dark { DARK } else { LIGHT };
    let mut visuals = if dark { Visuals::dark() } else { Visuals::light() };

    visuals.override_text_color = Some(palette.text_primary);
    visuals.panel_fill = palette.background;
    visuals.window_fill = palette.glass;
    visuals.window_stroke = Stroke::new(1.0, palette.border);
    visuals.window_corner_radius = CornerRadius::same(RADIUS_MODAL);
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    visuals.menu_corner_radius = CornerRadius::same(RADIUS_CARD);

    let base_widget = visuals.widgets.noninteractive;
    let flat = move |bg: Color32, stroke: Color32| {
        let mut widget = base_widget;
        widget.bg_fill = bg;
        widget.weak_bg_fill = bg;
        widget.bg_stroke = Stroke::new(1.0, stroke);
        widget.fg_stroke = Stroke::new(1.0, palette.text_primary);
        widget.corner_radius = CornerRadius::same(RADIUS_CONTROL);
        widget.expansion = 0.0;
        widget
    };

    visuals.widgets.noninteractive = flat(palette.surface, palette.border);
    visuals.widgets.inactive = flat(palette.surface_raised, palette.border);
    visuals.widgets.hovered = flat(palette.hover, palette.border_strong);
    visuals.widgets.active = flat(palette.hover, palette.text_primary);
    visuals.widgets.open = flat(palette.surface_raised, palette.border_strong);

    visuals.selection.bg_fill = if dark {
        Color32::from_rgb(64, 64, 64)
    } else {
        Color32::from_rgb(224, 224, 224)
    };
    visuals.selection.stroke = Stroke::new(1.0, palette.text_primary);
    visuals.hyperlink_color = palette.text_primary;
    visuals.extreme_bg_color = palette.surface_raised;
    visuals.text_cursor.stroke = Stroke::new(2.0, palette.text_primary);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(12.0, 12.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.menu_margin = Margin::same(8);
    style.spacing.window_margin = Margin::same(24);
    style.spacing.scroll = egui::style::ScrollStyle {
        bar_width: 0.0,
        bar_inner_margin: 0.0,
        bar_outer_margin: 0.0,
        handle_min_length: 0.0,
        floating: true,
        ..Default::default()
    };
    style.animation_time = if animations { 0.15 } else { 0.0 };

    style.text_styles = [
        (TextStyle::Heading, extrabold(24.0)),
        (TextStyle::Body, FontId::proportional(14.0)),
        (TextStyle::Button, semibold(14.0)),
        (TextStyle::Small, FontId::proportional(12.0)),
        (TextStyle::Monospace, code_font(14.0)),
    ]
    .into();

    ctx.set_style(style);
}

pub fn palette(dark: bool) -> Palette {
    if dark {
        DARK
    } else {
        LIGHT
    }
}

/// Primary brand colors for well-known services, keyed by a normalized
/// issuer name. Used by the optional branded avatars.
pub fn brand_color(issuer: &str) -> Option<Color32> {
    let normalized: String = issuer
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let rgb = match normalized.as_str() {
        "github" => (24, 23, 23),
        "gitlab" => (252, 109, 38),
        "youtube" => (255, 0, 0),
        "google" | "gmail" => (66, 133, 244),
        "microsoft" | "outlook" | "office365" | "azure" => (0, 120, 212),
        "apple" | "icloud" => (10, 132, 255),
        "amazon" | "aws" | "amazonwebservices" => (255, 153, 0),
        "facebook" | "meta" => (24, 119, 242),
        "instagram" => (225, 48, 108),
        "x" | "twitter" => (29, 155, 240),
        "discord" => (88, 101, 242),
        "slack" => (74, 21, 75),
        "dropbox" => (0, 97, 255),
        "steam" | "steampowered" => (27, 40, 56),
        "epicgames" => (44, 44, 44),
        "riotgames" => (235, 0, 20),
        "twitch" => (145, 70, 255),
        "reddit" => (255, 69, 0),
        "linkedin" => (10, 102, 194),
        "paypal" => (0, 48, 135),
        "stripe" => (99, 91, 255),
        "coinbase" => (0, 82, 255),
        "binance" => (240, 185, 11),
        "kraken" => (87, 41, 206),
        "protonmail" | "proton" => (109, 74, 255),
        "tutanota" | "tuta" => (132, 15, 0),
        "bitwarden" => (23, 93, 220),
        "lastpass" => (213, 43, 30),
        "1password" => (10, 132, 255),
        "nextcloud" => (0, 130, 201),
        "cloudflare" => (243, 128, 32),
        "digitalocean" => (0, 105, 255),
        "heroku" => (67, 0, 152),
        "netlify" => (0, 173, 181),
        "vercel" => (0, 0, 0),
        "npm" | "npmjs" => (203, 56, 55),
        "docker" | "dockerhub" => (36, 150, 237),
        "atlassian" | "jira" | "bitbucket" => (0, 82, 204),
        "notion" => (0, 0, 0),
        "figma" => (162, 89, 255),
        "telegram" => (36, 161, 222),
        "whatsapp" => (37, 211, 102),
        "signal" => (58, 118, 240),
        "wordpress" => (33, 117, 155),
        "shopify" => (95, 190, 66),
        "ebay" => (230, 50, 56),
        "netflix" => (229, 9, 20),
        "spotify" => (30, 215, 96),
        "nintendo" => (230, 0, 18),
        "playstation" | "sony" => (0, 67, 156),
        "xbox" => (16, 124, 16),
        "yandex" => (255, 204, 0),
        "vk" | "vkontakte" => (0, 119, 255),
        "mailru" | "mail" => (0, 95, 249),
        _ => return None,
    };
    Some(Color32::from_rgb(rgb.0, rgb.1, rgb.2))
}
