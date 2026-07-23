import SwiftUI

struct HomeView: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization

    @State private var search = ""
    @State private var selectedCategory: String?
    @State private var showAdd = false
    @State private var showSync = false
    @State private var showSettings = false
    @State private var showTrash = false
    @State private var showNewCategory = false
    @State private var editing: AccountView?
    @State private var editingCategory: CategoryView?
    @State private var deleting: AccountView?
    @State private var revealed: Set<String> = []
    /// Non-nil while batch selection mode is active.
    @State private var selection: Set<String>?

    private var hideCodes: Bool { store.getSetting("hideCodes") == "true" }
    private var brandIcons: Bool { store.getSetting("brandIcons") == "true" }
    private var animations: Bool { store.getSetting("animations") != "false" }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            header
                .padding(.horizontal, 20)
                .padding(.top, 16)

            LiAuthField(hint: localization.t("home.search"), text: $search)
                .padding(.horizontal, 20)
                .padding(.top, 12)

            categoryChips
                .padding(.top, 10)

            if filteredResults.isEmpty {
                emptyState
            } else {
                ScrollView {
                    LazyVStack(spacing: 10) {
                        ForEach(filteredResults, id: \.account.id) { result in
                            row(for: result)
                        }
                    }
                    .padding(.horizontal, 20)
                    .padding(.vertical, 14)
                }
                .animation(animations ? .easeOut(duration: 0.18) : nil, value: filteredResults.map(\.account.id))
            }
            if let selected = selection {
                selectionBar(selected)
            }
            Spacer(minLength: 0)
        }
        .frame(minWidth: 380, minHeight: 560)
        .sheet(isPresented: $showAdd) { AddView() }
        .sheet(isPresented: $showSync) { SyncView() }
        .sheet(isPresented: $showSettings) { SettingsView() }
        .sheet(isPresented: $showTrash) { TrashView() }
        .sheet(isPresented: $showNewCategory) { NewCategorySheet() }
        .sheet(item: $editing) { account in EditView(account: account) }
        .sheet(item: $editingCategory) { category in EditCategorySheet(category: category) }
        .confirmationDialog(
            localization.tf("delete.title", ["name": deleting.map { $0.issuer.isEmpty ? $0.name : $0.issuer } ?? ""]),
            isPresented: Binding(get: { deleting != nil }, set: { if !$0 { deleting = nil } }),
            titleVisibility: .visible
        ) {
            Button(localization.t("delete.confirm"), role: .destructive) {
                if let account = deleting {
                    store.deleteAccount(id: account.id)
                    store.toast = localization.t("toast.accountTrashed")
                }
                deleting = nil
            }
            Button(localization.t("action.cancel"), role: .cancel) { deleting = nil }
        } message: {
            Text(localization.t("delete.trashSubtitle"))
        }
    }

    @ViewBuilder
    private func row(for result: SearchResultView) -> some View {
        let account = result.account
        SwipeableRow(
            enabled: selection == nil,
            onSwipeLeft: { deleting = account },
            onSwipeRight: { editing = account }
        ) {
            TokenCard(
                result: result,
                code: store.codes[account.id],
                hidden: hideCodes && !revealed.contains(account.id),
                brandIcons: brandIcons,
                selecting: selection != nil,
                selected: selection?.contains(account.id) == true,
                onToggleSelect: { toggleSelection(account.id) },
                onStartSelection: { selection = [account.id] },
                onReveal: { revealed.insert(account.id) },
                onCopy: { copy($0) },
                onEdit: { editing = account },
                onDelete: { deleting = account },
                onTogglePin: {
                    store.updateAccount(
                        id: account.id,
                        issuer: account.issuer,
                        name: account.name,
                        categoryId: account.categoryId,
                        pinned: !account.pinned
                    )
                },
                onNextCode: { store.advanceCounter(id: account.id) }
            )
        }
    }

    private func toggleSelection(_ id: String) {
        guard var current = selection else { return }
        if current.contains(id) {
            current.remove(id)
        } else {
            current.insert(id)
        }
        selection = current
    }

    private func selectionBar(_ selected: Set<String>) -> some View {
        HStack(spacing: 10) {
            Text(localization.tf("batch.selected", ["count": String(selected.count)]))
                .font(.inter(14, weight: .bold))
                .foregroundColor(Palette.textPrimary)
            Spacer()
            Menu {
                Button(localization.t("edit.noCategory")) { moveSelection(selected, to: nil) }
                ForEach(store.categories, id: \.id) { category in
                    Button(category.name) { moveSelection(selected, to: category.id) }
                }
            } label: {
                Text(localization.t("batch.moveToGroup"))
                    .font(.inter(13, weight: .semibold))
                    .foregroundColor(Palette.textPrimary)
            }
            .menuStyle(.borderlessButton)
            .fixedSize()
            Button(localization.t("batch.delete")) {
                let trashed = store.deleteAccounts(ids: Array(selected))
                store.toast = localization.tf("toast.movedToTrash", ["count": String(trashed)])
                selection = nil
            }
            .buttonStyle(.plain)
            .font(.inter(13, weight: .semibold))
            .foregroundColor(Palette.textPrimary)
            .disabled(selected.isEmpty)
            Button(localization.t("action.cancel")) { selection = nil }
                .buttonStyle(.plain)
                .font(.inter(13))
                .foregroundColor(Palette.textSecondary)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(RoundedRectangle(cornerRadius: Radius.card).fill(Palette.surface))
        .overlay(RoundedRectangle(cornerRadius: Radius.card).stroke(Palette.border, lineWidth: 1))
        .padding(.horizontal, 20)
        .padding(.bottom, 12)
    }

    private func moveSelection(_ selected: Set<String>, to categoryId: String?) {
        let moved = store.moveAccounts(ids: Array(selected), categoryId: categoryId)
        store.toast = localization.tf("toast.movedToGroup", ["count": String(moved)])
        selection = nil
    }

    private var header: some View {
        // Same set and order as the Windows header.
        HStack(spacing: 6) {
            Wordmark(height: 26)
            Spacer()
            headerButton("plus", localization.t("nav.add")) { showAdd = true }
            headerButton("arrow.left.arrow.right", localization.t("nav.sync")) { showSync = true }
            headerButton("trash", localization.t("trash.title")) { showTrash = true }
            headerButton("slider.horizontal.3", localization.t("nav.settings")) { showSettings = true }
        }
    }

    private func headerButton(_ icon: String, _ label: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: 15, weight: .medium))
                .foregroundColor(Palette.textPrimary)
                .frame(width: 36, height: 36)
                .overlay(RoundedRectangle(cornerRadius: Radius.control).stroke(Palette.border, lineWidth: 1))
        }
        .buttonStyle(.plain)
        .accessibilityLabel(label)
        .help(label)
    }

    private var categoryChips: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                Chip(text: localization.t("home.all"), selected: selectedCategory == nil) {
                    selectedCategory = nil
                }
                // Tap filters, context menu edits — so the chip keeps its
                // primary job while staying editable in place.
                ForEach(store.categories, id: \.id) { category in
                    Chip(text: category.name, selected: selectedCategory == category.id) {
                        selectedCategory = selectedCategory == category.id ? nil : category.id
                    }
                    .contextMenu {
                        Button(localization.t("categories.edit")) { editingCategory = category }
                    }
                }
                Chip(text: localization.t("home.manageCategories"), selected: false) {
                    showNewCategory = true
                }
            }
            .padding(.horizontal, 20)
        }
    }

    /// Typo-tolerant results from the Rust core, with highlight positions.
    private var filteredResults: [SearchResultView] {
        // `store.accounts` participates so the list refreshes on data changes.
        _ = store.accounts
        return store.searchAccounts(search).filter { result in
            selectedCategory == nil || result.account.categoryId == selectedCategory
        }
    }

    private var emptyState: some View {
        VStack(spacing: 6) {
            LogoMark(size: 72)
                .padding(.bottom, 14)
            Text(search.isEmpty && selectedCategory == nil
                ? localization.t("home.emptyTitle")
                : localization.t("home.noResultsTitle"))
                .font(.inter(18, weight: .bold))
                .foregroundColor(Palette.textPrimary)
            Text(search.isEmpty && selectedCategory == nil
                ? localization.t("home.emptySubtitle")
                : localization.t("home.noResultsSubtitle"))
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.top, 72)
    }

    private func copy(_ code: String) {
        #if os(macOS)
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(code, forType: .string)
        #else
        UIPasteboard.general.string = code
        #endif
        store.toast = localization.t("toast.codeCopied")
    }
}

/// Horizontal swipe wrapper: left reveals delete, right reveals edit.
struct SwipeableRow<Content: View>: View {
    let enabled: Bool
    let onSwipeLeft: () -> Void
    let onSwipeRight: () -> Void
    @ViewBuilder let content: () -> Content

    @State private var offset: CGFloat = 0

    var body: some View {
        content()
            .offset(x: offset)
            .simultaneousGesture(
                enabled ? DragGesture(minimumDistance: 24)
                    .onChanged { value in
                        guard abs(value.translation.width) > abs(value.translation.height) else { return }
                        offset = max(-96, min(96, value.translation.width))
                    }
                    .onEnded { value in
                        if value.translation.width < -72 {
                            onSwipeLeft()
                        } else if value.translation.width > 72 {
                            onSwipeRight()
                        }
                        withAnimation(.easeOut(duration: 0.15)) { offset = 0 }
                    } : nil
            )
    }
}

struct TokenCard: View {
    @EnvironmentObject private var localization: Localization

    let result: SearchResultView
    let code: CodeView?
    let hidden: Bool
    var brandIcons: Bool = false
    var selecting: Bool = false
    var selected: Bool = false
    var onToggleSelect: () -> Void = {}
    var onStartSelection: () -> Void = {}
    let onReveal: () -> Void
    let onCopy: (String) -> Void
    let onEdit: () -> Void
    let onDelete: () -> Void
    let onTogglePin: () -> Void
    let onNextCode: () -> Void

    private var account: AccountView { result.account }
    private var title: String { account.issuer.isEmpty ? account.name : account.issuer }
    private var titleIndices: [UInt32] {
        account.issuer.isEmpty ? result.nameIndices : result.issuerIndices
    }

    var body: some View {
        HStack(spacing: 14) {
            if selecting {
                Image(systemName: selected ? "checkmark.square.fill" : "square")
                    .font(.system(size: 18))
                    .foregroundColor(Palette.textPrimary)
            }
            AvatarView(title: title, branded: brandIcons)
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    HighlightedText(
                        text: title,
                        indices: titleIndices,
                        font: .inter(15, weight: .semibold),
                        color: Palette.textPrimary
                    )
                    if account.pinned {
                        PinMarker()
                    }
                }
                if !account.issuer.isEmpty && !account.name.isEmpty {
                    HighlightedText(
                        text: account.name,
                        indices: result.nameIndices,
                        font: .inter(12.5),
                        color: Palette.textSecondary
                    )
                }
                Text(hidden ? maskCode(formatCode(code?.code ?? "")) : formatCode(code?.code ?? ""))
                    .font(.code(26))
                    .foregroundColor(Palette.textPrimary)
                if !account.isCounterBased, let code {
                    CountdownBar(fraction: Double(code.secondsRemaining) / Double(max(code.period, 1)))
                        .padding(.top, 6)
                }
            }
            Spacer()
            if account.isCounterBased {
                Button(action: onNextCode) {
                    Image(systemName: "arrow.clockwise")
                        .foregroundColor(Palette.textPrimary)
                }
                .buttonStyle(.plain)
                .help(localization.t("home.nextCode"))
            } else if let code {
                Text(String(format: "%02d", code.secondsRemaining))
                    .font(.code(13))
                    .foregroundColor(Palette.textTertiary)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 14)
        .cardStyle()
        .contentShape(Rectangle())
        .onTapGesture {
            if selecting {
                onToggleSelect()
            } else if hidden {
                onReveal()
            } else if let value = code?.code {
                onCopy(value)
            }
        }
        .contextMenu {
            if !selecting {
                Button(localization.t("menu.copyCode")) {
                    if let value = code?.code { onCopy(value) }
                }
                Button(localization.t("menu.select"), action: onStartSelection)
                Button(localization.t("menu.edit"), action: onEdit)
                Button(account.pinned ? localization.t("menu.unpin") : localization.t("menu.pin"), action: onTogglePin)
                Divider()
                Button(localization.t("menu.delete"), role: .destructive, action: onDelete)
            }
        }
    }
}

func maskCode(_ formatted: String) -> String {
    String(formatted.map { $0.isWhitespace ? $0 : Character(UnicodeScalar(0x2022)!) })
}

extension AccountView: Identifiable {}
extension CategoryView: Identifiable {}

/// Creates a group and nothing else; existing groups are edited from the
/// context menu on their chip.
struct NewCategorySheet: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization
    @Environment(\.dismiss) private var dismiss

    @State private var name = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(localization.t("categories.newTitle"))
                .font(.inter(18, weight: .bold))
                .foregroundColor(Palette.textPrimary)
            Text(localization.t("categories.subtitle"))
                .font(.inter(13))
                .foregroundColor(Palette.textSecondary)
            LiAuthField(hint: localization.t("categories.newPlaceholder"), text: $name)
            Text(localization.t("categories.hintMobile"))
                .font(.inter(12))
                .foregroundColor(Palette.textTertiary)
            PrimaryButton(
                title: localization.t("categories.add"),
                enabled: !name.trimmingCharacters(in: .whitespaces).isEmpty
            ) {
                store.addCategory(name: name.trimmingCharacters(in: .whitespaces))
                dismiss()
            }
            Button(localization.t("action.cancel")) { dismiss() }
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
        }
        .padding(26)
        .background(Palette.background)
        .frame(minWidth: 340, minHeight: 280)
    }
}

/// Renames or removes one group.
struct EditCategorySheet: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization
    @Environment(\.dismiss) private var dismiss

    let category: CategoryView
    @State private var name: String

    init(category: CategoryView) {
        self.category = category
        _name = State(initialValue: category.name)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(localization.t("categories.edit"))
                .font(.inter(18, weight: .bold))
                .foregroundColor(Palette.textPrimary)
            Text(localization.t("categories.editSubtitle"))
                .font(.inter(13))
                .foregroundColor(Palette.textSecondary)
            LiAuthField(hint: localization.t("categories.newPlaceholder"), text: $name)
            PrimaryButton(
                title: localization.t("action.save"),
                enabled: !name.trimmingCharacters(in: .whitespaces).isEmpty
            ) {
                store.renameCategory(id: category.id, name: name.trimmingCharacters(in: .whitespaces))
                dismiss()
            }
            Button(localization.t("categories.delete"), role: .destructive) {
                store.deleteCategory(id: category.id)
                dismiss()
            }
            .font(.inter(14))
            .buttonStyle(.plain)
            .frame(maxWidth: .infinity)
            Button(localization.t("action.cancel")) { dismiss() }
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
        }
        .padding(26)
        .background(Palette.background)
        .frame(minWidth: 340, minHeight: 300)
    }
}

#if os(iOS)
import UIKit
#endif
