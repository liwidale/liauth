import Foundation

struct LanguagePack {
    let code: String
    let name: String
    let strings: [String: String]
}

final class Localization: ObservableObject {
    @Published private(set) var activeCode: String = "en"

    private var packs: [LanguagePack] = []
    private var fallback: [String: String] = [:]

    private static let preferenceKey = "liauth.language"

    init() {
        load()
    }

    func load() {
        var loaded: [LanguagePack] = []
        if let folder = Bundle.main.url(forResource: "localization", withExtension: nil),
           let files = try? FileManager.default.contentsOfDirectory(at: folder, includingPropertiesForKeys: nil) {
            for file in files where file.pathExtension == "json" {
                if let pack = Self.parse(code: file.deletingPathExtension().lastPathComponent, url: file) {
                    loaded.append(pack)
                }
            }
        }
        let userDir = Self.userLanguagesDirectory()
        if let files = try? FileManager.default.contentsOfDirectory(at: userDir, includingPropertiesForKeys: nil) {
            for file in files where file.pathExtension == "json" {
                if let pack = Self.parse(code: file.deletingPathExtension().lastPathComponent, url: file) {
                    if let index = loaded.firstIndex(where: { $0.code == pack.code }) {
                        loaded[index] = pack
                    } else {
                        loaded.append(pack)
                    }
                }
            }
        }
        packs = loaded
        fallback = loaded.first(where: { $0.code == "en" })?.strings ?? [:]

        let saved = UserDefaults.standard.string(forKey: Self.preferenceKey)
        let system = Locale.preferredLanguages.first?
            .components(separatedBy: CharacterSet(charactersIn: "-_")).first?
            .lowercased() ?? "en"
        let requested = saved ?? system
        if loaded.contains(where: { $0.code == requested }) {
            activeCode = requested
        } else if loaded.contains(where: { $0.code == system }) {
            activeCode = system
        } else {
            activeCode = "en"
        }
    }

    var availableLanguages: [(code: String, name: String)] {
        packs.map { ($0.code, $0.name) }
    }

    func setLanguage(_ code: String) {
        guard packs.contains(where: { $0.code == code }) else { return }
        activeCode = code
        UserDefaults.standard.set(code, forKey: Self.preferenceKey)
    }

    func t(_ key: String) -> String {
        packs.first(where: { $0.code == activeCode })?.strings[key] ?? fallback[key] ?? key
    }

    func tf(_ key: String, _ args: [String: String]) -> String {
        var value = t(key)
        for (name, replacement) in args {
            value = value.replacingOccurrences(of: "{\(name)}", with: replacement)
        }
        return value
    }

    private static func parse(code: String, url: URL) -> LanguagePack? {
        guard let data = try? Data(contentsOf: url),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return nil
        }
        let strings = object.compactMapValues { $0 as? String }
        guard !strings.isEmpty else { return nil }
        return LanguagePack(code: code, name: strings["language.name"] ?? code.uppercased(), strings: strings)
    }

    private static func userLanguagesDirectory() -> URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("LiAuth/languages", isDirectory: true)
        try? FileManager.default.createDirectory(at: base, withIntermediateDirectories: true)
        return base
    }
}
