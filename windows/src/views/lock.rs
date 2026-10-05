use egui::{Align, Layout, RichText, Ui};
use liauth_core::time::{system_now, unix_now};
use liauth_crypto::KdfParams;
use liauth_vault::{Lockout, VaultManager};

use crate::app::{LiAuthApp, Screen};
use crate::security;
use crate::views::widgets;

pub const RESET_PHRASE: &str = "delete all data";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResetStage {
    Warn,
    Phrase,
}

#[derive(Default)]
pub struct LockState {
    pub password: String,
    pub confirm: String,
    pub error: Option<String>,
    pub busy: bool,
    pub reset_stage: Option<ResetStage>,
    pub reset_input: String,
}

pub fn onboarding(app: &mut LiAuthApp, ui: &mut Ui) {
    let palette = app.palette(ui.ctx());
    center_column(ui, |ui| {
        ui.add_space(48.0);
        app.draw_wordmark(ui, 34.0);
        ui.add_space(10.0);
        ui.label(RichText::new(app.t("onboarding.tagline")).color(palette.text_secondary));
        ui.add_space(40.0);

        widgets::glass_frame(&palette).show(ui, |ui| {
            ui.set_width(320.0);
            ui.label(RichText::new(app.t("onboarding.title")).font(crate::theme::bold(19.0)));
            ui.add_space(4.0);
            ui.label(RichText::new(app.t("onboarding.subtitle")).color(palette.text_secondary));
            ui.add_space(18.0);

            let mut password = std::mem::take(&mut app.lock.password);
            let mut confirm = std::mem::take(&mut app.lock.confirm);
            widgets::text_field(ui, &palette, &mut password, &app.t("field.password"), true);
            ui.add_space(8.0);
            widgets::text_field(ui, &palette, &mut confirm, &app.t("field.confirmPassword"), true);
            app.lock.password = password;
            app.lock.confirm = confirm;

            if let Some(error) = app.lock.error.clone() {
                ui.add_space(8.0);
                ui.label(RichText::new(error).color(palette.text_secondary).size(13.0));
            }
            ui.add_space(18.0);

            let valid = app.lock.password.chars().count() >= 4
                && app.lock.password == app.lock.confirm
                && !app.lock.busy;
            let submitted = ui.input(|i| i.key_pressed(egui::Key::Enter));
            if widgets::primary_button(ui, &palette, &app.t("onboarding.create"), valid).clicked()
                || (valid && submitted)
            {
                create_vault(app);
            }

            if !app.lock.password.is_empty() && app.lock.password.chars().count() < 4 {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(app.t("onboarding.passwordHint"))
                        .color(palette.text_tertiary)
                        .size(12.5),
                );
            } else if !app.lock.confirm.is_empty() && app.lock.password != app.lock.confirm {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(app.t("onboarding.passwordMismatch"))
                        .color(palette.text_tertiary)
                        .size(12.5),
                );
            }
        });
    });
}

pub fn unlock(app: &mut LiAuthApp, ui: &mut Ui) {
    let palette = app.palette(ui.ctx());
    center_column(ui, |ui| {
        ui.add_space(72.0);
        app.draw_wordmark(ui, 34.0);
        ui.add_space(10.0);
        ui.label(RichText::new(app.t("lock.subtitle")).color(palette.text_secondary));
        ui.add_space(40.0);

        widgets::glass_frame(&palette).show(ui, |ui| {
            ui.set_width(320.0);
            let mut password = std::mem::take(&mut app.lock.password);
            let field = widgets::text_field(ui, &palette, &mut password, &app.t("field.password"), true);
            app.lock.password = password;
            if app.lock.reset_stage.is_none() {
                field.request_focus();
            }

            // Progressive anti-brute-force delay: while it is running the
            // unlock button stays disabled and a countdown is shown.
            let lockout_wait = Lockout::for_vault(&app.dirs.vault_path).remaining_delay(system_now());
            if lockout_wait > 0 {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(app.tf("lock.tooManyAttempts", &[("seconds", &lockout_wait.to_string())]))
                        .color(palette.text_secondary)
                        .size(13.0),
                );
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(250));
            } else if let Some(error) = app.lock.error.clone() {
                ui.add_space(8.0);
                ui.label(RichText::new(error).color(palette.text_secondary).size(13.0));
            }
            ui.add_space(16.0);

            let valid = !app.lock.password.is_empty() && !app.lock.busy && lockout_wait == 0;
            let submitted = app.lock.reset_stage.is_none() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if widgets::primary_button(ui, &palette, &app.t("lock.unlock"), valid).clicked()
                || (valid && submitted)
            {
                unlock_with_password(app);
            }

            ui.add_space(10.0);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                if app.prefs.quick_unlock
                    && widgets::quiet_button(ui, &palette, &app.t("lock.quickUnlock")).clicked()
                {
                    quick_unlock(app);
                }
                if widgets::quiet_button(ui, &palette, &app.t("lock.forgot")).clicked() {
                    app.lock.reset_stage = Some(ResetStage::Warn);
                    app.lock.reset_input.clear();
                }
            });
        });
    });

    if app.lock.reset_stage.is_some() {
        reset_modal(app, ui.ctx());
    }
}

fn reset_modal(app: &mut LiAuthApp, ctx: &egui::Context) {
    let palette = app.palette(ctx);
    let Some(stage) = app.lock.reset_stage else { return };
    let mut next_stage = Some(stage);

    let modal = egui::Modal::new(egui::Id::new("reset-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(320.0);
            ui.label(RichText::new(app.t("reset.title")).font(crate::theme::extrabold(19.0)));
            ui.add_space(8.0);

            match stage {
                ResetStage::Warn => {
                    ui.label(RichText::new(app.t("reset.warning")).color(palette.text_secondary));
                    ui.add_space(18.0);
                    if widgets::danger_button(ui, &palette, &app.t("reset.yes"), true).clicked() {
                        next_stage = Some(ResetStage::Phrase);
                    }
                    ui.add_space(6.0);
                    if widgets::quiet_button(ui, &palette, &app.t("action.cancel")).clicked() {
                        next_stage = None;
                    }
                }
                ResetStage::Phrase => {
                    ui.label(
                        RichText::new(app.tf("reset.typePhrase", &[("phrase", RESET_PHRASE)]))
                            .color(palette.text_secondary),
                    );
                    ui.add_space(12.0);
                    let mut input = std::mem::take(&mut app.lock.reset_input);
                    widgets::text_field(ui, &palette, &mut input, RESET_PHRASE, false);
                    app.lock.reset_input = input;
                    ui.add_space(14.0);
                    let confirmed = app.lock.reset_input.trim().eq_ignore_ascii_case(RESET_PHRASE);
                    if widgets::danger_button(ui, &palette, &app.t("reset.confirm"), confirmed).clicked() {
                        perform_reset(app);
                        next_stage = None;
                    }
                    ui.add_space(6.0);
                    if widgets::quiet_button(ui, &palette, &app.t("action.cancel")).clicked() {
                        next_stage = None;
                    }
                }
            }
        });

    if modal.should_close() {
        next_stage = None;
    }
    app.lock.reset_stage = next_stage;
}

fn perform_reset(app: &mut LiAuthApp) {
    app.manager = None;
    let _ = std::fs::remove_file(&app.dirs.vault_path);
    let _ = std::fs::remove_file(app.dirs.vault_path.with_extension("lockout"));
    security::clear_quick_unlock_key();
    app.prefs.quick_unlock = false;
    app.save_prefs();
    app.lock = LockState::default();
    app.home = Default::default();
    app.screen = Screen::Onboarding;
    let message = app.t("reset.done");
    app.toasts.push(message);
}

fn center_column(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    ui.with_layout(Layout::top_down(Align::Center), content);
}

fn create_vault(app: &mut LiAuthApp) {
    app.lock.busy = true;
    match VaultManager::create(&app.dirs.vault_path, &app.lock.password, KdfParams::default()) {
        Ok(manager) => {
            app.manager = Some(manager);
            app.lock = LockState::default();
            app.screen = Screen::Home;
            let message = app.t("onboarding.welcome");
            app.toasts.push(message);
        }
        Err(_) => {
            app.lock.error = Some(app.t("error.createFailed"));
            app.lock.busy = false;
        }
    }
}

fn unlock_with_password(app: &mut LiAuthApp) {
    app.lock.busy = true;
    let mut lockout = Lockout::for_vault(&app.dirs.vault_path);
    if lockout.remaining_delay(system_now()) > 0 {
        app.lock.busy = false;
        return;
    }
    let result = VaultManager::open(&app.dirs.vault_path).and_then(|mut manager| {
        manager.unlock_with_password(&app.lock.password)?;
        Ok(manager)
    });
    match result {
        Ok(manager) => {
            lockout.record_success();
            finish_unlock(app, manager);
        }
        Err(_) => {
            lockout.record_failure(system_now());
            app.lock.error = Some(app.t("lock.wrongPassword"));
            app.lock.password.clear();
            app.lock.busy = false;
        }
    }
}

fn quick_unlock(app: &mut LiAuthApp) {
    let Some(key) = security::load_quick_unlock_key() else {
        app.lock.error = Some(app.t("lock.quickUnlockUnavailable"));
        return;
    };
    let result = VaultManager::open(&app.dirs.vault_path).and_then(|mut manager| {
        manager.unlock_with_slot(security::QUICK_UNLOCK_SLOT, key.bytes())?;
        Ok(manager)
    });
    match result {
        Ok(manager) => {
            Lockout::for_vault(&app.dirs.vault_path).record_success();
            finish_unlock(app, manager);
        }
        Err(_) => {
            app.lock.error = Some(app.t("lock.quickUnlockUnavailable"));
        }
    }
}

/// Shared tail of both unlock paths: configures the auto-backup target from
/// vault settings and runs the startup integrity / trash-retention pass.
fn finish_unlock(app: &mut LiAuthApp, mut manager: VaultManager) {
    let backup_dir = manager
        .vault()
        .ok()
        .and_then(|v| v.settings.get(crate::app::SETTING_AUTO_BACKUP_DIR).cloned())
        .filter(|d| !d.is_empty());
    manager.set_auto_backup_dir(backup_dir.map(std::path::PathBuf::from));

    let maintenance = manager.startup_maintenance(unix_now());
    app.manager = Some(manager);
    app.lock = LockState::default();
    app.screen = Screen::Home;

    if let Ok(report) = maintenance {
        if !report.integrity.is_clean() && report.integrity.repaired {
            let message = app.t("toast.vaultRepaired");
            app.toasts.push(message);
        }
    }
}
