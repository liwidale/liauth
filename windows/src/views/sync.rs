use std::sync::mpsc;
use std::time::Duration;

use egui::{Align, Layout, RichText, Ui};
use liauth_core::time::unix_now;
use liauth_sync::{discover, send_payload, Peer, Receiver, ReceiverEvent};
use liauth_vault::{merge, BackupPayload};

use crate::app::{LiAuthApp, Overlay};
use crate::views::widgets;

pub enum SyncMode {
    Menu,
    Receiving {
        receiver: Receiver,
    },
    Discovering {
        rx: mpsc::Receiver<Result<Vec<Peer>, String>>,
    },
    PeerList {
        peers: Vec<Peer>,
    },
    EnterCode {
        peer: Peer,
        code: String,
    },
    Sending {
        rx: mpsc::Receiver<Result<(), String>>,
    },
    Done {
        message: String,
    },
    Failed {
        message: String,
    },
}

pub struct SyncState {
    pub mode: SyncMode,
}

impl Default for SyncState {
    fn default() -> Self {
        Self { mode: SyncMode::Menu }
    }
}

pub fn show(app: &mut LiAuthApp, ctx: &egui::Context, mut state: SyncState) {
    let palette = app.palette(ctx);
    let mut open = true;

    let modal = egui::Modal::new(egui::Id::new("sync-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(330.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(app.t("sync.title")).font(crate::theme::extrabold(19.0)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::quiet_button(ui, &palette, &app.t("action.close")).clicked() {
                        open = false;
                    }
                });
            });
            ui.add_space(6.0);

            state.mode = match std::mem::replace(&mut state.mode, SyncMode::Menu) {
                SyncMode::Menu => menu(app, ui),
                SyncMode::Receiving { receiver } => receiving(app, ui, receiver),
                SyncMode::Discovering { rx } => discovering(app, ui, rx),
                SyncMode::PeerList { peers } => peer_list(app, ui, peers),
                SyncMode::EnterCode { peer, code } => enter_code(app, ui, peer, code),
                SyncMode::Sending { rx } => sending(app, ui, rx),
                SyncMode::Done { message } => done(app, ui, message, &mut open),
                SyncMode::Failed { message } => failed(app, ui, message),
            };
        });

    if modal.should_close() {
        open = false;
    }
    if open {
        app.overlay = Overlay::Sync(state);
    } else if let SyncMode::Receiving { mut receiver } = state.mode {
        receiver.stop();
    }
}

fn menu(app: &mut LiAuthApp, ui: &mut Ui) -> SyncMode {
    let palette = app.palette(ui.ctx());
    ui.label(RichText::new(app.t("sync.subtitle")).color(palette.text_secondary));
    ui.add_space(16.0);

    if widgets::primary_button(ui, &palette, &app.t("sync.receive"), true).clicked() {
        match Receiver::start(&device_name()) {
            Ok(receiver) => return SyncMode::Receiving { receiver },
            Err(e) => {
                return SyncMode::Failed {
                    message: e.to_string(),
                }
            }
        }
    }
    ui.add_space(8.0);
    if widgets::secondary_button(ui, &palette, &app.t("sync.send")).clicked() {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = discover(Duration::from_secs(10)).map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
        return SyncMode::Discovering { rx };
    }
    ui.add_space(4.0);
    ui.label(
        RichText::new(app.t("sync.hint"))
            .size(12.5)
            .color(palette.text_tertiary),
    );
    SyncMode::Menu
}

fn receiving(app: &mut LiAuthApp, ui: &mut Ui, receiver: Receiver) -> SyncMode {
    let palette = app.palette(ui.ctx());
    ui.ctx().request_repaint_after(Duration::from_millis(200));

    ui.label(RichText::new(app.t("sync.receiveTitle")).color(palette.text_secondary));
    ui.add_space(18.0);
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        let code = format!("{} {}", &receiver.code[..3], &receiver.code[3..]);
        ui.label(RichText::new(code).font(crate::theme::code_font(42.0)));
    });
    ui.add_space(18.0);
    ui.label(
        RichText::new(app.t("sync.receiveHint"))
            .size(12.5)
            .color(palette.text_tertiary),
    );
    ui.add_space(10.0);

    match receiver.poll(Duration::from_millis(10)) {
        Some(ReceiverEvent::Payload(payload)) => match serde_json::from_slice::<BackupPayload>(&payload) {
            Ok(parsed) => {
                let outcome = app
                    .manager
                    .as_mut()
                    .and_then(|m| m.vault_mut().ok())
                    .map(|vault| merge(vault, parsed));
                app.save_vault();
                match outcome {
                    Some(outcome) => SyncMode::Done {
                        message: app.tf(
                            "sync.received",
                            &[
                                ("added", &outcome.added_accounts.to_string()),
                                ("skipped", &outcome.skipped.to_string()),
                            ],
                        ),
                    },
                    None => SyncMode::Failed {
                        message: app.t("error.saveFailed"),
                    },
                }
            }
            Err(_) => SyncMode::Failed {
                message: app.t("sync.failed"),
            },
        },
        Some(ReceiverEvent::Failed(_)) => SyncMode::Failed {
            message: app.t("sync.failed"),
        },
        None => SyncMode::Receiving { receiver },
    }
}

fn discovering(app: &mut LiAuthApp, ui: &mut Ui, rx: mpsc::Receiver<Result<Vec<Peer>, String>>) -> SyncMode {
    let palette = app.palette(ui.ctx());
    ui.ctx().request_repaint_after(Duration::from_millis(200));
    ui.add_space(8.0);
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.label(RichText::new(app.t("sync.searching")).color(palette.text_secondary));
    });
    ui.add_space(8.0);

    match rx.try_recv() {
        Ok(Ok(peers)) => SyncMode::PeerList { peers },
        Ok(Err(message)) => SyncMode::Failed { message },
        Err(mpsc::TryRecvError::Empty) => SyncMode::Discovering { rx },
        Err(mpsc::TryRecvError::Disconnected) => SyncMode::Failed {
            message: app.t("sync.failed"),
        },
    }
}

fn peer_list(app: &mut LiAuthApp, ui: &mut Ui, peers: Vec<Peer>) -> SyncMode {
    let palette = app.palette(ui.ctx());
    if peers.is_empty() {
        ui.add_space(8.0);
        ui.label(RichText::new(app.t("sync.noDevices")).color(palette.text_secondary));
        ui.add_space(6.0);
        ui.label(
            RichText::new(app.t("sync.noDevicesHint"))
                .size(12.5)
                .color(palette.text_tertiary),
        );
        ui.add_space(14.0);
        if widgets::secondary_button(ui, &palette, &app.t("action.back")).clicked() {
            return SyncMode::Menu;
        }
        return SyncMode::PeerList { peers };
    }

    ui.label(RichText::new(app.t("sync.selectDevice")).color(palette.text_secondary));
    ui.add_space(10.0);
    let mut selected: Option<Peer> = None;
    for peer in &peers {
        let response = widgets::card_frame(&palette)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    widgets::avatar(ui, &palette, &peer.name, 32.0, false);
                    ui.label(RichText::new(&peer.name).font(crate::theme::bold(15.0)));
                });
            })
            .response
            .interact(egui::Sense::click());
        if response.clicked() {
            selected = Some(peer.clone());
        }
        ui.add_space(2.0);
    }
    ui.add_space(8.0);
    if widgets::quiet_button(ui, &palette, &app.t("action.back")).clicked() {
        return SyncMode::Menu;
    }
    match selected {
        Some(peer) => SyncMode::EnterCode {
            peer,
            code: String::new(),
        },
        None => SyncMode::PeerList { peers },
    }
}

fn enter_code(app: &mut LiAuthApp, ui: &mut Ui, peer: Peer, mut code: String) -> SyncMode {
    let palette = app.palette(ui.ctx());
    ui.label(RichText::new(app.tf("sync.enterCode", &[("name", &peer.name)])).color(palette.text_secondary));
    ui.add_space(12.0);
    widgets::text_field(ui, &palette, &mut code, "000 000", false);
    code.retain(|c| c.is_ascii_digit());
    code.truncate(6);
    ui.add_space(14.0);

    let ready = code.len() == 6;
    let submitted = ui.input(|i| i.key_pressed(egui::Key::Enter));
    if widgets::primary_button(ui, &palette, &app.t("sync.sendNow"), ready).clicked() || (ready && submitted)
    {
        let payload = app
            .manager
            .as_ref()
            .and_then(|m| m.vault().ok())
            .map(|vault| BackupPayload {
                accounts: vault.accounts.clone(),
                categories: vault.categories.clone(),
                exported_at: unix_now(),
            });
        let Some(payload) = payload else {
            return SyncMode::Failed {
                message: app.t("error.saveFailed"),
            };
        };
        let Ok(bytes) = serde_json::to_vec(&payload) else {
            return SyncMode::Failed {
                message: app.t("sync.failed"),
            };
        };
        let addresses = peer.addresses.clone();
        let port = peer.port;
        let send_code = code.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = send_payload(&addresses, port, &send_code, &bytes).map_err(|e| match e {
                liauth_sync::SyncError::PairingFailed => String::from("pairing"),
                other => other.to_string(),
            });
            let _ = tx.send(result);
        });
        return SyncMode::Sending { rx };
    }
    ui.add_space(6.0);
    if widgets::quiet_button(ui, &palette, &app.t("action.back")).clicked() {
        return SyncMode::Menu;
    }
    SyncMode::EnterCode { peer, code }
}

fn sending(app: &mut LiAuthApp, ui: &mut Ui, rx: mpsc::Receiver<Result<(), String>>) -> SyncMode {
    let palette = app.palette(ui.ctx());
    ui.ctx().request_repaint_after(Duration::from_millis(200));
    ui.add_space(8.0);
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.label(RichText::new(app.t("sync.sending")).color(palette.text_secondary));
    });
    ui.add_space(8.0);

    match rx.try_recv() {
        Ok(Ok(())) => SyncMode::Done {
            message: app.t("sync.sent"),
        },
        Ok(Err(kind)) => {
            let message = if kind == "pairing" {
                app.t("sync.codeRejected")
            } else {
                app.t("sync.failed")
            };
            SyncMode::Failed { message }
        }
        Err(mpsc::TryRecvError::Empty) => SyncMode::Sending { rx },
        Err(mpsc::TryRecvError::Disconnected) => SyncMode::Failed {
            message: app.t("sync.failed"),
        },
    }
}

fn done(app: &mut LiAuthApp, ui: &mut Ui, message: String, open: &mut bool) -> SyncMode {
    let palette = app.palette(ui.ctx());
    ui.add_space(8.0);
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::hover());
        widgets::Icon::Check.paint(ui.painter(), rect, palette.success);
        ui.add_space(8.0);
        ui.label(RichText::new(&message).color(palette.text_secondary));
    });
    ui.add_space(14.0);
    if widgets::primary_button(ui, &palette, &app.t("action.done"), true).clicked() {
        *open = false;
    }
    SyncMode::Done { message }
}

fn failed(app: &mut LiAuthApp, ui: &mut Ui, message: String) -> SyncMode {
    let palette = app.palette(ui.ctx());
    ui.add_space(8.0);
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
        widgets::Icon::Close.paint(ui.painter(), rect, palette.danger);
        ui.add_space(8.0);
        ui.label(RichText::new(app.t("sync.failedTitle")).font(crate::theme::bold(15.0)));
        ui.add_space(4.0);
        ui.label(RichText::new(&message).size(12.5).color(palette.text_tertiary));
    });
    ui.add_space(14.0);
    if widgets::secondary_button(ui, &palette, &app.t("action.back")).clicked() {
        return SyncMode::Menu;
    }
    SyncMode::Failed { message }
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "LiAuth Desktop".to_string())
}
