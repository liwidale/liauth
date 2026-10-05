import SwiftUI

@main
struct LiAuthApp: App {
    @StateObject private var store = VaultStore()
    @StateObject private var localization = Localization()
    @Environment(\.scenePhase) private var scenePhase

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(store)
                .environmentObject(localization)
                .preferredColorScheme(store.colorScheme)
                .onChange(of: scenePhase) { phase in
                    store.handleScenePhase(phase)
                }
        }
        #if os(macOS)
        .defaultSize(width: 600, height: 820)
        .windowResizability(.contentMinSize)
        #endif
    }
}

struct RootView: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization

    var body: some View {
        ZStack {
            Palette.background.ignoresSafeArea()
            switch store.state {
            case .onboarding, .locked:
                LockView()
            case .unlocked:
                HomeView()
            }

            if store.privacyShieldVisible {
                PrivacyShield()
            }
        }
        .toastOverlay(message: $store.toast)
    }
}

struct PrivacyShield: View {
    @EnvironmentObject private var localization: Localization

    var body: some View {
        ZStack {
            Rectangle()
                .fill(.ultraThinMaterial)
                .background(Palette.background.opacity(0.92))
            Text(localization.t("app.name"))
                .font(.inter(30, weight: .bold))
                .foregroundColor(Palette.textPrimary)
        }
        .ignoresSafeArea()
    }
}
