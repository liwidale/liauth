use egui::RichText;
use liauth_core::time::unix_now;
use liauth_core::{base32, uri, Account, Algorithm, TokenKind};
use liauth_import::{detect_and_import, ImportError};
use liauth_vault::{import_backup, is_backup, merge, BackupPayload};

use crate::app::{LiAuthApp, Overlay};
use crate::qr;
use crate::views::widgets;

pub struct AddState {
    pub issuer: String,
    pub name: String,
    pub secret: String,
    pub error: Option<String>,
    pub pending_import: Option<PendingImport>,
    pub show_advanced: bool,
    pub algorithm: Algorithm,
    pub digits: u32,
    pub period: u32,
}

impl Default for AddState {
    fn default() -> Self {
        Self {
            issuer: String::new(),
            name: String::new(),
            secret: String::new(),
            error: None,
            pending_import: None,
            show_advanced: false,
            algorithm: Algorithm::Sha1,
            digits: 6,
            period: 30,
        }
    }
}

pub struct PendingImport {
    pub data: Vec<u8>,
    pub password: String,
    pub error: Option<String>,
}

pub fn show(app: &mut LiAuthApp, ctx: &egui::Context, mut state: AddState) {
    let palette = app.palette(ctx);
    let mut open = true;

    let modal = egui::Modal::new(egui::Id::new("add-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(340.0);

            if let Some(pending) = &mut state.pending_import {
                ui.label(RichText::new(app.t("import.passwordTitle")).font(crate::theme::bold(19.0)));
                ui.add_space(4.0);
                ui.label(RichText::new(app.t("import.passwordSubtitle")).color(palette.text_secondary));
                ui.add_space(16.0);
                widgets::text_field(
                    ui,
                    &palette,
                    &mut pending.password,
                    &app.t("field.password"),
                    true,
                );
                if let Some(error) = pending.error.clone() {
                    ui.add_space(8.0);
                    ui.label(RichText::new(error).color(palette.text_secondary).size(13.0));
                }
                ui.add_space(16.0);
                let ready = !pending.password.is_empty();
                let submitted = ui.input(|i| i.key_pressed(egui::Key::Enter));
                if widgets::primary_button(ui, &palette, &app.t("import.unlock"), ready).clicked()
                    || (ready && submitted)
                {
                    let data = pending.data.clone();
                    let password = pending.password.clone();
                    match import_bytes(app, &data, Some(&password)) {
                        ImportOutcome::Done => open = false,
                        ImportOutcome::WrongPassword => {
                            pending.error = Some(app.t("lock.wrongPassword"));
                            pending.password.clear();
                        }
                        ImportOutcome::NeedsPassword => {}
                        ImportOutcome::Failed(message) => {
                            pending.error = Some(message);
                        }
                    }
                }
                ui.add_space(6.0);
                if widgets::quiet_button(ui, &palette, &app.t("action.cancel")).clicked() {
                    state.pending_import = None;
                }
                return;
            }

            ui.label(RichText::new(app.t("add.title")).font(crate::theme::extrabold(19.0)));
            ui.add_space(4.0);
            ui.label(RichText::new(app.t("add.subtitle")).color(palette.text_secondary));
            ui.add_space(18.0);

            ui.horizontal(|ui| {
                if widgets::secondary_button(ui, &palette, &app.t("add.scanImage")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg"])
                        .pick_file()
                    {
                        match std::fs::read(&path)
                            .ok()
                            .and_then(|bytes| qr::decode_image_bytes(&bytes))
                        {
                            Some(content) => {
                                apply_scanned(app, &mut state, &content, &mut open);
                            }
                            None => state.error = Some(app.t("add.noQrFound")),
                        }
                    }
                }
                if widgets::secondary_button(ui, &palette, &app.t("add.scanClipboard")).clicked() {
                    match clipboard_qr() {
                        Some(content) => {
                            apply_scanned(app, &mut state, &content, &mut open);
                        }
                        None => state.error = Some(app.t("add.noQrFound")),
                    }
                }
            });
            ui.add_space(6.0);
            if widgets::secondary_button(ui, &palette, &app.t("add.importFile")).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(bytes) = std::fs::read(&path) {
                        match import_bytes(app, &bytes, None) {
                            ImportOutcome::Done => open = false,
                            ImportOutcome::NeedsPassword => {
                                state.pending_import = Some(PendingImport {
                                    data: bytes,
                                    password: String::new(),
                                    error: None,
                                });
                            }
                            ImportOutcome::WrongPassword | ImportOutcome::Failed(_) => {
                                state.error = Some(app.t("import.unrecognized"));
                            }
                        }
                    }
                }
            }

            ui.add_space(18.0);
            widgets::section_label(ui, &palette, &app.t("add.manualSection"));
            ui.add_space(8.0);
            widgets::text_field(ui, &palette, &mut state.issuer, &app.t("field.service"), false);
            ui.add_space(8.0);
            widgets::text_field(ui, &palette, &mut state.name, &app.t("field.account"), false);
            ui.add_space(8.0);
            widgets::text_field(ui, &palette, &mut state.secret, &app.t("field.key"), false);

            ui.add_space(8.0);
            let advanced_label = if state.show_advanced {
                app.t("add.advancedHide")
            } else {
                app.t("add.advancedShow")
            };
            if widgets::quiet_button(ui, &palette, &advanced_label).clicked() {
                state.show_advanced = !state.show_advanced;
            }
            if state.show_advanced {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(app.t("advanced.algorithm"))
                            .color(palette.text_secondary)
                            .size(13.0),
                    );
                    egui::ComboBox::from_id_salt("add-algorithm")
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
                    egui::ComboBox::from_id_salt("add-digits")
                        .selected_text(state.digits.to_string())
                        .show_ui(ui, |ui| {
                            for digits in [6u32, 7, 8] {
                                if ui
                                    .selectable_label(state.digits == digits, digits.to_string())
                                    .clicked()
                                {
                                    state.digits = digits;
                                }
                            }
                        });
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

            if let Some(error) = state.error.clone() {
                ui.add_space(10.0);
                ui.label(RichText::new(error).color(palette.text_secondary).size(13.0));
            }

            ui.add_space(18.0);
            let secret_trimmed = state.secret.trim().to_string();
            // A plausible key alone is enough - names are optional.
            let ready = secret_trimmed.starts_with("otpauth") || base32::is_plausible(&secret_trimmed);
            if widgets::primary_button(ui, &palette, &app.t("add.save"), ready).clicked() {
                match build_account(&state) {
                    Ok(mut account) => {
                        if account.issuer.trim().is_empty() && account.name.trim().is_empty() {
                            account.issuer = app.t("add.untitled");
                        }
                        insert_account(app, account);
                        open = false;
                    }
                    Err(()) => state.error = Some(app.t("add.invalidKey")),
                }
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
        app.overlay = Overlay::Add(state);
    }
}

fn apply_scanned(app: &mut LiAuthApp, state: &mut AddState, content: &str, open: &mut bool) {
    if content.starts_with("otpauth-migration://") {
        match import_bytes(app, content.as_bytes(), None) {
            ImportOutcome::Done => *open = false,
            _ => state.error = Some(app.t("import.unrecognized")),
        }
        return;
    }
    match uri::parse(content) {
        Ok(account) => {
            insert_account(app, account);
            *open = false;
        }
        Err(_) => state.error = Some(app.t("add.noQrFound")),
    }
}

fn build_account(state: &AddState) -> Result<Account, ()> {
    let secret = state.secret.trim();
    if secret.starts_with("otpauth") {
        return uri::parse(secret).map_err(|_| ());
    }
    let decoded = base32::decode(secret).map_err(|_| ())?;
    let mut account = Account::new(
        state.issuer.trim().to_string(),
        state.name.trim().to_string(),
        decoded,
        unix_now(),
    );
    if state.show_advanced {
        account.algorithm = state.algorithm;
        account.digits = state.digits.clamp(4, 10);
        account.kind = TokenKind::Totp {
            period: state.period.clamp(5, 300),
        };
    }
    Ok(account)
}

fn insert_account(app: &mut LiAuthApp, account: Account) {
    if let Some(manager) = app.manager.as_mut() {
        if let Ok(vault) = manager.vault_mut() {
            vault.accounts.push(account);
        }
    }
    app.save_vault();
    let message = app.t("toast.accountAdded");
    app.toasts.push(message);
}

fn clipboard_qr() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    if let Ok(text) = clipboard.get_text() {
        let trimmed = text.trim();
        if trimmed.starts_with("otpauth") {
            return Some(trimmed.to_string());
        }
    }
    let image = clipboard.get_image().ok()?;
    qr::decode_rgba(image.width, image.height, &image.bytes)
}

pub enum ImportOutcome {
    Done,
    NeedsPassword,
    WrongPassword,
    Failed(String),
}

pub fn import_bytes(app: &mut LiAuthApp, data: &[u8], password: Option<&str>) -> ImportOutcome {
    let payload = if is_backup(data) {
        let Some(password) = password else {
            return ImportOutcome::NeedsPassword;
        };
        match import_backup(data, password) {
            Ok(payload) => payload,
            Err(liauth_vault::VaultError::Crypto(liauth_crypto::CryptoError::Unauthenticated)) => {
                return ImportOutcome::WrongPassword
            }
            Err(e) => return ImportOutcome::Failed(e.to_string()),
        }
    } else {
        match detect_and_import(data, password) {
            Ok(result) => BackupPayload {
                accounts: result.accounts,
                categories: vec![],
                exported_at: unix_now(),
            },
            Err(ImportError::PasswordRequired) => return ImportOutcome::NeedsPassword,
            Err(ImportError::WrongPassword) => return ImportOutcome::WrongPassword,
            Err(e) => return ImportOutcome::Failed(e.to_string()),
        }
    };

    let outcome = match app.manager.as_mut().and_then(|m| m.vault_mut().ok()) {
        Some(vault) => merge(vault, payload),
        None => return ImportOutcome::Failed("locked".into()),
    };
    app.save_vault();
    let message = app.tf(
        "toast.imported",
        &[
            ("added", &outcome.added_accounts.to_string()),
            ("skipped", &outcome.skipped.to_string()),
        ],
    );
    app.toasts.push(message);
    ImportOutcome::Done
}

pub fn handle_incoming_bytes(app: &mut LiAuthApp, bytes: &[u8]) {
    if let Some(content) = qr::decode_image_bytes(bytes) {
        if let Ok(account) = uri::parse(&content) {
            insert_account(app, account);
            return;
        }
        if content.starts_with("otpauth-migration://") {
            let _ = import_bytes(app, content.as_bytes(), None);
            return;
        }
    }
    match import_bytes(app, bytes, None) {
        ImportOutcome::NeedsPassword => {
            app.overlay = Overlay::Add(AddState {
                pending_import: Some(PendingImport {
                    data: bytes.to_vec(),
                    password: String::new(),
                    error: None,
                }),
                ..Default::default()
            });
        }
        ImportOutcome::Failed(_) | ImportOutcome::WrongPassword => {
            let message = app.t("import.unrecognized");
            app.toasts.push(message);
        }
        ImportOutcome::Done => {}
    }
}
