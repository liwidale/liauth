<div align="center">

# <img width="25%" height="1000" alt="text" src="branding/text.png" />


**Your codes. Your device. Nothing else.**

A modern, open-source authenticator for Android, iOS, macOS and Windows.
Offline-first, end-to-end encrypted, built around a shared Rust core.

[![CI](https://github.com/liwidale/liauth/actions/workflows/ci.yml/badge.svg)](https://github.com/liwidale/liauth/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-white.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-Android%20%7C%20iOS%20%7C%20macOS%20%7C%20Windows-lightgrey)

English · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [简体中文](README.zh.md)

</div>

---

## Features

- **One-time codes** - time-based and counter-based codes (RFC 6238 TOTP, RFC 4226 HOTP), plus Steam codes. Everything is detected automatically from QR codes and links; power users can still set custom parameters (8 digits, SHA-256/SHA-512, custom periods) when adding codes manually.
- **System autofill on Android** - the app registers as an autofill service, so one-time codes are offered right in login forms.
- **Clock drift correction** - a built-in SNTP client measures how far the device clock has drifted and corrects code generation without ever touching the OS clock.
- **Typo-tolerant search** - fuzzy matching finds "GitHub" even when you type "gthub", and the matched characters are highlighted in the list.
- **Trash** - deleted accounts rest in a Recently Deleted section for 30 days before disappearing for good, so a mistaken tap never costs a login.
- **Notes and recovery codes** - every account can carry free-form notes and a list of recovery codes, encrypted together with the secrets.
- **Anti-brute-force lock** - failed unlock attempts trigger a progressively growing delay that survives app restarts.
- **Offline-first** - no cloud, no servers, no accounts. All data stays on the device.
- **Strong encryption** - the vault is sealed with AES-256-GCM; the key is derived from your password with Argon2id. Device unlock keys are protected by Android Keystore, the Apple Keychain with Secure Enclave-backed biometry, and the Windows credential store.
- **Biometric unlock** - Face ID on iOS, Touch ID on macOS, fingerprint or face on Android, quick unlock on Windows.
- **Local sync** - move accounts between devices over your own Wi-Fi. Devices pair with a one-time 6-digit code (SPAKE2), and the channel is end-to-end encrypted with AES-256-GCM. Nothing ever leaves the local network.
- **Encrypted backups** - export and import password-protected backup files, keep an automatic encrypted copy in a folder of your choice after every change, or push backups to your own Nextcloud/NAS over WebDAV - the server only ever sees ciphertext.
- **Import from other apps** - Google Authenticator (migration QR), Aegis (plain and encrypted), 2FAS (plain and encrypted), Authy exports, Microsoft Authenticator accounts via otpauth links, and any list of `otpauth://` URIs.
- **Groups and batch actions** - organize accounts into custom groups such as Finance, Gaming, Social Media or Development, and delete or move many accounts at once.
- **Privacy protection** - the app hides its content in the task switcher and blocks screenshots and screen recording. Both can be turned off in Settings.
- **Fully localizable** - every string lives in JSON files. English, Russian, German, Spanish, French and Simplified Chinese ship by default, the system language is detected automatically, and the language switches instantly without a restart. Drop a new JSON file into the app's `languages` folder to add a language without touching the code.
- **Vercel-grade design** - a minimal dashboard-style interface built on a single design system across all platforms: pure black background, hairline #262626 borders, Inter typography (400-700), JetBrains Mono codes, 4/6/8/12px radii, 36px controls, 150ms ease-out transitions and zero shadows, gradients or glass. Animations can be switched off entirely, and an optional brand-icons mode shows real service logos ([Simple Icons](https://simpleicons.org), CC0) on brand-colored backgrounds.

## Screenshots

### Android

<p>
  <img src="branding/screenshots/android/main.jpg" height="360" alt="Android home" />
  <img src="branding/screenshots/android/accounts.jpg" height="360" alt="Android accounts" />
  <img src="branding/screenshots/android/sync.jpg" height="360" alt="Android sync" />
  <img src="branding/screenshots/android/settings.jpg" height="360" alt="Android settings" />
</p>

### Windows

<p>
  <img src="branding/screenshots/windows/main.png" height="360" alt="Windows home" />
  <img src="branding/screenshots/windows/accounts.png" height="360" alt="Windows accounts" />
  <img src="branding/screenshots/windows/sync.png" height="360" alt="Windows sync" />
  <img src="branding/screenshots/windows/settings.png" height="360" alt="Windows settings" />
</p>

### macOS

<p>
  <img src="branding/screenshots/macos/main.png" height="320" alt="macOS home" />
  <img src="branding/screenshots/macos/accounts.png" height="320" alt="macOS accounts" />
</p>
<p>
  <img src="branding/screenshots/macos/sync.png" height="320" alt="macOS sync" />
  <img src="branding/screenshots/macos/settings.png" height="320" alt="macOS settings" />
</p>

## Architecture

All business logic lives in Rust and is shared by every platform:

```
liauth
├── core/
│   ├── liauth-core      Codes (TOTP, HOTP, Steam), otpauth URIs, search, SNTP
│   ├── liauth-crypto    AES-256-GCM envelopes, Argon2id key derivation, key slots
│   ├── liauth-vault     Encrypted vault, categories, settings, backups, merging
│   ├── liauth-import    Google Authenticator, Aegis, 2FAS, Authy importers
│   ├── liauth-sync      Local network sync: mDNS discovery + SPAKE2 pairing
│   └── liauth-ffi       uniffi bindings for Kotlin and Swift
├── windows/             Windows app (Rust + egui)
├── android/             Android app (Kotlin + Jetpack Compose)
├── apple/               iOS and macOS apps (Swift + SwiftUI, XcodeGen)
├── localization/        Shared JSON language files (en, ru, de, es, fr, zh)
├── branding/            Logo and app identity configuration
└── scripts/             Icon generation and tooling
```

The vault is a single encrypted file. A random 256-bit data key encrypts the contents; that data key is wrapped by one or more *key slots*: your password (via Argon2id) and, optionally, a device key held in the platform's secure hardware for biometric unlock. Changing the password re-seals the vault and revokes all device slots.

## Getting the apps

Prebuilt binaries are attached to every [release](https://github.com/liwidale/liauth/releases): an `.apk` for Android, a `.dmg` for macOS, an `.ipa` for iOS and an `.exe` for Windows.

The macOS and Windows builds are not signed with a paid developer certificate, so the operating system may show a warning the first time you open them. This is expected for open-source software without a commercial signing identity. If you prefer not to rely on downloaded binaries, build the apps from source as described below — the release checksums let you verify what you downloaded.

## Building from source

### Prerequisites

- Rust 1.85+ (`rustup`)
- Android: JDK 17, Android SDK + NDK, `cargo-ndk` (`cargo install cargo-ndk`)
- iOS / macOS: Xcode 15+, [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)

### Core

```sh
cargo test --workspace
```

### Windows

```sh
cargo build --release -p liauth-windows
# target/release/LiAuth.exe
```

### Android

```sh
cd android
gradle assembleRelease
# app/build/outputs/apk/release/app-release.apk
```

The Gradle build compiles the Rust core for all Android ABIs via `cargo-ndk` and generates the Kotlin bindings automatically.

### iOS and macOS

```sh
bash apple/scripts/build-xcframework.sh
cd apple
xcodegen generate
open LiAuth.xcodeproj
```

Build the `LiAuth-iOS` or `LiAuth-macOS` scheme.

## Localization

Languages live in [`localization/`](localization) as flat JSON files (`en.json`, `ru.json`, `de.json`, `es.json`, `fr.json`, `zh.json`). To add a language:

1. Copy `en.json` to `<code>.json` (for example `de.json`).
2. Translate the values and set `language.name` to the language's own name.
3. Either open a pull request, or drop the file into the app's `languages` folder on your device - it appears in Settings immediately, no rebuild needed.

## Branding

All artwork is driven by two files in [`branding/`](branding), so the whole project can be rebranded by replacing them:

```
branding/logo.png        1024 x 1024, PNG, transparent background - the app mark
branding/text.png        white on transparent PNG            - the LiAuth wordmark
```

The mark feeds the Android launcher icons, the Apple icon set, the Windows executable icon and the in-app artwork; the wordmark is shown on the home and lock screens. Both are tinted at runtime, so a white-on-transparent source adapts to the light and dark themes automatically.

After replacing a file, run `bash scripts/generate-icons.sh` (requires ImageMagick) to regenerate the platform icons, or just push - CI runs it automatically. Paths are declared in [`branding/branding.json`](branding/branding.json).

## Security model

- Secrets are encrypted at rest with AES-256-GCM; keys are derived with Argon2id (64 MiB / 3 iterations on desktop, mobile-tuned parameters on phones).
- Key material is zeroized in memory after use.
- Local sync pairs devices with SPAKE2 over a one-time 6-digit code, derives a session key via HKDF-SHA256, and encrypts every frame with AES-256-GCM. A wrong code aborts the session; each code works once.
- Backups are self-contained encrypted envelopes with the same construction as the vault.
- No telemetry, no analytics, no network access except the local-network sync you explicitly start.

Found a vulnerability? Please open a private security advisory on GitHub rather than a public issue.

## Development

```sh
cargo fmt --all            # format
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace     # 70 unit tests, including RFC 4226/6238 vectors
```

The repository ships with `.editorconfig`, `rustfmt.toml` and CI that enforces formatting, lints and tests on every push.

## Links

- Project: [github.com/liwidale/liauth](https://github.com/liwidale/liauth)
- Developer: [github.com/liwidale](https://github.com/liwidale)

## License

[MIT](LICENSE) - free to use, modify and distribute.
