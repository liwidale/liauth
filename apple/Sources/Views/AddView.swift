import SwiftUI
import UniformTypeIdentifiers

struct AddView: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization
    @Environment(\.dismiss) private var dismiss

    @State private var issuer = ""
    @State private var name = ""
    @State private var secret = ""
    @State private var error: String?
    @State private var scanning = false
    @State private var importing = false
    @State private var pendingImport: Data?
    @State private var importPassword = ""
    @State private var importError: String?
    @State private var showAdvanced = false
    @State private var algorithm = "SHA1"
    @State private var digits: UInt32 = 6
    @State private var period: UInt32 = 30

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                Text(localization.t("add.title"))
                    .font(.interBold(20))
                    .foregroundColor(Palette.textPrimary)
                Text(localization.t("add.subtitle"))
                    .font(.inter(13.5))
                    .foregroundColor(Palette.textSecondary)
                    .padding(.top, 4)

                VStack(spacing: 10) {
                    #if os(iOS)
                    PrimaryButton(title: localization.t("add.scanCamera")) { scanning = true }
                    #endif
                    SecondaryButton(title: localization.t("add.importFile")) { importing = true }
                }
                .padding(.top, 18)

                SectionLabel(text: localization.t("add.manualSection"))
                    .padding(.top, 26)

                VStack(spacing: 10) {
                    LiAuthField(hint: localization.t("field.service"), text: $issuer)
                    LiAuthField(hint: localization.t("field.account"), text: $name)
                    LiAuthField(hint: localization.t("field.key"), text: $secret)
                }
                .padding(.top, 10)

                Button(showAdvanced ? localization.t("add.advancedHide") : localization.t("add.advancedShow")) {
                    showAdvanced.toggle()
                }
                .font(.inter(13))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
                .padding(.top, 10)

                if showAdvanced {
                    VStack(alignment: .leading, spacing: 8) {
                        advancedRow(localization.t("advanced.algorithm"), ["SHA1", "SHA256", "SHA512"], algorithm) {
                            algorithm = $0
                        }
                        advancedRow(localization.t("advanced.digits"), ["6", "7", "8"], String(digits)) {
                            digits = UInt32($0) ?? 6
                        }
                        advancedRow(localization.t("advanced.period"), ["15", "30", "60", "90"], String(period)) {
                            period = UInt32($0) ?? 30
                        }
                    }
                    .padding(.top, 8)
                }

                if let error {
                    Text(error)
                        .font(.inter(13))
                        .foregroundColor(Palette.textSecondary)
                        .padding(.top, 10)
                }

                PrimaryButton(title: localization.t("add.save"), enabled: ready) {
                    save()
                }
                .padding(.top, 20)

                Button(localization.t("action.cancel")) { dismiss() }
                    .font(.inter(14))
                    .foregroundColor(Palette.textSecondary)
                    .buttonStyle(.plain)
                    .frame(maxWidth: .infinity)
                    .padding(.top, 12)
            }
            .padding(26)
        }
        .background(Palette.background)
        .frame(minWidth: 380, minHeight: 480)
        #if os(iOS)
        .fullScreenCover(isPresented: $scanning) {
            QrScannerView { content in
                scanning = false
                handleScanned(content)
            }
        }
        #endif
        .fileImporter(isPresented: $importing, allowedContentTypes: [.data, .json, .text]) { result in
            guard case .success(let url) = result else { return }
            let scoped = url.startAccessingSecurityScopedResource()
            defer { if scoped { url.stopAccessingSecurityScopedResource() } }
            guard let data = try? Data(contentsOf: url) else { return }
            handleImport(data, password: nil)
        }
        .alert(localization.t("import.passwordTitle"), isPresented: Binding(
            get: { pendingImport != nil },
            set: { if !$0 { pendingImport = nil; importPassword = "" } }
        )) {
            SecureField(localization.t("field.password"), text: $importPassword)
            Button(localization.t("import.unlock")) {
                if let data = pendingImport {
                    handleImport(data, password: importPassword)
                }
            }
            Button(localization.t("action.cancel"), role: .cancel) {
                pendingImport = nil
                importPassword = ""
            }
        } message: {
            Text(importError ?? localization.t("import.passwordSubtitle"))
        }
    }

    private var ready: Bool {
        let trimmed = secret.trimmingCharacters(in: .whitespacesAndNewlines)
        return !trimmed.isEmpty && (trimmed.hasPrefix("otpauth") || !issuer.trimmingCharacters(in: .whitespaces).isEmpty)
    }

    private func advancedRow(
        _ label: String,
        _ options: [String],
        _ selected: String,
        onSelect: @escaping (String) -> Void
    ) -> some View {
        HStack {
            Text(label)
                .font(.inter(13))
                .foregroundColor(Palette.textSecondary)
            Spacer()
            ForEach(options, id: \.self) { option in
                Button(option) { onSelect(option) }
                    .font(.inter(13, weight: option == selected ? .bold : .regular))
                    .foregroundColor(option == selected ? Palette.textPrimary : Palette.textTertiary)
                    .buttonStyle(.plain)
            }
        }
    }

    private func save() {
        let trimmed = secret.trimmingCharacters(in: .whitespacesAndNewlines)
        let added: Bool
        if showAdvanced && !trimmed.hasPrefix("otpauth") {
            added = store.addManualAdvanced(
                issuer: issuer, name: name, secret: secret,
                algorithm: algorithm, digits: digits, period: period
            )
        } else {
            added = store.addManual(issuer: issuer, name: name, secret: secret)
        }
        if added {
            store.toast = localization.t("toast.accountAdded")
            dismiss()
        } else {
            error = localization.t("add.invalidKey")
        }
    }

    private func handleScanned(_ content: String) {
        if content.hasPrefix("otpauth-migration://") {
            handleImport(Data(content.utf8), password: nil)
            return
        }
        if store.addFromUri(content) {
            store.toast = localization.t("toast.accountAdded")
            dismiss()
        } else {
            error = localization.t("add.noQrFound")
        }
    }

    private func handleImport(_ data: Data, password: String?) {
        switch store.importData(data, password: password) {
        case .done(let added, let skipped):
            pendingImport = nil
            importPassword = ""
            store.toast = localization.tf(
                "toast.imported",
                ["added": String(added), "skipped": String(skipped)]
            )
            dismiss()
        case .passwordRequired:
            importError = password == nil ? nil : localization.t("lock.wrongPassword")
            importPassword = ""
            pendingImport = data
        case .failed:
            pendingImport = nil
            error = localization.t("import.unrecognized")
        }
    }
}
