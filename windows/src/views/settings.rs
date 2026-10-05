use std::sync::mpsc;

use egui::{Align, Layout, RichText, Ui};
use liauth_core::sntp;
use liauth_core::time::{set_time_offset, time_offset, unix_now};
use liauth_crypto::KdfParams;
use liauth_sync::webdav::{self, WebDavConfig};
use liauth_vault::export_backup;

use crate::app::{
    LiAuthApp, Overlay, SETTING_AUTO_BACKUP_DIR, SETTING_WEBDAV_BACKUP_PASSWORD, SETTING_WEBDAV_PASSWORD,
    SETTING_WEBDAV_URL, SETTING_WEBDAV_USER,
};
use crate::security;
use crate::theme::ThemeMode;
use crate::views::widgets;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PROJECT_URL: &str = "https://github.com/liwidale/liauth";
const DEVELOPER_URL: &str = "https://github.com/liwidale";

#[derive(Default)]
pub struct SettingsState {
    pub export_password: String,
    pub export_confirm: String,
    pub exporting: bool,
    pub change_current: String,
    pub change_new: String,
    pub change_confirm: String,
    pub changing_password: bool,
    pub error: Option<String>,
    pub version_clicks: u32,
    pub webdav_editing: bool,
    pub webdav_url: String,
    pub webdav_user: String,
    pub webdav_pass: String,
    pub webdav_backup_pass: String,
    pub webdav_status: Option<String>,
    pub webdav_job: Option<mpsc::Receiver<Result<(), String>>>,
    pub time_status: Option<String>,
    pub time_job: Option<mpsc::Receiver<Result<i64, String>>>,
}

pub fn show(app: &mut LiAuthApp, ctx: &egui::Context, mut state: SettingsState) {
    let palette = app.palette(ctx);
    let mut open = true;

    let modal = egui::Modal::new(egui::Id::new("settings-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(440.0);
            ui.set_max_height(600.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(app.t("settings.title")).font(crate::theme::extrabold(19.0)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::quiet_button(ui, &palette, &app.t("action.done")).clicked() {
                        open = false;
                    }
                });
            });
            ui.add_space(16.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 16.0;
                    if state.exporting {
                        export_section(app, ui, &mut state, &mut open);
                        return;
                    }
                    if state.changing_password {
                        change_password_section(app, ui, &mut state);
                        return;
                    }
                    general_section(app, ui, &mut state, &mut open);
                });
        });

    if modal.should_close() {
        open = false;
    }
    if open {
        app.overlay = Overlay::Settings(state);
    }
}

/// One row of a settings card: title and description on the left, the
/// control on the right. Consistent 12px vertical rhythm.
fn settings_row(
    ui: &mut Ui,
    palette: &crate::theme::Palette,
    title: &str,
    hint: Option<&str>,
    control: impl FnOnce(&mut Ui),
) {
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        let control_width = 170.0;
        let text_width = (ui.available_width() - control_width).max(140.0);
        ui.vertical(|ui| {
            ui.set_width(text_width);
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(
                RichText::new(title)
                    .font(crate::theme::semibold(14.0))
                    .color(palette.text_primary),
            );
            if let Some(hint) = hint {
                ui.label(RichText::new(hint).size(12.0).color(palette.text_tertiary));
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), control);
    });
    ui.add_space(12.0);
}

/// Hairline between rows of a settings card.
fn row_divider(ui: &mut Ui, palette: &crate::theme::Palette) {
    let y = ui.cursor().top();
    let rect = ui.max_rect();
    ui.painter().hline(
        rect.left()..=rect.right(),
        y,
        egui::Stroke::new(1.0, palette.border),
    );
    ui.add_space(1.0);
}

/// A titled card holding a group of settings rows.
fn section_card(ui: &mut Ui, palette: &crate::theme::Palette, title: &str, content: impl FnOnce(&mut Ui)) {
    widgets::section_label(ui, palette, title);
    ui.add_space(8.0);
    widgets::card_frame(palette).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 0.0;
        content(ui);
    });
    ui.add_space(24.0);
}

fn general_section(app: &mut LiAuthApp, ui: &mut Ui, state: &mut SettingsState, open: &mut bool) {
    let palette = app.palette(ui.ctx());
    ui.spacing_mut().item_spacing.y = 0.0;
    let mut lock_requested = false;

    // Appearance: language, theme, motion, icons.
    section_card(ui, &palette, &app.t("settings.appearance"), |ui| {
        settings_row(ui, &palette, &app.t("settings.language"), None, |ui| {
            let languages: Vec<(String, String)> = app
                .loc
                .languages()
                .map(|(code, name)| (code.to_string(), name.to_string()))
                .collect();
            let active = app.loc.active_code().to_string();
            let active_name = languages
                .iter()
                .find(|(code, _)| *code == active)
                .map(|(_, name)| name.clone())
                .unwrap_or_default();
            egui::ComboBox::from_id_salt("settings-language")
                .selected_text(active_name)
                // Tall enough for every bundled language: no scrollbar band.
                .height(360.0)
                .show_ui(ui, |ui| {
                    for (code, name) in &languages {
                        if ui.selectable_label(*code == active, name).clicked() {
                            app.loc.set_language(code);
                            app.prefs.language = Some(code.clone());
                            app.save_prefs();
                        }
                    }
                });
        });
        row_divider(ui, &palette);
        settings_row(ui, &palette, &app.t("settings.theme"), None, |ui| {
            let current = app.prefs.theme_mode();
            let options = [
                (ThemeMode::System, app.t("settings.themeSystem")),
                (ThemeMode::Light, app.t("settings.themeLight")),
                (ThemeMode::Dark, app.t("settings.themeDark")),
            ];
            let current_label = options
                .iter()
                .find(|(mode, _)| *mode == current)
                .map(|(_, label)| label.clone())
                .unwrap_or_default();
            egui::ComboBox::from_id_salt("settings-theme")
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    for (mode, label) in &options {
                        if ui.selectable_label(current == *mode, label).clicked() {
                            app.prefs.set_theme_mode(*mode);
                            app.save_prefs();
                        }
                    }
                });
        });
        row_divider(ui, &palette);
        settings_row(
            ui,
            &palette,
            &app.t("settings.animations"),
            Some(&app.t("settings.animationsHint")),
            |ui| {
                let mut value = app.prefs.animations;
                if widgets::toggle(ui, &palette, &mut value).changed() {
                    app.prefs.animations = value;
                    app.save_prefs();
                }
            },
        );
        row_divider(ui, &palette);
        settings_row(
            ui,
            &palette,
            &app.t("settings.brandIcons"),
            Some(&app.t("settings.brandIconsHint")),
            |ui| {
                let mut value = app.prefs.brand_icons;
                if widgets::toggle(ui, &palette, &mut value).changed() {
                    app.prefs.brand_icons = value;
                    app.save_prefs();
                }
            },
        );
    });

    // Protection: privacy toggles, auto lock, password.
    section_card(ui, &palette, &app.t("settings.security"), |ui| {
        settings_row(
            ui,
            &palette,
            &app.t("settings.hideCodes"),
            Some(&app.t("settings.hideCodesHint")),
            |ui| {
                let mut value = app.prefs.hide_codes;
                if widgets::toggle(ui, &palette, &mut value).changed() {
                    app.prefs.hide_codes = value;
                    app.home.revealed.clear();
                    app.save_prefs();
                }
            },
        );
        row_divider(ui, &palette);
        settings_row(
            ui,
            &palette,
            &app.t("settings.blockCapture"),
            Some(&app.t("settings.blockCaptureHint")),
            |ui| {
                let mut value = app.prefs.block_capture;
                if widgets::toggle(ui, &palette, &mut value).changed() {
                    app.prefs.block_capture = value;
                    app.save_prefs();
                }
            },
        );
        row_divider(ui, &palette);
        settings_row(
            ui,
            &palette,
            &app.t("settings.quickUnlock"),
            Some(&app.t("settings.quickUnlockHint")),
            |ui| {
                let mut value = app.prefs.quick_unlock;
                if widgets::toggle(ui, &palette, &mut value).changed() {
                    if value {
                        match security::store_quick_unlock_key() {
                            Some(key) => {
                                let added = app
                                    .manager
                                    .as_mut()
                                    .map(|m| m.add_key_slot(security::QUICK_UNLOCK_SLOT, key.bytes()).is_ok())
                                    .unwrap_or(false);
                                if added {
                                    app.prefs.quick_unlock = true;
                                } else {
                                    security::clear_quick_unlock_key();
                                }
                            }
                            None => {
                                let message = app.t("error.keychainUnavailable");
                                app.toasts.push(message);
                            }
                        }
                    } else {
                        if let Some(manager) = app.manager.as_mut() {
                            let _ = manager.remove_key_slot(security::QUICK_UNLOCK_SLOT);
                        }
                        security::clear_quick_unlock_key();
                        app.prefs.quick_unlock = false;
                    }
                    app.save_prefs();
                }
            },
        );
        row_divider(ui, &palette);
        settings_row(ui, &palette, &app.t("settings.autoLock"), None, |ui| {
            let options: [(u32, String); 5] = [
                (1, app.tf("settings.minutes", &[("count", "1")])),
                (5, app.tf("settings.minutes", &[("count", "5")])),
                (15, app.tf("settings.minutes", &[("count", "15")])),
                (30, app.tf("settings.minutes", &[("count", "30")])),
                (0, app.t("settings.never")),
            ];
            let current_label = options
                .iter()
                .find(|(v, _)| *v == app.prefs.auto_lock_minutes)
                .map(|(_, l)| l.clone())
                .unwrap_or_else(|| options[1].1.clone());
            egui::ComboBox::from_id_salt("settings-autolock")
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    for (value, label) in &options {
                        if ui
                            .selectable_label(app.prefs.auto_lock_minutes == *value, label)
                            .clicked()
                        {
                            app.prefs.auto_lock_minutes = *value;
                            app.save_prefs();
                        }
                    }
                });
        });
        row_divider(ui, &palette);
        settings_row(ui, &palette, &app.t("settings.changePassword"), None, |ui| {
            if widgets::secondary_button(ui, &palette, &app.t("action.change")).clicked() {
                state.changing_password = true;
                state.error = None;
            }
        });
        row_divider(ui, &palette);
        settings_row(ui, &palette, &app.t("settings.lockNow"), None, |ui| {
            if widgets::secondary_button(ui, &palette, &app.t("settings.lockNow")).clicked() {
                lock_requested = true;
            }
        });
    });
    if lock_requested {
        app.lock_now();
        *open = false;
        return;
    }

    // Backup: manual export, automatic copy, WebDAV.
    section_card(ui, &palette, &app.t("settings.backup"), |ui| {
        settings_row(
            ui,
            &palette,
            &app.t("settings.exportBackup"),
            Some(&app.t("settings.exportHint")),
            |ui| {
                if widgets::secondary_button(ui, &palette, &app.t("export.save")).clicked() {
                    state.exporting = true;
                    state.error = None;
                }
            },
        );
        row_divider(ui, &palette);
        auto_backup_row(app, ui);
        row_divider(ui, &palette);
        webdav_section(app, ui, state);
    });

    // Data: clock accuracy.
    section_card(ui, &palette, &app.t("settings.data"), |ui| {
        time_sync_row(app, ui, state);
    });

    // About.
    section_card(ui, &palette, &app.t("settings.about"), |ui| {
        settings_row(ui, &palette, &app.t("settings.project"), None, |ui| {
            if ui
                .link(RichText::new("liwidale/liauth").color(palette.text_primary))
                .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl::new_tab(PROJECT_URL));
            }
        });
        row_divider(ui, &palette);
        settings_row(ui, &palette, &app.t("settings.developer"), None, |ui| {
            if ui
                .link(RichText::new("liwidale").color(palette.text_primary))
                .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl::new_tab(DEVELOPER_URL));
            }
        });
        row_divider(ui, &palette);
        settings_row(ui, &palette, &app.t("settings.version"), None, |ui| {
            if ui
                .add(
                    egui::Label::new(RichText::new(VERSION).color(palette.text_secondary))
                        .sense(egui::Sense::click()),
                )
                .clicked()
            {
                state.version_clicks += 1;
                if state.version_clicks >= 5 && !app.prefs.advanced_visible {
                    app.prefs.advanced_visible = true;
                    app.save_prefs();
                    let message = app.t("settings.advancedUnlocked");
                    app.toasts.push(message);
                }
            }
        });
    });

    if app.prefs.advanced_visible {
        section_card(ui, &palette, &app.t("settings.advanced"), |ui| {
            settings_row(
                ui,
                &palette,
                &app.t("settings.advancedVisible"),
                Some(&app.t("settings.advancedHint")),
                |ui| {
                    let mut value = app.prefs.advanced_visible;
                    if widgets::toggle(ui, &palette, &mut value).changed() {
                        app.prefs.advanced_visible = value;
                        if !value {
                            state.version_clicks = 0;
                        }
                        app.save_prefs();
                    }
                },
            );
        });
    }
}

fn vault_setting(app: &LiAuthApp, key: &str) -> Option<String> {
    app.manager
        .as_ref()
        .and_then(|m| m.vault().ok())
        .and_then(|v| v.settings.get(key).cloned())
        .filter(|v| !v.is_empty())
}

fn set_vault_setting(app: &mut LiAuthApp, key: &str, value: Option<String>) {
    if let Some(manager) = app.manager.as_mut() {
        if let Ok(vault) = manager.vault_mut() {
            match value {
                Some(value) => {
                    vault.settings.insert(key.to_string(), value);
                }
                None => {
                    vault.settings.remove(key);
                }
            }
        }
    }
    app.save_vault();
}

/// Automatic encrypted copy of the vault written to a user-chosen folder.
fn auto_backup_row(app: &mut LiAuthApp, ui: &mut Ui) {
    let palette = app.palette(ui.ctx());
    let current = vault_setting(app, SETTING_AUTO_BACKUP_DIR);
    let hint = current
        .clone()
        .unwrap_or_else(|| app.t("settings.autoBackupHint"));
    let mut choose = false;
    let mut disable = false;

    settings_row(ui, &palette, &app.t("settings.autoBackup"), Some(&hint), |ui| {
        if current.is_some()
            && widgets::quiet_button(ui, &palette, &app.t("settings.autoBackupOff")).clicked()
        {
            disable = true;
            return;
        }
        if widgets::secondary_button(ui, &palette, &app.t("settings.autoBackupChoose")).clicked() {
            choose = true;
        }
    });

    if disable {
        set_vault_setting(app, SETTING_AUTO_BACKUP_DIR, None);
        if let Some(manager) = app.manager.as_mut() {
            manager.set_auto_backup_dir(None);
        }
    }
    if choose {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            let dir_string = dir.to_string_lossy().into_owned();
            set_vault_setting(app, SETTING_AUTO_BACKUP_DIR, Some(dir_string));
            if let Some(manager) = app.manager.as_mut() {
                manager.set_auto_backup_dir(Some(dir));
                let _ = manager.save();
            }
            let message = app.t("toast.saved");
            app.toasts.push(message);
        }
    }
}

fn current_webdav_config(app: &LiAuthApp) -> Option<WebDavConfig> {
    let url = vault_setting(app, SETTING_WEBDAV_URL)?;
    Some(WebDavConfig {
        url,
        username: vault_setting(app, SETTING_WEBDAV_USER).unwrap_or_default(),
        password: vault_setting(app, SETTING_WEBDAV_PASSWORD).unwrap_or_default(),
    })
}

/// Starts a background upload of a freshly exported encrypted backup.
fn spawn_webdav_upload(app: &LiAuthApp, state: &mut SettingsState) {
    let Some(config) = current_webdav_config(app) else {
        return;
    };
    let backup_password = vault_setting(app, SETTING_WEBDAV_BACKUP_PASSWORD).unwrap_or_default();
    let backup = app
        .manager
        .as_ref()
        .and_then(|m| m.vault().ok())
        .map(|v| export_backup(v, &backup_password, unix_now()));
    let Some(Ok(bytes)) = backup else {
        state.webdav_status = Some(app.t("error.saveFailed"));
        return;
    };
    let (tx, rx) = mpsc::channel();
    state.webdav_job = Some(rx);
    state.webdav_status = Some(app.t("webdav.uploading"));
    std::thread::spawn(move || {
        let result = webdav::upload(&config, "LiAuth-backup.liauthbackup", &bytes).map_err(|e| e.to_string());
        let _ = tx.send(result);
    });
}

fn webdav_section(app: &mut LiAuthApp, ui: &mut Ui, state: &mut SettingsState) {
    let palette = app.palette(ui.ctx());

    // Poll a running background job.
    if let Some(rx) = &state.webdav_job {
        match rx.try_recv() {
            Ok(Ok(())) => {
                state.webdav_job = None;
                state.webdav_status = Some(app.t("webdav.done"));
            }
            Ok(Err(message)) => {
                state.webdav_job = None;
                state.webdav_status = Some(message);
            }
            Err(mpsc::TryRecvError::Empty) => {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(200));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                state.webdav_job = None;
            }
        }
    }

    let config = current_webdav_config(app);
    let hint = state
        .webdav_status
        .clone()
        .or_else(|| config.as_ref().map(|c| c.url.clone()))
        .unwrap_or_else(|| app.t("webdav.hint"));
    let mut disable = false;
    let mut sync_now = false;

    settings_row(ui, &palette, &app.t("webdav.title"), Some(&hint), |ui| {
        if config.is_some() {
            if widgets::quiet_button(ui, &palette, &app.t("webdav.disable")).clicked() {
                disable = true;
                return;
            }
            if state.webdav_job.is_none()
                && widgets::secondary_button(ui, &palette, &app.t("webdav.syncNow")).clicked()
            {
                sync_now = true;
            }
        } else if widgets::secondary_button(ui, &palette, &app.t("webdav.configure")).clicked() {
            state.webdav_editing = true;
        }
    });

    if disable {
        for key in [
            SETTING_WEBDAV_URL,
            SETTING_WEBDAV_USER,
            SETTING_WEBDAV_PASSWORD,
            SETTING_WEBDAV_BACKUP_PASSWORD,
        ] {
            set_vault_setting(app, key, None);
        }
        state.webdav_status = None;
        state.webdav_editing = false;
        return;
    }
    if sync_now {
        spawn_webdav_upload(app, state);
    }

    if state.webdav_editing {
        ui.add_space(4.0);
        ui.spacing_mut().item_spacing.y = 8.0;
        widgets::text_field(ui, &palette, &mut state.webdav_url, &app.t("webdav.url"), false);
        widgets::text_field(
            ui,
            &palette,
            &mut state.webdav_user,
            &app.t("webdav.username"),
            false,
        );
        widgets::text_field(
            ui,
            &palette,
            &mut state.webdav_pass,
            &app.t("webdav.password"),
            true,
        );
        widgets::text_field(
            ui,
            &palette,
            &mut state.webdav_backup_pass,
            &app.t("webdav.backupPassword"),
            true,
        );
        ui.label(
            RichText::new(app.t("webdav.backupPasswordHint"))
                .size(12.0)
                .color(palette.text_tertiary),
        );
        let ready = state.webdav_url.trim().starts_with("http")
            && state.webdav_backup_pass.chars().count() >= 4
            && state.webdav_job.is_none();
        if widgets::primary_button(ui, &palette, &app.t("action.save"), ready).clicked() {
            set_vault_setting(
                app,
                SETTING_WEBDAV_URL,
                Some(state.webdav_url.trim().trim_end_matches('/').to_string()),
            );
            set_vault_setting(app, SETTING_WEBDAV_USER, Some(state.webdav_user.clone()));
            set_vault_setting(app, SETTING_WEBDAV_PASSWORD, Some(state.webdav_pass.clone()));
            set_vault_setting(
                app,
                SETTING_WEBDAV_BACKUP_PASSWORD,
                Some(state.webdav_backup_pass.clone()),
            );
            state.webdav_editing = false;
            state.webdav_pass.clear();
            state.webdav_backup_pass.clear();
            spawn_webdav_upload(app, state);
        }
        if widgets::quiet_button(ui, &palette, &app.t("action.cancel")).clicked() {
            state.webdav_editing = false;
        }
        ui.add_space(8.0);
        ui.spacing_mut().item_spacing.y = 0.0;
    }
}

/// Measures SNTP drift on a background thread and applies the offset to
/// code generation only - the OS clock is never touched.
fn time_sync_row(app: &mut LiAuthApp, ui: &mut Ui, state: &mut SettingsState) {
    let palette = app.palette(ui.ctx());

    if let Some(rx) = &state.time_job {
        match rx.try_recv() {
            Ok(Ok(offset)) => {
                state.time_job = None;
                set_time_offset(offset);
                state.time_status = Some(app.tf("timeSync.result", &[("seconds", &offset.to_string())]));
            }
            Ok(Err(message)) => {
                state.time_job = None;
                state.time_status = Some(message);
            }
            Err(mpsc::TryRecvError::Empty) => {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(200));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                state.time_job = None;
            }
        }
    }

    let status = state
        .time_status
        .clone()
        .unwrap_or_else(|| app.tf("timeSync.current", &[("seconds", &time_offset().to_string())]));

    settings_row(ui, &palette, &app.t("timeSync.title"), Some(&status), |ui| {
        if state.time_job.is_none()
            && widgets::secondary_button(ui, &palette, &app.t("timeSync.run")).clicked()
        {
            let (tx, rx) = mpsc::channel();
            state.time_job = Some(rx);
            state.time_status = Some(app.t("timeSync.running"));
            std::thread::spawn(move || {
                let result = sntp::measure_offset(&sntp::DEFAULT_SERVERS, std::time::Duration::from_secs(5))
                    .map_err(|e| e.to_string());
                let _ = tx.send(result);
            });
        }
    });
}

fn export_section(app: &mut LiAuthApp, ui: &mut Ui, state: &mut SettingsState, open: &mut bool) {
    let palette = app.palette(ui.ctx());
    ui.label(RichText::new(app.t("export.title")).font(crate::theme::bold(17.0)));
    ui.add_space(4.0);
    ui.label(
        RichText::new(app.t("export.subtitle"))
            .color(palette.text_secondary)
            .size(13.0),
    );
    ui.add_space(14.0);
    widgets::text_field(
        ui,
        &palette,
        &mut state.export_password,
        &app.t("field.password"),
        true,
    );
    ui.add_space(8.0);
    widgets::text_field(
        ui,
        &palette,
        &mut state.export_confirm,
        &app.t("field.confirmPassword"),
        true,
    );
    if let Some(error) = state.error.clone() {
        ui.add_space(8.0);
        ui.label(RichText::new(error).color(palette.text_secondary).size(13.0));
    }
    ui.add_space(14.0);
    let ready = state.export_password.chars().count() >= 4 && state.export_password == state.export_confirm;
    if widgets::primary_button(ui, &palette, &app.t("export.save"), ready).clicked() {
        let backup = app
            .manager
            .as_ref()
            .and_then(|m| m.vault().ok())
            .map(|v| export_backup(v, &state.export_password, liauth_core::time::unix_now()));
        match backup {
            Some(Ok(bytes)) => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("liauth-backup.liauth")
                    .add_filter("LiAuth Backup", &["liauth"])
                    .save_file()
                {
                    if std::fs::write(&path, &bytes).is_ok() {
                        let message = app.t("toast.backupSaved");
                        app.toasts.push(message);
                        *open = false;
                    } else {
                        state.error = Some(app.t("error.saveFailed"));
                    }
                }
            }
            _ => state.error = Some(app.t("error.saveFailed")),
        }
    }
    ui.add_space(6.0);
    if widgets::quiet_button(ui, &palette, &app.t("action.back")).clicked() {
        state.exporting = false;
        state.export_password.clear();
        state.export_confirm.clear();
        state.error = None;
    }
}

fn change_password_section(app: &mut LiAuthApp, ui: &mut Ui, state: &mut SettingsState) {
    let palette = app.palette(ui.ctx());
    ui.label(RichText::new(app.t("settings.changePassword")).font(crate::theme::bold(17.0)));
    ui.add_space(14.0);
    widgets::text_field(
        ui,
        &palette,
        &mut state.change_current,
        &app.t("field.currentPassword"),
        true,
    );
    ui.add_space(8.0);
    widgets::text_field(
        ui,
        &palette,
        &mut state.change_new,
        &app.t("field.newPassword"),
        true,
    );
    ui.add_space(8.0);
    widgets::text_field(
        ui,
        &palette,
        &mut state.change_confirm,
        &app.t("field.confirmPassword"),
        true,
    );
    if let Some(error) = state.error.clone() {
        ui.add_space(8.0);
        ui.label(RichText::new(error).color(palette.text_secondary).size(13.0));
    }
    ui.add_space(14.0);
    let ready = !state.change_current.is_empty()
        && state.change_new.chars().count() >= 4
        && state.change_new == state.change_confirm;
    if widgets::primary_button(ui, &palette, &app.t("action.save"), ready).clicked() {
        let result = app
            .manager
            .as_mut()
            .map(|m| m.change_password(&state.change_current, &state.change_new, KdfParams::default()));
        match result {
            Some(Ok(())) => {
                if app.prefs.quick_unlock {
                    security::clear_quick_unlock_key();
                    app.prefs.quick_unlock = false;
                    app.save_prefs();
                }
                let message = app.t("toast.passwordChanged");
                app.toasts.push(message);
                state.changing_password = false;
                state.change_current.clear();
                state.change_new.clear();
                state.change_confirm.clear();
                state.error = None;
            }
            _ => {
                state.error = Some(app.t("lock.wrongPassword"));
                state.change_current.clear();
            }
        }
    }
    ui.add_space(6.0);
    if widgets::quiet_button(ui, &palette, &app.t("action.back")).clicked() {
        state.changing_password = false;
        state.error = None;
    }
}
