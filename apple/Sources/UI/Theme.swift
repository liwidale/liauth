import SwiftUI

/// Vercel-style design tokens. Dark values follow the reference palette
/// exactly; light values are the equivalent tones.
enum Palette {
    static let background = Color.adaptive(light: 0xFFFFFF, dark: 0x000000)
    static let surface = Color.adaptive(light: 0xFFFFFF, dark: 0x090909)
    static let surfaceRaised = Color.adaptive(light: 0xFAFAFA, dark: 0x111111)
    static let border = Color.adaptive(light: 0xEAEAEA, dark: 0x262626)
    static let borderStrong = Color.adaptive(light: 0xD4D4D8, dark: 0x3F3F46)
    static let hover = Color.adaptive(light: 0xF4F4F5, dark: 0x18181B)
    static let textPrimary = Color.adaptive(light: 0x0A0A0A, dark: 0xFAFAFA)
    static let textSecondary = Color.adaptive(light: 0x52525B, dark: 0xA1A1AA)
    static let textTertiary = Color.adaptive(light: 0xA1A1AA, dark: 0x71717A)
    static let accent = Color.adaptive(light: 0x0A0A0A, dark: 0xFFFFFF)
    static let accentText = Color.adaptive(light: 0xFFFFFF, dark: 0x000000)
    static let danger = Color(hex: 0xDC2626)
    static let success = Color(hex: 0x16A34A)
    static let warning = Color(hex: 0xF59E0B)
}

/// Corner radii of the design system.
enum Radius {
    static let small: CGFloat = 4
    static let control: CGFloat = 6
    static let card: CGFloat = 8
    static let modal: CGFloat = 12
}

extension Color {
    init(hex: UInt32) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255,
            opacity: 1
        )
    }

    static func adaptive(light: UInt32, dark: UInt32) -> Color {
        #if os(macOS)
        return Color(NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            return NSColor(Color(hex: isDark ? dark : light))
        })
        #else
        return Color(UIColor { traits in
            UIColor(Color(hex: traits.userInterfaceStyle == .dark ? dark : light))
        })
        #endif
    }
}

extension Font {
    /// Inter, weights capped at 700.
    static func inter(_ size: CGFloat, weight: Font.Weight = .regular) -> Font {
        switch weight {
        case .bold, .heavy, .black:
            return .custom("Inter-Bold", size: size)
        case .semibold:
            return .custom("Inter-SemiBold", size: size)
        case .medium:
            return .custom("Inter-Medium", size: size)
        default:
            return .custom("Inter-Regular", size: size)
        }
    }

    static func interBold(_ size: CGFloat) -> Font {
        .custom("Inter-Bold", size: size)
    }

    static func code(_ size: CGFloat) -> Font {
        .custom("JetBrainsMono-SemiBold", size: size)
    }
}

/// Card: raised surface, hairline border, 8px radius, no shadow.
struct CardBackground: ViewModifier {
    func body(content: Content) -> some View {
        content
            .background(RoundedRectangle(cornerRadius: Radius.card).fill(Palette.surface))
            .overlay(RoundedRectangle(cornerRadius: Radius.card).stroke(Palette.border, lineWidth: 1))
    }
}

/// Elevated panel (lock screen, overlays): opaque modal surface, 12px radius.
struct PanelBackground: ViewModifier {
    func body(content: Content) -> some View {
        content
            .background(RoundedRectangle(cornerRadius: Radius.modal).fill(Palette.surface))
            .overlay(RoundedRectangle(cornerRadius: Radius.modal).stroke(Palette.border, lineWidth: 1))
    }
}

extension View {
    func cardStyle() -> some View {
        modifier(CardBackground())
    }

    func glassStyle() -> some View {
        modifier(PanelBackground())
    }
}

struct ToastOverlay: ViewModifier {
    @Binding var message: String?

    func body(content: Content) -> some View {
        content.overlay(alignment: .bottom) {
            if let message {
                Text(message)
                    .font(.inter(13, weight: .medium))
                    .foregroundColor(Palette.textPrimary)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 10)
                    .background(RoundedRectangle(cornerRadius: Radius.card).fill(Palette.surface))
                    .overlay(RoundedRectangle(cornerRadius: Radius.card).stroke(Palette.border, lineWidth: 1))
                    .padding(.bottom, 24)
                    .task {
                        try? await Task.sleep(nanoseconds: 2_400_000_000)
                        self.message = nil
                    }
            }
        }
    }
}

extension View {
    func toastOverlay(message: Binding<String?>) -> some View {
        modifier(ToastOverlay(message: message))
    }
}

func formatCode(_ code: String) -> String {
    switch code.count {
    case 6:
        return code.prefix(3) + " " + code.suffix(3)
    case 8:
        return code.prefix(4) + " " + code.suffix(4)
    default:
        return code
    }
}
