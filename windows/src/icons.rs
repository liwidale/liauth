//! Bundled Simple Icons glyphs (white 96x96 PNGs, CC0) used by the optional
//! branded avatars. Textures are decoded once and cached in the egui context.

use eframe::egui;

macro_rules! icon_set {
    ($(($slug:literal, $file:literal)),* $(,)?) => {
        fn icon_bytes(slug: &str) -> Option<&'static [u8]> {
            match slug {
                $($slug => Some(include_bytes!(concat!("../../assets/icons/", $file))),)*
                _ => None,
            }
        }
    };
}

icon_set![
    ("github", "github.png"),
    ("gitlab", "gitlab.png"),
    ("google", "google.png"),
    ("gmail", "gmail.png"),
    ("youtube", "youtube.png"),
    ("apple", "apple.png"),
    ("icloud", "icloud.png"),
    ("facebook", "facebook.png"),
    ("meta", "meta.png"),
    ("instagram", "instagram.png"),
    ("x", "x.png"),
    ("discord", "discord.png"),
    ("dropbox", "dropbox.png"),
    ("steam", "steam.png"),
    ("epicgames", "epicgames.png"),
    ("riotgames", "riotgames.png"),
    ("twitch", "twitch.png"),
    ("reddit", "reddit.png"),
    ("paypal", "paypal.png"),
    ("stripe", "stripe.png"),
    ("coinbase", "coinbase.png"),
    ("binance", "binance.png"),
    ("protonmail", "protonmail.png"),
    ("proton", "proton.png"),
    ("bitwarden", "bitwarden.png"),
    ("nextcloud", "nextcloud.png"),
    ("cloudflare", "cloudflare.png"),
    ("digitalocean", "digitalocean.png"),
    ("npm", "npm.png"),
    ("docker", "docker.png"),
    ("atlassian", "atlassian.png"),
    ("jira", "jira.png"),
    ("bitbucket", "bitbucket.png"),
    ("notion", "notion.png"),
    ("figma", "figma.png"),
    ("telegram", "telegram.png"),
    ("whatsapp", "whatsapp.png"),
    ("signal", "signal.png"),
    ("wordpress", "wordpress.png"),
    ("shopify", "shopify.png"),
    ("ebay", "ebay.png"),
    ("netflix", "netflix.png"),
    ("spotify", "spotify.png"),
    ("playstation", "playstation.png"),
    ("vk", "vk.png"),
    ("maildotru", "maildotru.png"),
    ("netlify", "netlify.png"),
    ("vercel", "vercel.png"),
    ("lastpass", "lastpass.png"),
    ("1password", "1password.png"),
];

/// Maps a normalized issuer (lowercase alphanumerics) to a bundled icon slug.
pub fn slug_for_issuer(issuer: &str) -> Option<&'static str> {
    let normalized: String = issuer
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let slug = match normalized.as_str() {
        "twitter" => "x",
        "mailru" | "mail" => "maildotru",
        other => other,
    };
    STATIC_SLUGS.iter().copied().find(|s| *s == slug)
}

const STATIC_SLUGS: [&str; 50] = [
    "github",
    "gitlab",
    "google",
    "gmail",
    "youtube",
    "apple",
    "icloud",
    "facebook",
    "meta",
    "instagram",
    "x",
    "discord",
    "dropbox",
    "steam",
    "epicgames",
    "riotgames",
    "twitch",
    "reddit",
    "paypal",
    "stripe",
    "coinbase",
    "binance",
    "protonmail",
    "proton",
    "bitwarden",
    "nextcloud",
    "cloudflare",
    "digitalocean",
    "npm",
    "docker",
    "atlassian",
    "jira",
    "bitbucket",
    "notion",
    "figma",
    "telegram",
    "whatsapp",
    "signal",
    "wordpress",
    "shopify",
    "ebay",
    "netflix",
    "spotify",
    "playstation",
    "vk",
    "maildotru",
    "netlify",
    "vercel",
    "lastpass",
    "1password",
];

/// Returns the cached texture for a brand glyph, decoding it on first use.
pub fn texture(ctx: &egui::Context, slug: &'static str) -> Option<egui::TextureHandle> {
    let id = egui::Id::new(("brand-icon", slug));
    if let Some(handle) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(handle);
    }
    let bytes = icon_bytes(slug)?;
    let image = image::load_from_memory(bytes).ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let color_image =
        egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], rgba.as_raw());
    let handle = ctx.load_texture(format!("brand-{slug}"), color_image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, handle.clone()));
    Some(handle)
}
