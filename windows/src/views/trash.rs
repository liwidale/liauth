use egui::{Align, Layout, RichText};
use liauth_core::time::unix_now;
use liauth_vault::TRASH_RETENTION_SECONDS;
use uuid::Uuid;

use crate::app::{LiAuthApp, Overlay};
use crate::views::widgets;

#[derive(Default)]
pub struct TrashState;

struct TrashRow {
    id: Uuid,
    title: String,
    subtitle: String,
    days_left: i64,
}

enum TrashAction {
    Restore(Uuid),
    Purge(Uuid),
    EmptyAll,
}

pub fn show(app: &mut LiAuthApp, ctx: &egui::Context, state: TrashState) {
    let palette = app.palette(ctx);
    let mut open = true;

    let now = unix_now();
    let rows: Vec<TrashRow> = app
        .manager
        .as_ref()
        .and_then(|m| m.vault().ok())
        .map(|v| {
            v.trashed_accounts()
                .into_iter()
                .map(|a| {
                    let deleted_at = a.deleted_at.unwrap_or(now);
                    let seconds_left = (deleted_at + TRASH_RETENTION_SECONDS - now).max(0);
                    TrashRow {
                        id: a.id,
                        title: a.display_title().to_string(),
                        subtitle: if a.issuer.is_empty() {
                            String::new()
                        } else {
                            a.name.clone()
                        },
                        days_left: (seconds_left + 86_399) / 86_400,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let mut action: Option<TrashAction> = None;

    let modal = egui::Modal::new(egui::Id::new("trash-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(380.0);
            ui.set_max_height(480.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(app.t("trash.title")).font(crate::theme::extrabold(19.0)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::quiet_button(ui, &palette, &app.t("action.done")).clicked() {
                        open = false;
                    }
                });
            });
            ui.add_space(4.0);
            ui.label(RichText::new(app.t("trash.subtitle")).color(palette.text_secondary));
            ui.add_space(14.0);

            if rows.is_empty() {
                ui.add_space(24.0);
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.label(RichText::new(app.t("trash.empty")).color(palette.text_tertiary));
                });
                ui.add_space(24.0);
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(320.0)
                .show(ui, |ui| {
                    for row in &rows {
                        widgets::card_frame(&palette).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                widgets::avatar(ui, &palette, &row.title, 34.0, app.prefs.brand_icons);
                                ui.add_space(4.0);
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 2.0;
                                    ui.label(RichText::new(&row.title).font(crate::theme::bold(14.0)));
                                    if !row.subtitle.is_empty() {
                                        ui.label(
                                            RichText::new(&row.subtitle)
                                                .size(12.0)
                                                .color(palette.text_secondary),
                                        );
                                    }
                                    ui.label(
                                        RichText::new(
                                            app.tf("trash.daysLeft", &[("days", &row.days_left.to_string())]),
                                        )
                                        .size(11.5)
                                        .color(palette.text_tertiary),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if widgets::quiet_button(ui, &palette, &app.t("trash.deleteForever"))
                                        .clicked()
                                    {
                                        action = Some(TrashAction::Purge(row.id));
                                    }
                                    if widgets::quiet_button(ui, &palette, &app.t("trash.restore")).clicked()
                                    {
                                        action = Some(TrashAction::Restore(row.id));
                                    }
                                });
                            });
                        });
                        ui.add_space(4.0);
                    }
                });

            ui.add_space(10.0);
            if widgets::danger_button(ui, &palette, &app.t("trash.emptyAll"), true).clicked() {
                action = Some(TrashAction::EmptyAll);
            }
        });

    match action {
        Some(TrashAction::Restore(id)) => {
            if let Some(manager) = app.manager.as_mut() {
                if let Ok(vault) = manager.vault_mut() {
                    vault.restore_account(id, unix_now());
                }
            }
            app.save_vault();
            let message = app.t("trash.restored");
            app.toasts.push(message);
        }
        Some(TrashAction::Purge(id)) => {
            if let Some(manager) = app.manager.as_mut() {
                if let Ok(vault) = manager.vault_mut() {
                    vault.remove_account(id);
                }
            }
            app.save_vault();
            let message = app.t("toast.accountDeleted");
            app.toasts.push(message);
        }
        Some(TrashAction::EmptyAll) => {
            if let Some(manager) = app.manager.as_mut() {
                if let Ok(vault) = manager.vault_mut() {
                    let ids: Vec<Uuid> = vault.trashed_accounts().iter().map(|a| a.id).collect();
                    for id in ids {
                        vault.remove_account(id);
                    }
                }
            }
            app.save_vault();
            let message = app.t("toast.accountDeleted");
            app.toasts.push(message);
        }
        None => {}
    }

    if modal.should_close() {
        open = false;
    }
    if open {
        app.overlay = Overlay::Trash(state);
    }
}
