use egui::{RichText, Ui};
use liauth_core::time::unix_now;
use liauth_core::{Algorithm, TokenKind};
use uuid::Uuid;

use crate::app::{LiAuthApp, Overlay};
use crate::views::widgets;

pub struct EditState {
    pub account_id: Uuid,
    pub issuer: String,
    pub name: String,
    pub category_id: Option<Uuid>,
    pub algorithm: Algorithm,
    pub digits: u32,
    pub period: u32,
    pub is_totp: bool,
    pub notes: String,
    /// One recovery code per line.
    pub recovery_codes: String,
}

impl EditState {
    pub fn from_account(app: &LiAuthApp, id: Uuid) -> Option<Self> {
        let vault = app.manager.as_ref()?.vault().ok()?;
        let account = vault.account(id)?;
        let (is_totp, period) = match account.kind {
            TokenKind::Totp { period } => (true, period),
            _ => (false, 30),
        };
        Some(Self {
            account_id: id,
            issuer: account.issuer.clone(),
            name: account.name.clone(),
            category_id: account.category_id,
            algorithm: account.algorithm,
            digits: account.digits,
            period,
            is_totp,
            notes: account.notes.clone(),
            recovery_codes: account.recovery_codes.join("\n"),
        })
    }
}

pub fn show(app: &mut LiAuthApp, ctx: &egui::Context, mut state: EditState) {
    let palette = app.palette(ctx);
    let mut open = true;

    let categories: Vec<(Uuid, String)> = app
        .manager
        .as_ref()
        .and_then(|m| m.vault().ok())
        .map(|v| {
            let mut list: Vec<_> = v
                .categories
                .iter()
                .map(|c| (c.id, c.name.clone(), c.position))
                .collect();
            list.sort_by_key(|(_, _, p)| *p);
            list.into_iter().map(|(id, name, _)| (id, name)).collect()
        })
        .unwrap_or_default();

    let modal = egui::Modal::new(egui::Id::new("edit-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(330.0);
            ui.label(RichText::new(app.t("edit.title")).font(crate::theme::extrabold(19.0)));
            ui.add_space(16.0);

            widgets::text_field(ui, &palette, &mut state.issuer, &app.t("field.service"), false);
            ui.add_space(8.0);
            widgets::text_field(ui, &palette, &mut state.name, &app.t("field.account"), false);
            ui.add_space(14.0);

            widgets::section_label(ui, &palette, &app.t("edit.category"));
            ui.add_space(6.0);
            let selected_name = state
                .category_id
                .and_then(|id| categories.iter().find(|(cid, _)| *cid == id))
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| app.t("edit.noCategory"));
            egui::ComboBox::from_id_salt("edit-category")
                .selected_text(selected_name)
                .width(280.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(state.category_id.is_none(), app.t("edit.noCategory"))
                        .clicked()
                    {
                        state.category_id = None;
                    }
                    for (id, name) in &categories {
                        if ui
                            .selectable_label(state.category_id == Some(*id), name)
                            .clicked()
                        {
                            state.category_id = Some(*id);
                        }
                    }
                });

            ui.add_space(14.0);
            widgets::section_label(ui, &palette, &app.t("edit.notes"));
            ui.add_space(6.0);
            ui.add(
                egui::TextEdit::multiline(&mut state.notes)
                    .hint_text(RichText::new(app.t("edit.notesHint")).color(palette.text_tertiary))
                    .desired_rows(2)
                    .desired_width(300.0),
            );
            ui.add_space(10.0);
            widgets::section_label(ui, &palette, &app.t("edit.recoveryCodes"));
            ui.add_space(6.0);
            ui.add(
                egui::TextEdit::multiline(&mut state.recovery_codes)
                    .hint_text(RichText::new(app.t("edit.recoveryHint")).color(palette.text_tertiary))
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(3)
                    .desired_width(300.0),
            );

            if app.prefs.advanced_visible && state.is_totp {
                ui.add_space(16.0);
                widgets::section_label(ui, &palette, &app.t("settings.advanced"));
                ui.add_space(6.0);
                advanced_editor(app, ui, &mut state);
            }

            ui.add_space(20.0);
            let ready = !state.issuer.trim().is_empty() || !state.name.trim().is_empty();
            if widgets::primary_button(ui, &palette, &app.t("action.save"), ready).clicked() {
                save(app, &state);
                open = false;
            }
            ui.add_space(6.0);
            if widgets::quiet_button(ui, &palette, &app.t("action.cancel")).clicked() {
                open = false;
            }
        });

    if modal.should_close() {
        open = false;
    }
    if open {
        app.overlay = Overlay::Edit(state);
    }
}

fn advanced_editor(app: &LiAuthApp, ui: &mut Ui, state: &mut EditState) {
    let palette = app.palette(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(app.t("advanced.algorithm"))
                .color(palette.text_secondary)
                .size(13.0),
        );
        egui::ComboBox::from_id_salt("edit-algorithm")
            .selected_text(state.algorithm.name())
            .show_ui(ui, |ui| {
                for algorithm in [Algorithm::Sha1, Algorithm::Sha256, Algorithm::Sha512] {
                    if ui
                        .selectable_label(state.algorithm == algorithm, algorithm.name())
                        .clicked()
                    {
                        state.algorithm = algorithm;
                    }
                }
            });
    });
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(app.t("advanced.digits"))
                .color(palette.text_secondary)
                .size(13.0),
        );
        ui.add(egui::Slider::new(&mut state.digits, 6..=8).step_by(1.0));
    });
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(app.t("advanced.period"))
                .color(palette.text_secondary)
                .size(13.0),
        );
        ui.add(egui::Slider::new(&mut state.period, 10..=120).step_by(5.0));
    });
}

fn save(app: &mut LiAuthApp, state: &EditState) {
    if let Some(manager) = app.manager.as_mut() {
        if let Ok(vault) = manager.vault_mut() {
            if let Some(account) = vault.account_mut(state.account_id) {
                account.issuer = state.issuer.trim().to_string();
                account.name = state.name.trim().to_string();
                account.category_id = state.category_id;
                account.algorithm = state.algorithm;
                account.digits = state.digits;
                if state.is_totp {
                    account.kind = TokenKind::Totp { period: state.period };
                }
                account.notes = state.notes.trim().to_string();
                account.recovery_codes = state
                    .recovery_codes
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();
                account.updated_at = unix_now();
            }
        }
    }
    app.save_vault();
    let message = app.t("toast.saved");
    app.toasts.push(message);
}
