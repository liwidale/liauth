use std::time::Instant;

use eframe::egui;
use liauth_vault::VaultManager;
use uuid::Uuid;

use crate::i18n::Localization;
use crate::prefs::{app_dirs, AppDirs, Preferences};
use crate::security;
use crate::theme::{self, Palette, ThemeMode};
use crate::toast::Toasts;
use crate::views;

/// Vault-settings keys shared with the mobile FFI engine, so a synced vault
/// behaves the same on every platform.
pub const SETTING_AUTO_BACKUP_DIR: &str = "autoBackup.dir";
pub const SETTING_WEBDAV_URL: &str = "webdav.url";
pub const SETTING_WEBDAV_USER: &str = "webdav.username";
pub const SETTING_WEBDAV_PASSWORD: &str = "webdav.password";
pub const SETTING_WEBDAV_BACKUP_PASSWORD: &str = "webdav.backupPassword";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Onboarding,
    Locked,
    Home,
}

pub enum Overlay {
    None,
    Add(views::add::AddState),
    Edit(views::edit::EditState),
    Settings(views::settings::SettingsState),
    Sync(views::sync::SyncState),
    Categories(views::home::CategoriesState),
    EditCategory(views::home::EditCategoryState),
    Trash(views::trash::TrashState),
    ShowQr {
        account_id: Uuid,
        texture: Option<egui::TextureHandle>,
    },
    ConfirmDelete {
        account_id: Uuid,
        title: String,
    },
}

pub struct LiAuthApp {
    pub dirs: AppDirs,
    pub prefs: Preferences,
    pub loc: Localization,
    pub manager: Option<VaultManager>,
    pub screen: Screen,
    pub overlay: Overlay,
    pub toasts: Toasts,
    pub home: views::home::HomeState,
    pub lock: views::lock::LockState,
    pub last_activity: Instant,
    capture_applied: Option<bool>,
    wordmark: Option<Option<egui::TextureHandle>>,
    logo: Option<Option<egui::TextureHandle>>,
    /// Screen shown last frame, used to drive the transition fade.
    last_screen: Screen,
    screen_changed_at: Option<Instant>,
}

impl LiAuthApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let dirs = app_dirs();
        let prefs = Preferences::load(&dirs.prefs_path);
        let loc = Localization::load(Some(dirs.languages_dir.clone()), prefs.language.as_deref());

        theme::install_fonts(&cc.egui_ctx);

        let screen = if VaultManager::exists(&dirs.vault_path) {
            Screen::Locked
        } else {
            Screen::Onboarding
        };

        let last_screen = screen;
        Self {
            dirs,
            prefs,
            loc,
            manager: None,
            screen,
            overlay: Overlay::None,
            toasts: Toasts::new(),
            home: views::home::HomeState::default(),
            lock: views::lock::LockState::default(),
            last_activity: Instant::now(),
            capture_applied: None,
            wordmark: None,
            logo: None,
            last_screen,
            screen_changed_at: None,
        }
    }

    fn decode_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<egui::TextureHandle> {
        image::load_from_memory(bytes).ok().map(|img| {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let color_image =
                egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], rgba.as_raw());
            ctx.load_texture(name, color_image, egui::TextureOptions::NEAREST)
        })
    }

    pub fn wordmark_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if self.wordmark.is_none() {
            let bytes = include_bytes!("../../branding/text.png");
            self.wordmark = Some(Self::decode_texture(ctx, "wordmark", bytes));
        }
        self.wordmark.clone().flatten()
    }

    pub fn logo_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if self.logo.is_none() {
            let bytes = include_bytes!("../../branding/logo.png");
            self.logo = Some(Self::decode_texture(ctx, "logo", bytes));
        }
        self.logo.clone().flatten()
    }

    pub fn draw_logo(&mut self, ui: &mut egui::Ui, size: f32) -> bool {
        let palette = self.palette(ui.ctx());
        match self.logo_texture(ui.ctx()) {
            Some(texture) => {
                ui.add(
                    egui::Image::from_texture(&texture)
                        .fit_to_exact_size(egui::vec2(size, size))
                        .tint(palette.text_primary),
                );
                true
            }
            None => false,
        }
    }

    pub fn draw_wordmark(&mut self, ui: &mut egui::Ui, height: f32) {
        let palette = self.palette(ui.ctx());
        match self.wordmark_texture(ui.ctx()) {
            Some(texture) => {
                let aspect = texture.aspect_ratio();
                ui.add(
                    egui::Image::from_texture(&texture)
                        .fit_to_exact_size(egui::vec2(height * aspect, height))
                        .tint(palette.text_primary),
                );
            }
            None => {
                ui.label(egui::RichText::new(self.t("app.name")).font(theme::extrabold(height * 0.9)));
            }
        }
    }

    pub fn is_dark(&self, ctx: &egui::Context) -> bool {
        match self.prefs.theme_mode() {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => ctx.style().visuals.dark_mode || system_prefers_dark(ctx),
        }
    }

    pub fn palette(&self, ctx: &egui::Context) -> Palette {
        theme::palette(self.is_dark(ctx))
    }

    pub fn t(&self, key: &str) -> String {
        self.loc.t(key)
    }

    pub fn tf(&self, key: &str, args: &[(&str, &str)]) -> String {
        self.loc.tf(key, args)
    }

    pub fn save_prefs(&self) {
        self.prefs.save(&self.dirs.prefs_path);
    }

    pub fn save_vault(&mut self) {
        let message = self.t("error.saveFailed");
        if let Some(manager) = self.manager.as_mut() {
            if manager.save().is_err() {
                self.toasts.push(message);
            }
        }
        self.spawn_webdav_sync();
    }

    /// Keeps the WebDAV copy fresh after every change; sealing and uploading
    /// happen on a background thread so the UI never blocks.
    fn spawn_webdav_sync(&self) {
        let Some(vault) = self.manager.as_ref().and_then(|m| m.vault().ok()) else {
            return;
        };
        let Some(url) = vault
            .settings
            .get(SETTING_WEBDAV_URL)
            .filter(|u| !u.is_empty())
            .cloned()
        else {
            return;
        };
        let config = liauth_sync::webdav::WebDavConfig {
            url,
            username: vault
                .settings
                .get(SETTING_WEBDAV_USER)
                .cloned()
                .unwrap_or_default(),
            password: vault
                .settings
                .get(SETTING_WEBDAV_PASSWORD)
                .cloned()
                .unwrap_or_default(),
        };
        let backup_password = vault
            .settings
            .get(SETTING_WEBDAV_BACKUP_PASSWORD)
            .cloned()
            .unwrap_or_default();
        let payload = liauth_vault::BackupPayload {
            accounts: vault.active_accounts().cloned().collect(),
            categories: vault.categories.clone(),
            exported_at: liauth_core::time::unix_now(),
        };
        std::thread::spawn(move || {
            if let Ok(bytes) = liauth_vault::export_payload(&payload, &backup_password) {
                let _ = liauth_sync::webdav::upload(&config, "LiAuth-backup.liauthbackup", &bytes);
            }
        });
    }

    pub fn lock_now(&mut self) {
        if let Some(manager) = self.manager.as_mut() {
            manager.lock();
        }
        self.manager = None;
        self.overlay = Overlay::None;
        self.home = views::home::HomeState::default();
        self.lock = views::lock::LockState::default();
        self.screen = Screen::Locked;
    }

    fn handle_auto_lock(&mut self, ctx: &egui::Context) {
        let interacted = ctx.input(|i| !i.events.is_empty() || i.pointer.any_down());
        if interacted {
            self.last_activity = Instant::now();
        }
        if matches!(self.screen, Screen::Home)
            && self.prefs.auto_lock_minutes > 0
            && self.last_activity.elapsed().as_secs() >= u64::from(self.prefs.auto_lock_minutes) * 60
        {
            self.lock_now();
        }
    }

    fn handle_capture_protection(&mut self) {
        let desired = self.prefs.block_capture;
        if self.capture_applied != Some(desired) {
            security::apply_capture_protection(desired);
            self.capture_applied = Some(desired);
        }
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        if !matches!(self.screen, Screen::Home) {
            return;
        }
        let dropped: Vec<Vec<u8>> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| {
                    f.bytes
                        .as_ref()
                        .map(|b| b.to_vec())
                        .or_else(|| f.path.as_ref().and_then(|p| std::fs::read(p).ok()))
                })
                .collect()
        });
        for bytes in dropped {
            views::add::handle_incoming_bytes(self, &bytes);
        }
    }
}

impl eframe::App for LiAuthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dark = self.is_dark(ctx);
        theme::apply(ctx, dark, self.prefs.animations);
        self.handle_capture_protection();
        self.handle_auto_lock(ctx);
        self.handle_dropped_files(ctx);

        let palette = self.palette(ctx);

        // Micro-animation: fade the main screen in after a transition
        // (unlock, lock, onboarding), unless animations are disabled.
        if self.screen != self.last_screen {
            self.last_screen = self.screen;
            self.screen_changed_at = self.prefs.animations.then(Instant::now);
        }
        let fade = match self.screen_changed_at {
            Some(started) => {
                const FADE_SECONDS: f32 = 0.18;
                let progress = (started.elapsed().as_secs_f32() / FADE_SECONDS).min(1.0);
                if progress >= 1.0 {
                    self.screen_changed_at = None;
                } else {
                    ctx.request_repaint();
                }
                progress
            }
            None => 1.0,
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(palette.background))
            .show(ctx, |ui| {
                ui.multiply_opacity(fade);
                match self.screen {
                    Screen::Onboarding => views::lock::onboarding(self, ui),
                    Screen::Locked => views::lock::unlock(self, ui),
                    Screen::Home => views::home::show(self, ui),
                }
            });

        match std::mem::replace(&mut self.overlay, Overlay::None) {
            Overlay::None => {}
            Overlay::Add(state) => views::add::show(self, ctx, state),
            Overlay::Edit(state) => views::edit::show(self, ctx, state),
            Overlay::Settings(state) => views::settings::show(self, ctx, state),
            Overlay::Sync(state) => views::sync::show(self, ctx, state),
            Overlay::Categories(state) => views::home::show_categories(self, ctx, state),
            Overlay::EditCategory(state) => views::home::show_edit_category(self, ctx, state),
            Overlay::Trash(state) => views::trash::show(self, ctx, state),
            Overlay::ShowQr { account_id, texture } => views::home::show_qr(self, ctx, account_id, texture),
            Overlay::ConfirmDelete { account_id, title } => {
                views::home::show_confirm_delete(self, ctx, account_id, title)
            }
        }

        self.toasts.show(ctx, &palette);

        // Idle power use: codes only change once a second, so schedule the
        // next repaint for the next second boundary instead of a fixed fast
        // cadence, and slow down further when the window is not focused.
        if matches!(self.screen, Screen::Home) {
            let focused = ctx.input(|i| i.focused);
            let delay = if focused {
                let millis = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_millis() as u64)
                    .unwrap_or(0);
                std::time::Duration::from_millis(1010 - millis.min(1000))
            } else {
                std::time::Duration::from_secs(1)
            };
            ctx.request_repaint_after(delay);
        }
    }
}

fn system_prefers_dark(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.raw.system_theme.map(|t| t == egui::Theme::Dark).unwrap_or(true))
}
