import SwiftUI
import UniformTypeIdentifiers

struct SettingsView: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization
    @Environment(\.dismiss) private var dismiss

    @State private var biometricEnabled = DeviceKeychain.isEnabled
    @State private var privacyShield = true
    @State private var hideCodes = false
    @State private var exporting = false
    @State private var exportPassword = ""
    @State private var exportConfirm = ""
    @State private var exportDocument: BackupDocument?
    @State private var changingPassword = false
    @State private var currentPassword = ""
    @State private var newPassword = ""
    @State private var confirmPassword = ""
    @State private var passwordError: String?
    @State private var versionClicks = 0
    @State private var advancedVisible = false
    @State private var animations = true
    @State private var brandIcons = false
    @State private var autoBackup = false
    @State private var showWebdav = false
    @State private var webdavConfigured = false
    @State private var webdavUrl = ""
    @State private var webdavUser = ""
    @State private var webdavPassword = ""
    @State private var webdavBackupPassword = ""
    @State private var webdavBusy = false
    @State private var webdavError: String?
    @State private var timeStatus: String?

    private let projectUrl = URL(string: "https://github.com/liwidale/liauth")!
    private let developerUrl = URL(string: "https://github.com/liwidale")!

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                HStack {
                    Text(localization.t("settings.title"))
                        .font(.interBold(20))
                        .foregroundColor(Palette.textPrimary)
                    Spacer()
                    Button(localization.t("action.done")) { dismiss() }
                        .font(.inter(14))
                        .foregroundColor(Palette.textSecondary)
                        .buttonStyle(.plain)
                }

                SectionLabel(text: localization.t("settings.appearance"))
                    .padding(.top, 22)

                settingRow(localization.t("settings.language")) {
                    Picker("", selection: Binding(
                        get: { localization.activeCode },
                        set: { localization.setLanguage($0) }
                    )) {
                        ForEach(localization.availableLanguages, id: \.code) { language in
                            Text(language.name).tag(language.code)
                        }
                    }
                    .labelsHidden()
                    .frame(maxWidth: 160)
                }

                settingRow(localization.t("settings.animations")) {
                    Toggle("", isOn: $animations)
                        .labelsHidden()
                        .onChange(of: animations) { store.setSetting("animations", $0 ? "true" : "false") }
                }
                hint(localization.t("settings.animationsHint"))

                settingRow(localization.t("settings.brandIcons")) {
                    Toggle("", isOn: $brandIcons)
                        .labelsHidden()
                        .onChange(of: brandIcons) { store.setSetting("brandIcons", $0 ? "true" : "false") }
                }
                hint(localization.t("settings.brandIconsHint"))

                settingRow(localization.t("settings.theme")) {
                    Picker("", selection: $store.themeMode) {
                        Text(localization.t("settings.themeSystem")).tag("system")
                        Text(localization.t("settings.themeLight")).tag("light")
                        Text(localization.t("settings.themeDark")).tag("dark")
                    }
                    .labelsHidden()
                    .frame(maxWidth: 160)
                }

                SectionLabel(text: localization.t("settings.security"))
                    .padding(.top, 22)

                settingRow(localization.t("settings.hideCodes")) {
                    Toggle("", isOn: $hideCodes)
                        .labelsHidden()
                        .onChange(of: hideCodes) { store.setSetting("hideCodes", $0 ? "true" : "false") }
                }
                hint(localization.t("settings.hideCodesHint"))

                settingRow(localization.t("settings.blockCapture")) {
                    Toggle("", isOn: $privacyShield)
                        .labelsHidden()
                        .onChange(of: privacyShield) { store.privacyShieldEnabled = $0 }
                }
                hint(localization.t("settings.blockCaptureHint"))

                if DeviceKeychain.biometryAvailable {
                    settingRow(localization.t("settings.biometricUnlock")) {
                        Toggle("", isOn: $biometricEnabled)
                            .labelsHidden()
                            .onChange(of: biometricEnabled) { enabled in
                                if enabled {
                                    if !store.enableBiometricUnlock() {
                                        biometricEnabled = false
                                    }
                                } else {
                                    store.disableBiometricUnlock()
                                }
                            }
                    }
                    hint(localization.t("settings.biometricHint"))
                }

                actionRow(localization.t("settings.changePassword")) { changingPassword = true }
                actionRow(localization.t("settings.lockNow")) {
                    dismiss()
                    store.lock()
                }

                SectionLabel(text: localization.t("settings.backup"))
                    .padding(.top, 22)
                actionRow(localization.t("settings.exportBackup")) { exporting = true }
                hint(localization.t("settings.exportHint"))

                settingRow(localization.t("settings.autoBackup")) {
                    Toggle("", isOn: $autoBackup)
                        .labelsHidden()
                        .onChange(of: autoBackup) { store.setAutoBackup(enabled: $0) }
                }
                hint(localization.t("settings.autoBackupHint"))

                settingRow(localization.t("webdav.title")) {
                    if webdavConfigured {
                        HStack(spacing: 12) {
                            Button(localization.t("webdav.syncNow")) {
                                Task {
                                    if await store.webdavSyncNow() {
                                        store.toast = localization.t("webdav.done")
                                    }
                                }
                            }
                            .font(.inter(13, weight: .semibold))
                            .foregroundColor(Palette.textPrimary)
                            .buttonStyle(.plain)
                            Button(localization.t("webdav.disable")) {
                                Task {
                                    _ = await store.webdavConfigure(
                                        url: "", username: "", password: "", backupPassword: ""
                                    )
                                    webdavConfigured = false
                                }
                            }
                            .font(.inter(13))
                            .foregroundColor(Palette.textTertiary)
                            .buttonStyle(.plain)
                        }
                    } else {
                        Button(localization.t("webdav.configure")) { showWebdav = true }
                            .font(.inter(13, weight: .semibold))
                            .foregroundColor(Palette.textPrimary)
                            .buttonStyle(.plain)
                    }
                }
                hint(localization.t("webdav.hint"))

                SectionLabel(text: localization.t("settings.data"))
                    .padding(.top, 22)
                settingRow(localization.t("timeSync.title")) {
                    Button(localization.t("timeSync.run")) {
                        timeStatus = localization.t("timeSync.running")
                        Task {
                            if let offset = await store.syncTimeDrift() {
                                timeStatus = localization.tf(
                                    "timeSync.result", ["seconds": String(offset)]
                                )
                            } else {
                                timeStatus = localization.t("sync.failed")
                            }
                        }
                    }
                    .font(.inter(13, weight: .semibold))
                    .foregroundColor(Palette.textPrimary)
                    .buttonStyle(.plain)
                }
                hint(timeStatus ?? localization.tf(
                    "timeSync.current", ["seconds": String(store.timeDriftSeconds())]
                ))

                SectionLabel(text: localization.t("settings.about"))
                    .padding(.top, 22)
                settingRow(localization.t("settings.project")) {
                    Link("liwidale/liauth", destination: projectUrl)
                        .font(.inter(14))
                        .foregroundColor(Palette.textPrimary)
                }
                settingRow(localization.t("settings.developer")) {
                    Link("liwidale", destination: developerUrl)
                        .font(.inter(14))
                        .foregroundColor(Palette.textPrimary)
                }
                settingRow(localization.t("settings.version")) {
                    Text("2.0.0")
                        .font(.inter(14))
                        .foregroundColor(Palette.textSecondary)
                        .onTapGesture {
                            versionClicks += 1
                            if versionClicks >= 5 && !advancedVisible {
                                advancedVisible = true
                                store.setSetting("advancedVisible", "true")
                                store.toast = localization.t("settings.advancedUnlocked")
                            }
                        }
                }

                if advancedVisible {
                    SectionLabel(text: localization.t("settings.advanced"))
                        .padding(.top, 22)
                    hint(localization.t("settings.advancedHint"))
                }
            }
            .padding(26)
        }
        .background(Palette.background)
        .frame(minWidth: 400, minHeight: 520)
        .onAppear {
            privacyShield = store.privacyShieldEnabled
            hideCodes = store.getSetting("hideCodes") == "true"
            advancedVisible = store.getSetting("advancedVisible") == "true"
            animations = store.getSetting("animations") != "false"
            brandIcons = store.getSetting("brandIcons") == "true"
            autoBackup = store.autoBackupEnabled()
            webdavConfigured = store.webdavIsConfigured()
        }
        .sheet(isPresented: $exporting) { exportSheet }
        .sheet(isPresented: $changingPassword) { changePasswordSheet }
        .sheet(isPresented: $showWebdav) { webdavSheet }
        .fileExporter(
            isPresented: Binding(
                get: { exportDocument != nil },
                set: { if !$0 { exportDocument = nil } }
            ),
            document: exportDocument,
            contentType: .data,
            defaultFilename: "liauth-backup.liauth"
        ) { result in
            if case .success = result {
                store.toast = localization.t("toast.backupSaved")
            }
            exportDocument = nil
        }
    }

    private var exportSheet: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(localization.t("export.title"))
                .font(.inter(18, weight: .bold))
                .foregroundColor(Palette.textPrimary)
            Text(localization.t("export.subtitle"))
                .font(.inter(13))
                .foregroundColor(Palette.textSecondary)
            LiAuthField(hint: localization.t("field.password"), text: $exportPassword, secure: true)
            LiAuthField(hint: localization.t("field.confirmPassword"), text: $exportConfirm, secure: true)
            PrimaryButton(
                title: localization.t("export.save"),
                enabled: exportPassword.count >= 4 && exportPassword == exportConfirm
            ) {
                if let data = store.exportBackup(password: exportPassword) {
                    exporting = false
                    exportDocument = BackupDocument(data: data)
                }
                exportPassword = ""
                exportConfirm = ""
            }
            Button(localization.t("action.cancel")) { exporting = false }
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
        }
        .padding(26)
        .background(Palette.background)
        .frame(minWidth: 360, minHeight: 300)
    }

    private var webdavSheet: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(localization.t("webdav.title"))
                .font(.inter(18, weight: .bold))
                .foregroundColor(Palette.textPrimary)
            LiAuthField(hint: localization.t("webdav.url"), text: $webdavUrl)
            LiAuthField(hint: localization.t("webdav.username"), text: $webdavUser)
            LiAuthField(hint: localization.t("webdav.password"), text: $webdavPassword, secure: true)
            LiAuthField(
                hint: localization.t("webdav.backupPassword"),
                text: $webdavBackupPassword,
                secure: true
            )
            Text(localization.t("webdav.backupPasswordHint"))
                .font(.inter(12))
                .foregroundColor(Palette.textTertiary)
            if let webdavError {
                Text(webdavError)
                    .font(.inter(13))
                    .foregroundColor(Palette.textSecondary)
            }
            PrimaryButton(
                title: webdavBusy ? localization.t("webdav.uploading") : localization.t("action.save"),
                enabled: !webdavBusy
                    && webdavUrl.trimmingCharacters(in: .whitespaces).hasPrefix("http")
                    && webdavBackupPassword.count >= 4
            ) {
                webdavBusy = true
                webdavError = nil
                Task {
                    let configured = await store.webdavConfigure(
                        url: webdavUrl.trimmingCharacters(in: .whitespaces),
                        username: webdavUser,
                        password: webdavPassword,
                        backupPassword: webdavBackupPassword
                    )
                    webdavBusy = false
                    if configured {
                        webdavConfigured = true
                        showWebdav = false
                        webdavPassword = ""
                        webdavBackupPassword = ""
                        store.toast = localization.t("webdav.done")
                    } else {
                        webdavError = localization.t("sync.failed")
                    }
                }
            }
            Button(localization.t("action.cancel")) { showWebdav = false }
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
        }
        .padding(26)
        .background(Palette.background)
        .frame(minWidth: 380, minHeight: 380)
    }

    private var changePasswordSheet: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(localization.t("settings.changePassword"))
                .font(.inter(18, weight: .bold))
                .foregroundColor(Palette.textPrimary)
            LiAuthField(hint: localization.t("field.currentPassword"), text: $currentPassword, secure: true)
            LiAuthField(hint: localization.t("field.newPassword"), text: $newPassword, secure: true)
            LiAuthField(hint: localization.t("field.confirmPassword"), text: $confirmPassword, secure: true)
            if let passwordError {
                Text(passwordError)
                    .font(.inter(13))
                    .foregroundColor(Palette.textSecondary)
            }
            PrimaryButton(
                title: localization.t("action.save"),
                enabled: !currentPassword.isEmpty && newPassword.count >= 4 && newPassword == confirmPassword
            ) {
                if store.changePassword(current: currentPassword, new: newPassword) {
                    biometricEnabled = false
                    store.toast = localization.t("toast.passwordChanged")
                    changingPassword = false
                } else {
                    passwordError = localization.t("lock.wrongPassword")
                    currentPassword = ""
                }
            }
            Button(localization.t("action.cancel")) { changingPassword = false }
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
        }
        .padding(26)
        .background(Palette.background)
        .frame(minWidth: 360, minHeight: 340)
    }

    private func settingRow<Content: View>(_ label: String, @ViewBuilder content: () -> Content) -> some View {
        HStack {
            Text(label)
                .font(.inter(15))
                .foregroundColor(Palette.textPrimary)
            Spacer()
            content()
        }
        .padding(.vertical, 8)
    }

    private func actionRow(_ label: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack {
                Text(label)
                    .font(.inter(15))
                    .foregroundColor(Palette.textPrimary)
                Spacer()
                Image(systemName: "chevron.right")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundColor(Palette.textTertiary)
            }
            .padding(.vertical, 10)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    private func hint(_ text: String) -> some View {
        Text(text)
            .font(.inter(12))
            .foregroundColor(Palette.textTertiary)
            .padding(.bottom, 4)
    }
}

struct BackupDocument: FileDocument {
    static var readableContentTypes: [UTType] { [.data] }

    let data: Data

    init(data: Data) {
        self.data = data
    }

    init(configuration: ReadConfiguration) throws {
        data = configuration.file.regularFileContents ?? Data()
    }

    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        FileWrapper(regularFileWithContents: data)
    }
}
