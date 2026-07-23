import SwiftUI

struct SyncView: View {
    @EnvironmentObject private var store: VaultStore
    @EnvironmentObject private var localization: Localization
    @Environment(\.dismiss) private var dismiss

    private enum Mode {
        case menu
        case receiving(SyncSession)
        case discovering
        case peerList([SyncPeerView])
        case enterCode(SyncPeerView)
        case sending
        case done(String)
        case failed(String)
    }

    @State private var mode: Mode = .menu
    @State private var code = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text(localization.t("sync.title"))
                    .font(.interBold(20))
                    .foregroundColor(Palette.textPrimary)
                Spacer()
                Button(localization.t("action.close")) {
                    store.syncStopReceiver()
                    dismiss()
                }
                .font(.inter(14))
                .foregroundColor(Palette.textSecondary)
                .buttonStyle(.plain)
            }

            content
                .padding(.top, 16)

            Spacer(minLength: 0)
        }
        .padding(26)
        .background(Palette.background)
        .frame(minWidth: 380, minHeight: 420)
        .task {
            while !Task.isCancelled {
                if case .receiving = mode {
                    switch store.syncPollReceiver() {
                    case .completed(let added, _, let skipped):
                        store.refreshAll()
                        mode = .done(localization.tf(
                            "sync.received",
                            ["added": String(added), "skipped": String(skipped)]
                        ))
                    case .failed:
                        mode = .failed(localization.t("sync.failed"))
                    case .waiting:
                        break
                    }
                }
                try? await Task.sleep(nanoseconds: 400_000_000)
            }
        }
        .onDisappear { store.syncStopReceiver() }
    }

    @ViewBuilder
    private var content: some View {
        switch mode {
        case .menu:
            VStack(alignment: .leading, spacing: 10) {
                Text(localization.t("sync.subtitle"))
                    .font(.inter(13.5))
                    .foregroundColor(Palette.textSecondary)
                PrimaryButton(title: localization.t("sync.receive")) {
                    if let session = store.syncStartReceiver() {
                        mode = .receiving(session)
                    } else {
                        mode = .failed(localization.t("sync.failed"))
                    }
                }
                .padding(.top, 8)
                SecondaryButton(title: localization.t("sync.send")) {
                    mode = .discovering
                    Task {
                        let peers = await store.syncDiscover()
                        mode = .peerList(peers)
                    }
                }
                Text(localization.t("sync.hint"))
                    .font(.inter(12))
                    .foregroundColor(Palette.textTertiary)
            }

        case .receiving(let session):
            VStack(spacing: 20) {
                Text(localization.t("sync.receiveTitle"))
                    .font(.inter(13.5))
                    .foregroundColor(Palette.textSecondary)
                Text("\(session.code.prefix(3)) \(session.code.suffix(3))")
                    .font(.code(42))
                    .foregroundColor(Palette.textPrimary)
                    .frame(maxWidth: .infinity)
                Text(localization.t("sync.receiveHint"))
                    .font(.inter(12))
                    .foregroundColor(Palette.textTertiary)
            }

        case .discovering:
            progress(localization.t("sync.searching"))

        case .peerList(let peers):
            if peers.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    Text(localization.t("sync.noDevices"))
                        .font(.inter(16, weight: .semibold))
                        .foregroundColor(Palette.textPrimary)
                    Text(localization.t("sync.noDevicesHint"))
                        .font(.inter(12.5))
                        .foregroundColor(Palette.textTertiary)
                    SecondaryButton(title: localization.t("action.back")) { mode = .menu }
                        .padding(.top, 10)
                }
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    Text(localization.t("sync.selectDevice"))
                        .font(.inter(13.5))
                        .foregroundColor(Palette.textSecondary)
                    ForEach(peers, id: \.name) { peer in
                        Button {
                            code = ""
                            mode = .enterCode(peer)
                        } label: {
                            HStack(spacing: 12) {
                                AvatarView(title: peer.name, size: 36)
                                Text(peer.name)
                                    .font(.inter(15, weight: .semibold))
                                    .foregroundColor(Palette.textPrimary)
                                Spacer()
                            }
                            .padding(14)
                            .cardStyle()
                        }
                        .buttonStyle(.plain)
                    }
                }
            }

        case .enterCode(let peer):
            VStack(alignment: .leading, spacing: 12) {
                Text(localization.tf("sync.enterCode", ["name": peer.name]))
                    .font(.inter(13.5))
                    .foregroundColor(Palette.textSecondary)
                LiAuthField(hint: "000 000", text: Binding(
                    get: { code },
                    set: { code = String($0.filter(\.isNumber).prefix(6)) }
                ))
                PrimaryButton(title: localization.t("sync.sendNow"), enabled: code.count == 6) {
                    mode = .sending
                    Task {
                        let result = await store.syncSend(addresses: peer.addresses, port: peer.port, code: code)
                        switch result {
                        case .sent:
                            mode = .done(localization.t("sync.sent"))
                        case .wrongCode:
                            mode = .failed(localization.t("sync.codeRejected"))
                        case .failed:
                            mode = .failed(localization.t("sync.failed"))
                        }
                    }
                }
            }

        case .sending:
            progress(localization.t("sync.sending"))

        case .done(let message):
            VStack(spacing: 10) {
                Image(systemName: "checkmark")
                    .font(.system(size: 30, weight: .semibold))
                    .foregroundColor(Palette.textPrimary)
                Text(message)
                    .font(.inter(13.5))
                    .foregroundColor(Palette.textSecondary)
                    .multilineTextAlignment(.center)
                PrimaryButton(title: localization.t("action.done")) { dismiss() }
                    .padding(.top, 10)
            }
            .frame(maxWidth: .infinity)
            .padding(.top, 24)

        case .failed(let message):
            VStack(spacing: 8) {
                Text(localization.t("sync.failedTitle"))
                    .font(.inter(16, weight: .semibold))
                    .foregroundColor(Palette.textPrimary)
                Text(message)
                    .font(.inter(12.5))
                    .foregroundColor(Palette.textTertiary)
                SecondaryButton(title: localization.t("action.back")) { mode = .menu }
                    .padding(.top, 10)
            }
            .frame(maxWidth: .infinity)
            .padding(.top, 24)
        }
    }

    private func progress(_ label: String) -> some View {
        VStack(spacing: 12) {
            Text(label)
                .font(.inter(13.5))
                .foregroundColor(Palette.textSecondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.top, 32)
    }
}
