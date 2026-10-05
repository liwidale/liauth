import Foundation
import SwiftUI

enum VaultUiState {
    case onboarding
    case locked
    case unlocked
}

@MainActor
final class VaultStore: ObservableObject {
    @Published private(set) var state: VaultUiState = .locked
    @Published private(set) var accounts: [AccountView] = []
    @Published private(set) var codes: [String: CodeView] = [:]
    @Published private(set) var categories: [CategoryView] = []
    @Published var toast: String?
    @Published var privacyShieldVisible = false

    private let engine: LiAuthEngine
    private let vaultPath: String
    private var timer: Timer?

    var privacyShieldEnabled = UserDefaults.standard.object(forKey: "liauth.privacyShield") as? Bool ?? true {
        didSet { UserDefaults.standard.set(privacyShieldEnabled, forKey: "liauth.privacyShield") }
    }

    @Published var themeMode: String = UserDefaults.standard.string(forKey: "liauth.theme") ?? "system" {
        didSet { UserDefaults.standard.set(themeMode, forKey: "liauth.theme") }
    }

    var colorScheme: ColorScheme? {
        switch themeMode {
        case "light": return .light
        case "dark": return .dark
        default: return nil
        }
    }

    init() {
        // The vault lives in the App Group container so the credential
        // provider extension can reach the same file; older installs are
        // migrated from Application Support transparently.
        let legacyDirectory = FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("LiAuth", isDirectory: true)
        let legacyPath = legacyDirectory.appendingPathComponent("vault.liauth").path

        let groupContainer = FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: "group.com.liwidale.liauth")
        let directory = (groupContainer ?? legacyDirectory.deletingLastPathComponent())
            .appendingPathComponent("LiAuth", isDirectory: true)
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        vaultPath = directory.appendingPathComponent("vault.liauth").path

        if groupContainer != nil,
           FileManager.default.fileExists(atPath: legacyPath),
           !FileManager.default.fileExists(atPath: vaultPath) {
            try? FileManager.default.moveItem(atPath: legacyPath, toPath: vaultPath)
        }

        engine = LiAuthEngine.newMobile(vaultPath: vaultPath)
        state = engine.vaultExists() ? .locked : .onboarding

        timer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.refreshCodes()
            }
        }
    }

    func handleScenePhase(_ phase: ScenePhase) {
        if privacyShieldEnabled {
            privacyShieldVisible = phase != .active
        }
    }

    func createVault(password: String) -> Bool {
        do {
            try engine.createVault(password: password)
            state = .unlocked
            refreshAll()
            return true
        } catch {
            return false
        }
    }

    func unlock(password: String) -> Bool {
        do {
            _ = try engine.unlock(password: password)
            state = .unlocked
            refreshAll()
            return true
        } catch {
            return false
        }
    }

    func unlockWithDeviceKey(_ key: Data) -> Bool {
        do {
            _ = try engine.unlockWithSlot(slot: DeviceKeychain.slotName, key: key)
            state = .unlocked
            refreshAll()
            return true
        } catch {
            return false
        }
    }

    /// Seconds until the next password attempt is accepted (anti-brute-force).
    func lockoutRemainingSeconds() -> UInt64 {
        engine.lockoutRemainingSeconds()
    }

    func enableBiometricUnlock() -> Bool {
        var key = Data(count: 32)
        let result = key.withUnsafeMutableBytes { SecRandomCopyBytes(kSecRandomDefault, 32, $0.baseAddress!) }
        guard result == errSecSuccess else { return false }
        do {
            try engine.addKeySlot(slot: DeviceKeychain.slotName, key: key)
        } catch {
            return false
        }
        guard DeviceKeychain.store(key: key) else {
            try? engine.removeKeySlot(slot: DeviceKeychain.slotName)
            return false
        }
        return true
    }

    func disableBiometricUnlock() {
        try? engine.removeKeySlot(slot: DeviceKeychain.slotName)
        DeviceKeychain.remove()
    }

    func lock() {
        engine.lock()
        accounts = []
        codes = [:]
        categories = []
        state = .locked
    }

    func resetVault() {
        engine.lock()
        try? FileManager.default.removeItem(atPath: vaultPath)
        DeviceKeychain.remove()
        accounts = []
        codes = [:]
        categories = []
        state = .onboarding
    }

    func addFromUri(_ uri: String) -> Bool {
        do {
            _ = try engine.addAccountUri(uriValue: uri)
            refreshAll()
            return true
        } catch {
            return false
        }
    }

    func addManual(issuer: String, name: String, secret: String) -> Bool {
        do {
            _ = try engine.addAccountManual(issuer: issuer, name: name, secret: secret)
            refreshAll()
            return true
        } catch {
            return false
        }
    }

    func updateAccount(id: String, issuer: String, name: String, categoryId: String?, pinned: Bool) {
        try? engine.updateAccount(id: id, issuer: issuer, name: name, categoryId: categoryId, pinned: pinned)
        refreshAll()
    }

    func deleteAccount(id: String) {
        try? engine.deleteAccount(id: id)
        refreshAll()
    }

    func advanceCounter(id: String) {
        _ = try? engine.advanceCounter(id: id)
        refreshCodes()
    }

    func accountUri(id: String) -> String? {
        try? engine.accountUri(id: id)
    }

    func addCategory(name: String) {
        _ = try? engine.addCategory(name: name)
        refreshAll()
    }

    func renameCategory(id: String, name: String) {
        try? engine.renameCategory(id: id, name: name)
        refreshAll()
    }

    func deleteCategory(id: String) {
        try? engine.deleteCategory(id: id)
        refreshAll()
    }

    func changePassword(current: String, new: String) -> Bool {
        do {
            try engine.changePassword(current: current, new: new)
            disableBiometricUnlock()
            return true
        } catch {
            return false
        }
    }

    func exportBackup(password: String) -> Data? {
        try? engine.exportBackup(password: password)
    }

    enum ImportOutcome {
        case done(added: UInt32, skipped: UInt32)
        case passwordRequired
        case failed
    }

    func importData(_ data: Data, password: String?) -> ImportOutcome {
        do {
            let summary = try engine.importData(data: data, password: password)
            refreshAll()
            return .done(added: summary.addedAccounts, skipped: summary.skipped)
        } catch LiAuthError.PasswordRequired {
            return .passwordRequired
        } catch LiAuthError.WrongPassword {
            return password == nil ? .passwordRequired : .failed
        } catch {
            return .failed
        }
    }

    func syncStartReceiver() -> SyncSession? {
        try? engine.syncStartReceiver(deviceName: Self.deviceName())
    }

    func syncPollReceiver() -> SyncReceiveStatus {
        (try? engine.syncPollReceiver()) ?? .waiting
    }

    func syncStopReceiver() {
        engine.syncStopReceiver()
    }

    enum SyncSendResult {
        case sent
        case wrongCode
        case failed
    }

    func syncDiscover() async -> [SyncPeerView] {
        let engine = self.engine
        return await Task.detached(priority: .userInitiated) {
            (try? engine.syncDiscover(timeoutMs: 10000)) ?? []
        }.value
    }

    func syncSend(addresses: [String], port: UInt16, code: String) async -> SyncSendResult {
        let engine = self.engine
        return await Task.detached(priority: .userInitiated) {
            do {
                try engine.syncSend(addresses: addresses, port: port, code: code)
                return SyncSendResult.sent
            } catch LiAuthError.WrongPassword {
                return SyncSendResult.wrongCode
            } catch {
                return SyncSendResult.failed
            }
        }.value
    }

    // MARK: Search, trash, notes, batch operations

    func searchAccounts(_ query: String) -> [SearchResultView] {
        (try? engine.searchAccounts(query: query)) ?? []
    }

    func trashedAccounts() -> [TrashedAccountView] {
        (try? engine.trashedAccounts()) ?? []
    }

    func restoreAccount(id: String) {
        try? engine.restoreAccount(id: id)
        refreshAll()
    }

    func purgeAccount(id: String) {
        try? engine.purgeAccount(id: id)
        refreshAll()
    }

    func updateNotes(id: String, notes: String, recoveryCodes: [String]) {
        try? engine.updateAccountNotes(id: id, notes: notes, recoveryCodes: recoveryCodes)
        refreshAll()
    }

    @discardableResult
    func deleteAccounts(ids: [String]) -> UInt32 {
        let trashed = (try? engine.deleteAccounts(ids: ids)) ?? 0
        refreshAll()
        return trashed
    }

    @discardableResult
    func moveAccounts(ids: [String], categoryId: String?) -> UInt32 {
        let moved = (try? engine.setAccountsCategory(ids: ids, categoryId: categoryId)) ?? 0
        refreshAll()
        return moved
    }

    func addManualAdvanced(
        issuer: String,
        name: String,
        secret: String,
        algorithm: String,
        digits: UInt32,
        period: UInt32
    ) -> Bool {
        do {
            _ = try engine.addAccountManualAdvanced(
                issuer: issuer, name: name, secret: secret,
                algorithm: algorithm, digits: digits, period: period
            )
            refreshAll()
            return true
        } catch {
            return false
        }
    }

    // MARK: Time drift, auto backup, WebDAV

    func syncTimeDrift() async -> Int64? {
        let engine = self.engine
        return await Task.detached(priority: .userInitiated) {
            try? engine.syncTimeDrift()
        }.value
    }

    func timeDriftSeconds() -> Int64 {
        engine.timeDriftSeconds()
    }

    func autoBackupEnabled() -> Bool {
        ((try? engine.autoBackupDir()) ?? nil) != nil
    }

    func setAutoBackup(enabled: Bool) {
        let directory = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?
            .appendingPathComponent("LiAuth Backups", isDirectory: true)
        let path = enabled ? directory?.path : nil
        try? engine.setAutoBackupDir(dir: path)
    }

    func webdavIsConfigured() -> Bool {
        engine.webdavIsConfigured()
    }

    func webdavConfigure(url: String, username: String, password: String, backupPassword: String) async -> Bool {
        let engine = self.engine
        return await Task.detached(priority: .userInitiated) {
            do {
                try engine.webdavConfigure(
                    url: url, username: username,
                    password: password, backupPassword: backupPassword
                )
                return true
            } catch {
                return false
            }
        }.value
    }

    func webdavSyncNow() async -> Bool {
        let engine = self.engine
        return await Task.detached(priority: .userInitiated) {
            (try? engine.webdavSyncNow()) != nil
        }.value
    }

    func getSetting(_ key: String) -> String? {
        (try? engine.getSetting(key: key)) ?? nil
    }

    func setSetting(_ key: String, _ value: String) {
        try? engine.setSetting(key: key, value: value)
    }

    func refreshAll() {
        accounts = (try? engine.accounts()) ?? []
        categories = (try? engine.categories()) ?? []
        refreshCodes()
    }

    func refreshCodes() {
        guard state == .unlocked else { return }
        let list = (try? engine.codes()) ?? []
        codes = Dictionary(uniqueKeysWithValues: list.map { ($0.id, $0) })
    }

    private static func deviceName() -> String {
        #if os(macOS)
        return Host.current().localizedName ?? "Mac"
        #else
        return UIDevice.current.name
        #endif
    }
}

#if os(iOS)
import UIKit
#endif
