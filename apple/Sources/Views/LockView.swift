import SwiftUI

struct LockView: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization

    @State private var password = ""
    @State private var confirm = ""
    @State private var error: String?
    @State private var resetWarning = false
    @State private var resetPhrase = false
    @State private var resetInput = ""
    @State private var lockoutSeconds: UInt64 = 0

    private let lockoutTimer = Timer.publish(every: 1, on: .main, in: .common).autoconnect()

    private let resetConfirmationPhrase = "delete all data"

    private var onboarding: Bool { store.state == .onboarding }

    var body: some View {
        VStack(spacing: 0) {
            Spacer()
            Wordmark(height: 34)
            Text(onboarding ? localization.t("onboarding.subtitle") : localization.t("lock.subtitle"))
                .font(.inter(15))
                .foregroundColor(Palette.textSecondary)
                .multilineTextAlignment(.center)
                .padding(.top, 6)
                .padding(.horizontal, 32)

            VStack(spacing: 10) {
                LiAuthField(hint: localization.t("field.password"), text: $password, secure: true)
                if onboarding {
                    LiAuthField(hint: localization.t("field.confirmPassword"), text: $confirm, secure: true)
                }
                if lockoutSeconds > 0 {
                    Text(localization.tf("lock.tooManyAttempts", ["seconds": String(lockoutSeconds)]))
                        .font(.inter(13))
                        .foregroundColor(Palette.textSecondary)
                } else if let error {
                    Text(error)
                        .font(.inter(13))
                        .foregroundColor(Palette.textSecondary)
                }
                PrimaryButton(
                    title: onboarding ? localization.t("onboarding.create") : localization.t("lock.unlock"),
                    enabled: ready
                ) {
                    submit()
                }
                .padding(.top, 8)

                if !onboarding && DeviceKeychain.biometryAvailable && DeviceKeychain.isEnabled {
                    Button(localization.t("settings.biometricUnlock")) {
                        Task { await biometricUnlock() }
                    }
                    .font(.inter(14))
                    .foregroundColor(Palette.textSecondary)
                    .buttonStyle(.plain)
                    .padding(.top, 4)
                }

                if !onboarding {
                    Button(localization.t("lock.forgot")) {
                        resetInput = ""
                        resetWarning = true
                    }
                    .font(.inter(14))
                    .foregroundColor(Palette.textSecondary)
                    .buttonStyle(.plain)
                    .padding(.top, 2)
                }
            }
            .frame(maxWidth: 340)
            .padding(24)
            .glassStyle()
            .padding(.top, 36)
            .padding(.horizontal, 24)

            Spacer()
            Spacer()
        }
        .task {
            if !onboarding {
                lockoutSeconds = store.lockoutRemainingSeconds()
            }
            if !onboarding && DeviceKeychain.biometryAvailable && DeviceKeychain.isEnabled {
                await biometricUnlock()
            }
        }
        .onReceive(lockoutTimer) { _ in
            if !onboarding {
                let remaining = store.lockoutRemainingSeconds()
                if remaining != lockoutSeconds {
                    lockoutSeconds = remaining
                }
            }
        }
        .alert(localization.t("reset.title"), isPresented: $resetWarning) {
            Button(localization.t("reset.yes"), role: .destructive) {
                resetPhrase = true
            }
            Button(localization.t("action.cancel"), role: .cancel) {}
        } message: {
            Text(localization.t("reset.warning"))
        }
        .alert(localization.t("reset.title"), isPresented: $resetPhrase) {
            TextField(resetConfirmationPhrase, text: $resetInput)
            Button(localization.t("reset.confirm"), role: .destructive) {
                if resetInput.trimmingCharacters(in: .whitespaces).lowercased() == resetConfirmationPhrase {
                    store.resetVault()
                    store.toast = localization.t("reset.done")
                }
            }
            Button(localization.t("action.cancel"), role: .cancel) {}
        } message: {
            Text(localization.tf("reset.typePhrase", ["phrase": resetConfirmationPhrase]))
        }
    }

    private var ready: Bool {
        onboarding ? password.count >= 4 && password == confirm : !password.isEmpty && lockoutSeconds == 0
    }

    private func submit() {
        if onboarding {
            if !store.createVault(password: password) {
                error = localization.t("error.createFailed")
            }
        } else if !store.unlock(password: password) {
            lockoutSeconds = store.lockoutRemainingSeconds()
            error = localization.t("lock.wrongPassword")
            password = ""
        }
    }

    private func biometricUnlock() async {
        guard let key = await DeviceKeychain.read(reason: localization.t("lock.biometricPrompt")) else {
            return
        }
        if !store.unlockWithDeviceKey(key) {
            error = localization.t("lock.biometricUnavailable")
        }
    }
}
