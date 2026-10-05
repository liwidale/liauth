import SwiftUI

struct EditView: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization
    @Environment(\.dismiss) private var dismiss

    let account: AccountView

    @State private var issuer: String
    @State private var name: String
    @State private var categoryId: String?
    @State private var newCategory = ""
    @State private var notes: String
    @State private var recoveryCodes: String

    init(account: AccountView) {
        self.account = account
        _issuer = State(initialValue: account.issuer)
        _name = State(initialValue: account.name)
        _categoryId = State(initialValue: account.categoryId)
        _notes = State(initialValue: account.notes)
        _recoveryCodes = State(initialValue: account.recoveryCodes.joined(separator: "\n"))
    }

    var body: some View {
        ScrollView {
            editorBody
        }
        .background(Palette.background)
        .frame(minWidth: 380, minHeight: 480)
    }

    private var editorBody: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(localization.t("edit.title"))
                .font(.interBold(20))
                .foregroundColor(Palette.textPrimary)

            VStack(spacing: 10) {
                LiAuthField(hint: localization.t("field.service"), text: $issuer)
                LiAuthField(hint: localization.t("field.account"), text: $name)
            }
            .padding(.top, 16)

            SectionLabel(text: localization.t("edit.category"))
                .padding(.top, 22)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    Chip(text: localization.t("edit.noCategory"), selected: categoryId == nil) {
                        categoryId = nil
                    }
                    ForEach(store.categories, id: \.id) { category in
                        Chip(text: category.name, selected: categoryId == category.id) {
                            categoryId = category.id
                        }
                    }
                }
            }
            .padding(.top, 8)

            HStack(spacing: 10) {
                LiAuthField(hint: localization.t("categories.newPlaceholder"), text: $newCategory)
                Button(localization.t("categories.add")) {
                    let trimmed = newCategory.trimmingCharacters(in: .whitespaces)
                    guard !trimmed.isEmpty else { return }
                    store.addCategory(name: trimmed)
                    newCategory = ""
                }
                .font(.inter(14, weight: .medium))
                .foregroundColor(Palette.textPrimary)
                .buttonStyle(.plain)
            }
            .padding(.top, 10)

            SectionLabel(text: localization.t("edit.notes"))
                .padding(.top, 20)
            NotesEditor(text: $notes, minHeight: 60)
                .padding(.top, 8)

            SectionLabel(text: localization.t("edit.recoveryCodes"))
                .padding(.top, 16)
            NotesEditor(text: $recoveryCodes, minHeight: 80, monospaced: true)
                .padding(.top, 8)

            PrimaryButton(title: localization.t("action.save")) {
                store.updateAccount(
                    id: account.id,
                    issuer: issuer.trimmingCharacters(in: .whitespaces),
                    name: name.trimmingCharacters(in: .whitespaces),
                    categoryId: categoryId,
                    pinned: account.pinned
                )
                store.updateNotes(
                    id: account.id,
                    notes: notes.trimmingCharacters(in: .whitespacesAndNewlines),
                    recoveryCodes: recoveryCodes
                        .split(separator: "\n")
                        .map { $0.trimmingCharacters(in: .whitespaces) }
                        .filter { !$0.isEmpty }
                )
                store.toast = localization.t("toast.saved")
                dismiss()
            }
            .padding(.top, 24)

            Button(localization.t("action.cancel")) { dismiss() }
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
                .padding(.top, 12)
        }
        .padding(26)
    }
}

/// Bordered multi-line editor matching the LiAuth field style.
struct NotesEditor: View {
    @Binding var text: String
    var minHeight: CGFloat = 60
    var monospaced: Bool = false

    var body: some View {
        TextEditor(text: $text)
            .font(monospaced ? .code(13) : .inter(14))
            .foregroundColor(Palette.textPrimary)
            .scrollContentBackground(.hidden)
            .padding(8)
            .frame(minHeight: minHeight)
            .background(RoundedRectangle(cornerRadius: Radius.control).fill(Palette.surface))
            .overlay(RoundedRectangle(cornerRadius: Radius.control).stroke(Palette.border, lineWidth: 1))
    }
}
