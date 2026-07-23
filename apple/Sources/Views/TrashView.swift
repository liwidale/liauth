import SwiftUI

struct TrashView: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization
    @Environment(\.dismiss) private var dismiss

    @State private var entries: [TrashedAccountView] = []

    private var brandIcons: Bool { store.getSetting("brandIcons") == "true" }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(localization.t("trash.title"))
                .font(.interBold(20))
                .foregroundColor(Palette.textPrimary)
            Text(localization.t("trash.subtitle"))
                .font(.inter(13.5))
                .foregroundColor(Palette.textSecondary)
                .padding(.top, 4)

            if entries.isEmpty {
                Text(localization.t("trash.empty"))
                    .font(.inter(14))
                    .foregroundColor(Palette.textTertiary)
                    .frame(maxWidth: .infinity)
                    .padding(.top, 48)
            } else {
                ScrollView {
                    LazyVStack(spacing: 10) {
                        ForEach(entries, id: \.id) { entry in
                            trashCard(entry)
                        }
                    }
                    .padding(.vertical, 14)
                }
            }

            Spacer(minLength: 0)
            Button(localization.t("action.close")) { dismiss() }
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
                .padding(.top, 12)
        }
        .padding(26)
        .background(Palette.background)
        .frame(minWidth: 400, minHeight: 440)
        .onAppear { entries = store.trashedAccounts() }
    }

    private func trashCard(_ entry: TrashedAccountView) -> some View {
        let title = entry.issuer.isEmpty ? entry.name : entry.issuer
        let now = Int64(Date().timeIntervalSince1970)
        let daysLeft = max(0, entry.purgeAt - now + 86_399) / 86_400
        return HStack(spacing: 14) {
            AvatarView(title: title, size: 38, branded: brandIcons)
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.inter(15, weight: .semibold))
                    .foregroundColor(Palette.textPrimary)
                if !entry.issuer.isEmpty && !entry.name.isEmpty {
                    Text(entry.name)
                        .font(.inter(12.5))
                        .foregroundColor(Palette.textSecondary)
                }
                Text(localization.tf("trash.daysLeft", ["days": String(daysLeft)]))
                    .font(.inter(11.5))
                    .foregroundColor(Palette.textTertiary)
            }
            Spacer()
            VStack(alignment: .trailing, spacing: 6) {
                Button(localization.t("trash.restore")) {
                    store.restoreAccount(id: entry.id)
                    entries = store.trashedAccounts()
                    store.toast = localization.t("trash.restored")
                }
                .font(.inter(13, weight: .semibold))
                .foregroundColor(Palette.textPrimary)
                .buttonStyle(.plain)
                Button(localization.t("trash.deleteForever")) {
                    store.purgeAccount(id: entry.id)
                    entries = store.trashedAccounts()
                    store.toast = localization.t("toast.accountDeleted")
                }
                .font(.inter(13))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .cardStyle()
    }
}
