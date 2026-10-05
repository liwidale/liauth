use egui::{Align, Layout, RichText, Sense, Ui};
use liauth_core::otp;
use liauth_core::time::unix_now;
use liauth_core::TokenKind;
use uuid::Uuid;

use crate::app::{LiAuthApp, Overlay};
use crate::qr;
use crate::views::{add, edit, settings, sync, widgets};

#[derive(Default)]
pub struct HomeState {
    pub search: String,
    pub category: Option<Uuid>,
    pub revealed: std::collections::HashSet<Uuid>,
    /// Some(..) while the batch-selection mode is active.
    pub selection: Option<std::collections::HashSet<Uuid>>,
}

/// State of the "New group" modal, which only ever creates.
#[derive(Default)]
pub struct CategoriesState {
    pub new_name: String,
}

/// State of the "Edit group" modal, opened by right-clicking a group chip.
pub struct EditCategoryState {
    pub id: Uuid,
    pub name: String,
}

struct Row {
    id: Uuid,
    title: String,
    subtitle: String,
    code: String,
    seconds_remaining: u32,
    period: u32,
    counter_based: bool,
    pinned: bool,
    title_indices: Vec<u32>,
    subtitle_indices: Vec<u32>,
}

enum Action {
    Copy(String),
    Reveal(Uuid),
    Edit(Uuid),
    ShowQr(Uuid),
    TogglePin(Uuid),
    Delete(Uuid, String),
    Advance(Uuid),
    ToggleSelect(Uuid),
    StartSelection(Uuid),
}

pub fn show(app: &mut LiAuthApp, ui: &mut Ui) {
    let palette = app.palette(ui.ctx());

    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(20, 16))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Match the icon-button height so the wordmark and the header
                // controls sit on the same line.
                ui.set_min_height(36.0);
                app.draw_wordmark(ui, 30.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if widgets::icon_button(ui, &palette, widgets::Icon::Settings, &app.t("nav.settings"))
                        .clicked()
                    {
                        app.overlay = Overlay::Settings(settings::SettingsState::default());
                    }
                    if widgets::icon_button(ui, &palette, widgets::Icon::Trash, &app.t("trash.title"))
                        .clicked()
                    {
                        app.overlay = Overlay::Trash(crate::views::trash::TrashState);
                    }
                    if widgets::icon_button(ui, &palette, widgets::Icon::Sync, &app.t("nav.sync")).clicked() {
                        app.overlay = Overlay::Sync(sync::SyncState::default());
                    }
                    if widgets::icon_button(ui, &palette, widgets::Icon::Plus, &app.t("nav.add")).clicked() {
                        app.overlay = Overlay::Add(add::AddState::default());
                    }
                });
            });
            ui.add_space(12.0);

            let mut search = std::mem::take(&mut app.home.search);
            widgets::text_field(ui, &palette, &mut search, &app.t("home.search"), false);
            app.home.search = search;
            ui.add_space(10.0);

            category_chips(app, ui);
            ui.add_space(6.0);

            let rows = collect_rows(app);
            if rows.is_empty() {
                empty_state(app, ui);
                return;
            }

            let mut actions: Vec<Action> = Vec::new();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for row in &rows {
                        token_card(app, ui, row, &mut actions);
                        ui.add_space(4.0);
                    }
                    ui.add_space(if app.home.selection.is_some() { 72.0 } else { 24.0 });
                });

            for action in actions {
                apply_action(app, ui.ctx(), action);
            }
        });

    if app.home.selection.is_some() {
        selection_bar(app, ui.ctx());
    }
}

/// Bottom action bar shown while batch selection is active.
fn selection_bar(app: &mut LiAuthApp, ctx: &egui::Context) {
    let palette = app.palette(ctx);
    let selected_count = app.home.selection.as_ref().map(|s| s.len()).unwrap_or(0);

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

    egui::TopBottomPanel::bottom("selection-bar")
        .frame(
            egui::Frame::new()
                .fill(palette.glass)
                .stroke(egui::Stroke::new(1.0, palette.border_strong))
                .inner_margin(egui::Margin::symmetric(20, 12)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(app.tf("batch.selected", &[("count", &selected_count.to_string())]))
                        .font(crate::theme::bold(14.0)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::quiet_button(ui, &palette, &app.t("action.cancel")).clicked() {
                        app.home.selection = None;
                        return;
                    }
                    let enabled = selected_count > 0;
                    if widgets::danger_button(ui, &palette, &app.t("batch.delete"), enabled).clicked() {
                        batch_delete(app);
                        return;
                    }
                    egui::ComboBox::from_id_salt("batch-category")
                        .selected_text(app.t("batch.moveToGroup"))
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(false, app.t("edit.noCategory")).clicked() {
                                batch_move(app, None);
                            }
                            for (id, name) in &categories {
                                if ui.selectable_label(false, name).clicked() {
                                    batch_move(app, Some(*id));
                                }
                            }
                        });
                });
            });
        });
}

fn batch_delete(app: &mut LiAuthApp) {
    let Some(selected) = app.home.selection.take() else {
        return;
    };
    let now = unix_now();
    let mut trashed = 0u32;
    if let Some(manager) = app.manager.as_mut() {
        if let Ok(vault) = manager.vault_mut() {
            for id in &selected {
                if vault.trash_account(*id, now) {
                    trashed += 1;
                }
            }
        }
    }
    if trashed > 0 {
        app.save_vault();
    }
    let message = app.tf("toast.movedToTrash", &[("count", &trashed.to_string())]);
    app.toasts.push(message);
}

fn batch_move(app: &mut LiAuthApp, category_id: Option<Uuid>) {
    let Some(selected) = app.home.selection.take() else {
        return;
    };
    let now = unix_now();
    let mut moved = 0u32;
    if let Some(manager) = app.manager.as_mut() {
        if let Ok(vault) = manager.vault_mut() {
            for id in &selected {
                if let Some(account) = vault.account_mut(*id) {
                    account.category_id = category_id;
                    account.updated_at = now;
                    moved += 1;
                }
            }
        }
    }
    if moved > 0 {
        app.save_vault();
    }
    let message = app.tf("toast.movedToGroup", &[("count", &moved.to_string())]);
    app.toasts.push(message);
}

fn category_chips(app: &mut LiAuthApp, ui: &mut Ui) {
    let palette = app.palette(ui.ctx());
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

    egui::ScrollArea::horizontal()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let all_selected = app.home.category.is_none();
                if chip(ui, &palette, &app.t("home.all"), all_selected).clicked() {
                    app.home.category = None;
                }
                for (id, name) in &categories {
                    let selected = app.home.category == Some(*id);
                    // Left click filters, right click edits — so the chip keeps
                    // its primary job while staying editable in place.
                    let response =
                        chip(ui, &palette, name, selected).on_hover_text(app.t("categories.hintDesktop"));
                    if response.clicked() {
                        app.home.category = if selected { None } else { Some(*id) };
                    }
                    if response.secondary_clicked() {
                        app.overlay = Overlay::EditCategory(EditCategoryState {
                            id: *id,
                            name: name.clone(),
                        });
                    }
                }
                if chip(ui, &palette, &app.t("home.manageCategories"), false).clicked() {
                    app.overlay = Overlay::Categories(CategoriesState::default());
                }
            });
        });
}

fn chip(ui: &mut Ui, palette: &crate::theme::Palette, text: &str, selected: bool) -> egui::Response {
    let (fill, text_color, stroke) = if selected {
        (palette.accent, palette.accent_text, egui::Stroke::NONE)
    } else {
        (
            palette.surface_raised,
            palette.text_secondary,
            egui::Stroke::new(1.0, palette.border),
        )
    };
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .font(crate::theme::semibold(12.5))
                .color(text_color),
        )
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::epaint::CornerRadius::same(crate::theme::RADIUS_CONTROL)),
    )
}

fn collect_rows(app: &LiAuthApp) -> Vec<Row> {
    let Some(vault) = app.manager.as_ref().and_then(|m| m.vault().ok()) else {
        return Vec::new();
    };
    let now = unix_now();
    let query = app.home.search.trim().to_string();

    let mut scored: Vec<(i64, Row)> = vault
        .active_accounts()
        .filter(|a| {
            app.home
                .category
                .map(|c| a.category_id == Some(c))
                .unwrap_or(true)
        })
        .filter_map(|a| {
            // Typo-tolerant matching; empty queries match everything.
            let hit = liauth_core::search::match_account(a, &query)?;
            let code = otp::code_for(&a.secret.0, a.kind, a.algorithm, a.digits, now);
            let (seconds_remaining, period, counter_based) = match a.kind {
                TokenKind::Totp { period } => (otp::seconds_remaining(now, period), period, false),
                TokenKind::Steam => (otp::seconds_remaining(now, 30), 30, false),
                TokenKind::Hotp { .. } => (0, 0, true),
            };
            let title = a.display_title().to_string();
            let subtitle = if a.issuer.is_empty() {
                String::new()
            } else {
                a.name.clone()
            };
            // The match reports issuer/name positions; map them onto what the
            // card actually shows (title = issuer or name, subtitle = name).
            let (title_indices, subtitle_indices) = if a.issuer.is_empty() {
                (hit.name_indices, Vec::new())
            } else {
                (hit.issuer_indices, hit.name_indices)
            };
            Some((
                hit.score,
                Row {
                    id: a.id,
                    title,
                    subtitle,
                    code,
                    seconds_remaining,
                    period,
                    counter_based,
                    pinned: a.pinned,
                    title_indices,
                    subtitle_indices,
                },
            ))
        })
        .collect();

    if query.is_empty() {
        scored.sort_by(|(_, a), (_, b)| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
    } else {
        scored.sort_by(|(score_a, a), (score_b, b)| {
            score_b
                .cmp(score_a)
                .then_with(|| b.pinned.cmp(&a.pinned))
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
    }
    scored.into_iter().map(|(_, row)| row).collect()
}

fn mask_code(formatted: &str) -> String {
    formatted
        .chars()
        .map(|c| if c.is_whitespace() { c } else { '•' })
        .collect()
}

fn token_card(app: &LiAuthApp, ui: &mut Ui, row: &Row, actions: &mut Vec<Action>) {
    let palette = app.palette(ui.ctx());
    let hidden = app.prefs.hide_codes && !app.home.revealed.contains(&row.id);
    let selecting = app.home.selection.is_some();
    let selected = app
        .home
        .selection
        .as_ref()
        .map(|s| s.contains(&row.id))
        .unwrap_or(false);
    let response = widgets::card_frame(&palette)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if selecting && widgets::selection_box(ui, &palette, selected).clicked() {
                    actions.push(Action::ToggleSelect(row.id));
                }
                widgets::avatar(ui, &palette, &row.title, 40.0, app.prefs.brand_icons);
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.horizontal(|ui| {
                        widgets::highlighted_label(
                            ui,
                            &palette,
                            &row.title,
                            &row.title_indices,
                            crate::theme::bold(15.0),
                            palette.text_primary,
                        );
                        if row.pinned {
                            widgets::pin_marker(ui, &palette);
                        }
                    });
                    if !row.subtitle.is_empty() {
                        widgets::highlighted_label(
                            ui,
                            &palette,
                            &row.subtitle,
                            &row.subtitle_indices,
                            egui::FontId::proportional(12.5),
                            palette.text_secondary,
                        );
                    }
                    let formatted = widgets::format_code(&row.code);
                    let display = if hidden { mask_code(&formatted) } else { formatted };
                    ui.label(
                        RichText::new(display)
                            .font(crate::theme::code_font(26.0))
                            .color(palette.text_primary),
                    );
                    if !row.counter_based {
                        ui.add_space(4.0);
                        let fraction = row.seconds_remaining as f32 / row.period.max(1) as f32;
                        widgets::countdown_bar(ui, &palette, fraction, 180.0);
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if row.counter_based {
                        if widgets::icon_button(ui, &palette, widgets::Icon::Refresh, &app.t("home.nextCode"))
                            .clicked()
                        {
                            actions.push(Action::Advance(row.id));
                        }
                    } else {
                        ui.label(
                            RichText::new(format!("{:02}", row.seconds_remaining))
                                .font(crate::theme::code_font(13.0))
                                .color(palette.text_tertiary),
                        )
                        .on_hover_text(app.tf(
                            "home.expiresIn",
                            &[("seconds", &row.seconds_remaining.to_string())],
                        ));
                    }
                });
            });
        })
        .response;

    let response = response.interact(Sense::click());
    if response.clicked() {
        if selecting {
            actions.push(Action::ToggleSelect(row.id));
        } else if hidden {
            actions.push(Action::Reveal(row.id));
        } else {
            actions.push(Action::Copy(row.code.clone()));
        }
    }
    if selecting {
        return;
    }
    response.context_menu(|ui| {
        ui.set_min_width(190.0);
        if ui.button(app.t("menu.copyCode")).clicked() {
            actions.push(Action::Copy(row.code.clone()));
            ui.close();
        }
        if ui.button(app.t("menu.select")).clicked() {
            actions.push(Action::StartSelection(row.id));
            ui.close();
        }
        if ui.button(app.t("menu.edit")).clicked() {
            actions.push(Action::Edit(row.id));
            ui.close();
        }
        if ui.button(app.t("menu.showQr")).clicked() {
            actions.push(Action::ShowQr(row.id));
            ui.close();
        }
        let pin_label = if row.pinned {
            app.t("menu.unpin")
        } else {
            app.t("menu.pin")
        };
        if ui.button(pin_label).clicked() {
            actions.push(Action::TogglePin(row.id));
            ui.close();
        }
        ui.separator();
        if ui
            .button(RichText::new(app.t("menu.delete")).color(palette.text_secondary))
            .clicked()
        {
            actions.push(Action::Delete(row.id, row.title.clone()));
            ui.close();
        }
    });
}

fn empty_state(app: &mut LiAuthApp, ui: &mut Ui) {
    let palette = app.palette(ui.ctx());
    ui.add_space(64.0);
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        if !app.draw_logo(ui, 64.0) {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(56.0, 56.0), Sense::hover());
            widgets::Icon::Square.paint(ui.painter(), rect, palette.text_tertiary);
        }
        ui.add_space(12.0);
        let (title, subtitle) = if app.home.search.trim().is_empty() && app.home.category.is_none() {
            (app.t("home.emptyTitle"), app.t("home.emptySubtitle"))
        } else {
            (app.t("home.noResultsTitle"), app.t("home.noResultsSubtitle"))
        };
        ui.label(RichText::new(title).font(crate::theme::bold(18.0)));
        ui.add_space(4.0);
        ui.label(RichText::new(subtitle).color(palette.text_secondary));
    });
}

fn apply_action(app: &mut LiAuthApp, ctx: &egui::Context, action: Action) {
    match action {
        Action::Copy(code) => {
            ctx.copy_text(code);
            let message = app.t("toast.codeCopied");
            app.toasts.push(message);
        }
        Action::Reveal(id) => {
            app.home.revealed.insert(id);
        }
        Action::Edit(id) => {
            if let Some(state) = edit::EditState::from_account(app, id) {
                app.overlay = Overlay::Edit(state);
            }
        }
        Action::ShowQr(id) => {
            app.overlay = Overlay::ShowQr {
                account_id: id,
                texture: None,
            };
        }
        Action::TogglePin(id) => {
            if let Some(manager) = app.manager.as_mut() {
                if let Ok(vault) = manager.vault_mut() {
                    if let Some(account) = vault.account_mut(id) {
                        account.pinned = !account.pinned;
                        account.updated_at = unix_now();
                    }
                }
            }
            app.save_vault();
        }
        Action::Delete(id, title) => {
            app.overlay = Overlay::ConfirmDelete {
                account_id: id,
                title,
            };
        }
        Action::Advance(id) => {
            if let Some(manager) = app.manager.as_mut() {
                if let Ok(vault) = manager.vault_mut() {
                    if let Some(account) = vault.account_mut(id) {
                        if let TokenKind::Hotp { counter } = account.kind {
                            account.kind = TokenKind::Hotp { counter: counter + 1 };
                            account.updated_at = unix_now();
                        }
                    }
                }
            }
            app.save_vault();
        }
        Action::ToggleSelect(id) => {
            if let Some(selection) = app.home.selection.as_mut() {
                if !selection.remove(&id) {
                    selection.insert(id);
                }
            }
        }
        Action::StartSelection(id) => {
            let mut selection = std::collections::HashSet::new();
            selection.insert(id);
            app.home.selection = Some(selection);
        }
    }
}

/// Creates a group and nothing else; existing groups are edited by
/// right-clicking their chip on the home screen.
pub fn show_categories(app: &mut LiAuthApp, ctx: &egui::Context, mut state: CategoriesState) {
    let palette = app.palette(ctx);
    let mut open = true;

    let modal = egui::Modal::new(egui::Id::new("categories-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(320.0);
            ui.label(RichText::new(app.t("categories.newTitle")).font(crate::theme::extrabold(19.0)));
            ui.add_space(4.0);
            ui.label(RichText::new(app.t("categories.subtitle")).color(palette.text_secondary));
            ui.add_space(16.0);

            let field = widgets::text_field(
                ui,
                &palette,
                &mut state.new_name,
                &app.t("categories.newPlaceholder"),
                false,
            );
            let submitted = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.add_space(10.0);
            let can_add = !state.new_name.trim().is_empty();
            if (widgets::primary_button(ui, &palette, &app.t("categories.add"), can_add).clicked()
                || submitted)
                && can_add
            {
                let name = state.new_name.trim().to_string();
                if let Some(manager) = app.manager.as_mut() {
                    if let Ok(vault) = manager.vault_mut() {
                        let position = vault.categories.iter().map(|c| c.position + 1).max().unwrap_or(0);
                        vault.categories.push(liauth_core::Category::new(name, position));
                    }
                }
                app.save_vault();
                open = false;
            }
            ui.add_space(10.0);
            ui.label(
                RichText::new(app.t("categories.hintDesktop"))
                    .size(12.0)
                    .color(palette.text_tertiary),
            );
            ui.add_space(8.0);
            if widgets::quiet_button(ui, &palette, &app.t("action.cancel")).clicked() {
                open = false;
            }
        });

    if modal.should_close() {
        open = false;
    }
    if open {
        app.overlay = Overlay::Categories(state);
    }
}

/// Renames or removes one group, reached by right-clicking its chip.
pub fn show_edit_category(app: &mut LiAuthApp, ctx: &egui::Context, mut state: EditCategoryState) {
    let palette = app.palette(ctx);
    let mut open = true;

    let modal = egui::Modal::new(egui::Id::new("edit-category-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(320.0);
            ui.label(RichText::new(app.t("categories.edit")).font(crate::theme::extrabold(19.0)));
            ui.add_space(4.0);
            ui.label(RichText::new(app.t("categories.editSubtitle")).color(palette.text_secondary));
            ui.add_space(16.0);

            let field = widgets::text_field(
                ui,
                &palette,
                &mut state.name,
                &app.t("categories.newPlaceholder"),
                false,
            );
            let submitted = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.add_space(10.0);
            let can_save = !state.name.trim().is_empty();
            if (widgets::primary_button(ui, &palette, &app.t("action.save"), can_save).clicked() || submitted)
                && can_save
            {
                let name = state.name.trim().to_string();
                if let Some(manager) = app.manager.as_mut() {
                    if let Ok(vault) = manager.vault_mut() {
                        if let Some(category) = vault.categories.iter_mut().find(|c| c.id == state.id) {
                            category.name = name;
                        }
                    }
                }
                app.save_vault();
                open = false;
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if widgets::quiet_button(ui, &palette, &app.t("action.cancel")).clicked() {
                    open = false;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::danger_button(ui, &palette, &app.t("categories.delete"), true).clicked() {
                        if let Some(manager) = app.manager.as_mut() {
                            if let Ok(vault) = manager.vault_mut() {
                                vault.remove_category(state.id);
                            }
                        }
                        if app.home.category == Some(state.id) {
                            app.home.category = None;
                        }
                        app.save_vault();
                        open = false;
                    }
                });
            });
        });

    if modal.should_close() {
        open = false;
    }
    if open {
        app.overlay = Overlay::EditCategory(state);
    }
}

pub fn show_qr(
    app: &mut LiAuthApp,
    ctx: &egui::Context,
    account_id: Uuid,
    texture: Option<egui::TextureHandle>,
) {
    let palette = app.palette(ctx);
    let dark = app.is_dark(ctx);

    let texture = texture.or_else(|| {
        let uri = app
            .manager
            .as_ref()
            .and_then(|m| m.vault().ok())
            .and_then(|v| v.account(account_id))
            .map(liauth_core::uri::build)?;
        let image = qr::render(&uri, dark)?;
        Some(ctx.load_texture("account-qr", image, egui::TextureOptions::NEAREST))
    });

    let mut open = true;
    let modal = egui::Modal::new(egui::Id::new("qr-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(300.0);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.label(RichText::new(app.t("qr.title")).font(crate::theme::extrabold(19.0)));
                ui.add_space(4.0);
                ui.label(RichText::new(app.t("qr.subtitle")).color(palette.text_secondary));
                ui.add_space(16.0);
                if let Some(texture) = &texture {
                    ui.add(
                        egui::Image::from_texture(texture)
                            .fit_to_exact_size(egui::vec2(240.0, 240.0))
                            .corner_radius(egui::epaint::CornerRadius::same(12)),
                    );
                }
                ui.add_space(16.0);
                if widgets::secondary_button(ui, &palette, &app.t("action.close")).clicked() {
                    open = false;
                }
            });
        });

    if modal.should_close() {
        open = false;
    }
    if open {
        app.overlay = Overlay::ShowQr { account_id, texture };
    }
}

pub fn show_confirm_delete(app: &mut LiAuthApp, ctx: &egui::Context, account_id: Uuid, title: String) {
    let palette = app.palette(ctx);
    let mut open = true;

    let modal = egui::Modal::new(egui::Id::new("delete-modal"))
        .frame(widgets::modal_frame(&palette))
        .show(ctx, |ui| {
            ui.set_width(300.0);
            ui.label(
                RichText::new(app.tf("delete.title", &[("name", &title)])).font(crate::theme::bold(18.0)),
            );
            ui.add_space(6.0);
            ui.label(RichText::new(app.t("delete.trashSubtitle")).color(palette.text_secondary));
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                if widgets::secondary_button(ui, &palette, &app.t("action.cancel")).clicked() {
                    open = false;
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::danger_button(ui, &palette, &app.t("delete.confirm"), true).clicked() {
                        if let Some(manager) = app.manager.as_mut() {
                            if let Ok(vault) = manager.vault_mut() {
                                vault.trash_account(account_id, unix_now());
                            }
                        }
                        app.save_vault();
                        let message = app.t("toast.accountTrashed");
                        app.toasts.push(message);
                        open = false;
                    }
                });
            });
        });

    if modal.should_close() {
        open = false;
    }
    if open {
        app.overlay = Overlay::ConfirmDelete { account_id, title };
    }
}
