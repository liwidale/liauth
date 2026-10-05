import SwiftUI

#if os(macOS)
import AppKit
#else
import UIKit
#endif

struct PrimaryButton: View {
    let title: String
    var enabled: Bool = true
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.inter(14, weight: .semibold))
                .foregroundColor(enabled ? Palette.accentText : Palette.textTertiary)
                .frame(maxWidth: .infinity)
                .frame(height: 40)
                .background(
                    RoundedRectangle(cornerRadius: Radius.control)
                        .fill(enabled ? Palette.accent : Palette.surfaceRaised)
                )
        }
        .buttonStyle(.plain)
        .disabled(!enabled)
    }
}

struct SecondaryButton: View {
    let title: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.inter(14, weight: .medium))
                .foregroundColor(Palette.textPrimary)
                .frame(maxWidth: .infinity)
                .frame(height: 40)
                .background(RoundedRectangle(cornerRadius: Radius.control).fill(Palette.surfaceRaised))
                .overlay(RoundedRectangle(cornerRadius: Radius.control).stroke(Palette.border, lineWidth: 1))
        }
        .buttonStyle(.plain)
    }
}

struct LiAuthField: View {
    let hint: String
    @Binding var text: String
    var secure: Bool = false

    @State private var revealed = false
    @FocusState private var focused: Bool

    var body: some View {
        HStack(spacing: 8) {
            Group {
                if secure && !revealed {
                    SecureField(hint, text: $text)
                } else {
                    TextField(hint, text: $text)
                }
            }
            .font(.inter(15))
            .textFieldStyle(.plain)
            .foregroundColor(Palette.textPrimary)
            #if os(iOS)
            .autocorrectionDisabled()
            .textInputAutocapitalization(.never)
            #endif

            if secure {
                Button {
                    revealed.toggle()
                } label: {
                    Image(systemName: revealed ? "eye.slash" : "eye")
                        .font(.system(size: 15))
                        .foregroundColor(Palette.textSecondary)
                }
                .buttonStyle(.plain)
                .accessibilityLabel(revealed ? "Hide" : "Show")
            }
        }
        .padding(.horizontal, 12)
        .frame(height: 40)
        .background(RoundedRectangle(cornerRadius: Radius.control).fill(Palette.surface))
        .overlay(
            RoundedRectangle(cornerRadius: Radius.control)
                .stroke(focused ? Palette.textPrimary : Palette.border, lineWidth: 1)
        )
        .focused($focused)
    }
}

struct SectionLabel: View {
    let text: String

    var body: some View {
        Text(text.uppercased())
            .font(.inter(12, weight: .medium))
            .kerning(0.8)
            .foregroundColor(Palette.textTertiary)
    }
}

struct CountdownBar: View {
    let fraction: Double

    var body: some View {
        GeometryReader { geometry in
            ZStack(alignment: .leading) {
                Rectangle().fill(Palette.border)
                Rectangle()
                    .fill(fraction < 0.2 ? Palette.warning : Palette.textPrimary)
                    .frame(width: geometry.size.width * max(0, min(1, fraction)))
            }
        }
        .frame(height: 2)
    }
}

/// Primary brand colors for well-known services, keyed by normalized issuer.
enum BrandColors {
    private static let colors: [String: (Double, Double, Double)] = [
        "github": (24, 23, 23), "gitlab": (252, 109, 38), "google": (66, 133, 244),
        "gmail": (66, 133, 244), "youtube": (255, 0, 0), "microsoft": (0, 120, 212),
        "outlook": (0, 120, 212), "azure": (0, 120, 212), "apple": (10, 132, 255),
        "icloud": (10, 132, 255), "amazon": (255, 153, 0), "aws": (255, 153, 0),
        "facebook": (24, 119, 242), "meta": (24, 119, 242), "instagram": (225, 48, 108),
        "x": (29, 155, 240), "twitter": (29, 155, 240), "discord": (88, 101, 242),
        "slack": (74, 21, 75), "dropbox": (0, 97, 255), "steam": (27, 40, 56),
        "epicgames": (44, 44, 44), "riotgames": (235, 0, 20), "twitch": (145, 70, 255),
        "reddit": (255, 69, 0), "linkedin": (10, 102, 194), "paypal": (0, 48, 135),
        "stripe": (99, 91, 255), "coinbase": (0, 82, 255), "binance": (240, 185, 11),
        "kraken": (87, 41, 206), "protonmail": (109, 74, 255), "proton": (109, 74, 255),
        "bitwarden": (23, 93, 220), "nextcloud": (0, 130, 201), "cloudflare": (243, 128, 32),
        "digitalocean": (0, 105, 255), "npm": (203, 56, 55), "docker": (36, 150, 237),
        "atlassian": (0, 82, 204), "jira": (0, 82, 204), "bitbucket": (0, 82, 204),
        "notion": (0, 0, 0), "figma": (162, 89, 255), "telegram": (36, 161, 222),
        "whatsapp": (37, 211, 102), "signal": (58, 118, 240), "wordpress": (33, 117, 155),
        "shopify": (95, 190, 66), "ebay": (230, 50, 56), "netflix": (229, 9, 20),
        "spotify": (30, 215, 96), "nintendo": (230, 0, 18), "playstation": (0, 67, 156),
        "xbox": (16, 124, 16), "yandex": (255, 204, 0), "vk": (0, 119, 255),
        "mailru": (0, 95, 249), "heroku": (67, 0, 152), "netlify": (0, 173, 181),
        "vercel": (0, 0, 0), "lastpass": (213, 43, 30), "1password": (10, 132, 255),
    ]

    static func forIssuer(_ issuer: String) -> Color? {
        let normalized = issuer.lowercased().filter { $0.isLetter || $0.isNumber }
        guard let rgb = colors[String(normalized)] else { return nil }
        return Color(red: rgb.0 / 255, green: rgb.1 / 255, blue: rgb.2 / 255)
    }

    static func isLight(_ issuer: String) -> Bool {
        let normalized = issuer.lowercased().filter { $0.isLetter || $0.isNumber }
        guard let rgb = colors[String(normalized)] else { return false }
        return 0.299 * rgb.0 + 0.587 * rgb.1 + 0.114 * rgb.2 > 150
    }

    /// Simple Icons slugs bundled under icons/ (white 96x96 PNGs, CC0).
    private static let iconSlugs: Set<String> = [
        "github", "gitlab", "google", "gmail", "youtube", "apple", "icloud",
        "facebook", "meta", "instagram", "x", "discord", "dropbox", "steam",
        "epicgames", "riotgames", "twitch", "reddit", "paypal", "stripe",
        "coinbase", "binance", "protonmail", "proton", "bitwarden", "nextcloud",
        "cloudflare", "digitalocean", "npm", "docker", "atlassian", "jira",
        "bitbucket", "notion", "figma", "telegram", "whatsapp", "signal",
        "wordpress", "shopify", "ebay", "netflix", "spotify", "playstation",
        "vk", "maildotru", "netlify", "vercel", "lastpass", "1password",
    ]

    static func iconSlug(_ issuer: String) -> String? {
        let normalized = String(issuer.lowercased().filter { $0.isLetter || $0.isNumber })
        let slug: String
        switch normalized {
        case "twitter": slug = "x"
        case "mailru", "mail": slug = "maildotru"
        default: slug = normalized
        }
        return iconSlugs.contains(slug) ? slug : nil
    }

    static func icon(_ issuer: String) -> Image? {
        guard let slug = iconSlug(issuer),
              let url = Bundle.main.url(forResource: slug, withExtension: "png", subdirectory: "icons")
        else {
            return nil
        }
        #if os(macOS)
        guard let image = NSImage(contentsOf: url) else { return nil }
        return Image(nsImage: image)
        #else
        guard let data = try? Data(contentsOf: url), let image = UIImage(data: data) else { return nil }
        return Image(uiImage: image)
        #endif
    }
}

struct AvatarView: View {
    let title: String
    var size: CGFloat = 42
    var branded: Bool = false

    var body: some View {
        let brand = branded ? BrandColors.forIssuer(title) : nil
        let logo = branded ? BrandColors.icon(title) : nil
        ZStack {
            if let logo {
                // Real Simple Icons logo (white glyph) on the brand background.
                RoundedRectangle(cornerRadius: Radius.control)
                    .fill(brand ?? Color(red: 0.11, green: 0.11, blue: 0.11))
                logo
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                    .frame(width: size * 0.6, height: size * 0.6)
            } else {
                RoundedRectangle(cornerRadius: Radius.control)
                    .fill(brand ?? Palette.surfaceRaised)
                    .overlay(
                        RoundedRectangle(cornerRadius: Radius.control)
                            .stroke(brand ?? Palette.border, lineWidth: 1)
                    )
                Text(title.prefix(1).uppercased())
                    .font(.inter(size * 0.4, weight: .semibold))
                    .foregroundColor(
                        brand == nil
                            ? Palette.textPrimary
                            : (BrandColors.isLight(title) ? .black : .white)
                    )
            }
        }
        .frame(width: size, height: size)
    }
}

/// Text with the characters at `indices` (char positions from the fuzzy
/// matcher) rendered inverted, used to highlight search hits.
struct HighlightedText: View {
    let text: String
    let indices: [UInt32]
    let font: Font
    let color: Color

    var body: some View {
        if indices.isEmpty {
            Text(text).font(font).foregroundColor(color)
        } else {
            Text(attributed).font(font)
        }
    }

    private var attributed: AttributedString {
        var result = AttributedString()
        let matched = Set(indices.map { Int($0) })
        for (i, character) in text.enumerated() {
            var piece = AttributedString(String(character))
            if matched.contains(i) {
                piece.backgroundColor = Palette.accent
                piece.foregroundColor = Palette.accentText
            } else {
                piece.foregroundColor = color
            }
            result += piece
        }
        return result
    }
}

struct LogoMark: View {
    var size: CGFloat

    var body: some View {
        if let image = loadBundledImage(named: "logo") {
            image
                .renderingMode(.template)
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: size, height: size)
                .foregroundColor(Palette.textPrimary)
        } else {
            Rectangle()
                .stroke(Palette.borderStrong, lineWidth: 1.5)
                .frame(width: size * 0.85, height: size * 0.85)
        }
    }
}

func loadBundledImage(named name: String) -> Image? {
    guard let url = Bundle.main.url(forResource: name, withExtension: "png") else { return nil }
    #if os(macOS)
    guard let image = NSImage(contentsOf: url) else { return nil }
    return Image(nsImage: image)
    #else
    guard let data = try? Data(contentsOf: url), let image = UIImage(data: data) else { return nil }
    return Image(uiImage: image)
    #endif
}

struct Wordmark: View {
    var height: CGFloat

    var body: some View {
        if let image = Self.load() {
            image
                .renderingMode(.template)
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(height: height)
                .foregroundColor(Palette.textPrimary)
                .accessibilityLabel("LiAuth")
        } else {
            Text("LiAuth")
                .font(.interBold(height * 0.9))
                .foregroundColor(Palette.textPrimary)
        }
    }

    private static func load() -> Image? {
        loadBundledImage(named: "text")
    }
}

struct PinMarker: View {
    var body: some View {
        Circle()
            .fill(Palette.textSecondary)
            .frame(width: 6, height: 6)
    }
}

struct Chip: View {
    let text: String
    let selected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(text)
                .font(.inter(13, weight: .medium))
                .foregroundColor(selected ? Palette.accentText : Palette.textSecondary)
                .padding(.horizontal, 12)
                .padding(.vertical, 6)
                .background(
                    RoundedRectangle(cornerRadius: Radius.control)
                        .fill(selected ? Palette.accent : Palette.surfaceRaised)
                )
                .overlay(
                    RoundedRectangle(cornerRadius: Radius.control)
                        .stroke(selected ? Color.clear : Palette.border, lineWidth: 1)
                )
        }
        .buttonStyle(.plain)
    }
}
