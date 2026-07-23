<div align="center">

# <img width="25%" height="1000" alt="text" src="branding/text.png" />


**Vos codes. Votre appareil. Rien d'autre.**

Un authentificateur moderne et open source pour Android, iOS, macOS et Windows.
Hors ligne d'abord, chiffré de bout en bout, bâti autour d'un cœur Rust partagé.

[![CI](https://github.com/liwidale/liauth/actions/workflows/ci.yml/badge.svg)](https://github.com/liwidale/liauth/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-white.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-Android%20%7C%20iOS%20%7C%20macOS%20%7C%20Windows-lightgrey)

[English](README.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Español](README.es.md) · Français · [简体中文](README.zh.md)

</div>

---

## Fonctionnalités

- **Codes à usage unique** — codes basés sur le temps et sur compteur (RFC 6238 TOTP, RFC 4226 HOTP), plus les codes Steam. Tout est détecté automatiquement depuis les codes QR et les liens ; les utilisateurs avancés peuvent définir leurs propres paramètres (8 chiffres, SHA-256/SHA-512, intervalle personnalisé) lors d'un ajout manuel.
- **Remplissage automatique système sur Android** — l'app s'enregistre comme service d'autofill : les codes sont proposés directement dans les formulaires de connexion.
- **Correction de la dérive d'horloge** — un client SNTP intégré mesure la dérive de l'horloge de l'appareil et corrige la génération des codes sans jamais toucher à l'horloge du système.
- **Recherche tolérante aux fautes** — la correspondance floue trouve « GitHub » même si vous tapez « gthub », et les caractères trouvés sont surlignés dans la liste.
- **Corbeille** — les comptes supprimés reposent 30 jours dans la section des suppressions récentes avant de disparaître pour de bon : un geste malheureux ne coûte jamais un accès.
- **Notes et codes de récupération** — chaque compte peut porter des notes libres et une liste de codes de récupération, chiffrés avec les secrets.
- **Verrou anti-force brute** — les tentatives de déverrouillage échouées déclenchent un délai progressif qui survit aux redémarrages de l'app.
- **Hors ligne d'abord** — pas de cloud, pas de serveurs, pas de comptes. Toutes les données restent sur l'appareil.
- **Chiffrement fort** — le coffre est scellé en AES-256-GCM ; la clé est dérivée de votre mot de passe avec Argon2id. Les clés de déverrouillage de l'appareil sont protégées par Android Keystore, le trousseau Apple avec biométrie adossée à la Secure Enclave et le magasin d'identifiants de Windows.
- **Déverrouillage biométrique** — Face ID sur iOS, Touch ID sur macOS, empreinte ou visage sur Android, déverrouillage rapide sur Windows.
- **Synchronisation locale** — déplacez vos comptes entre appareils via votre propre Wi-Fi. Les appareils s'apparient avec un code à 6 chiffres à usage unique (SPAKE2) et le canal est chiffré de bout en bout en AES-256-GCM. Rien ne quitte jamais le réseau local.
- **Sauvegardes chiffrées** — exportez et importez des fichiers protégés par mot de passe, conservez automatiquement une copie chiffrée dans le dossier de votre choix après chaque modification, ou poussez les sauvegardes vers votre propre Nextcloud/NAS via WebDAV : le serveur ne voit jamais que du chiffré.
- **Import depuis d'autres apps** — Google Authenticator (QR de migration), Aegis (clair et chiffré), 2FAS (clair et chiffré), exports Authy, comptes Microsoft Authenticator via liens otpauth et toute liste d'URIs `otpauth://`.
- **Groupes et actions par lot** — organisez les comptes en groupes personnalisés comme Finances, Jeux, Réseaux ou Développement, et supprimez ou déplacez plusieurs comptes d'un coup.
- **Protection de la vie privée** — l'app masque son contenu dans le sélecteur d'apps et bloque captures et enregistrements d'écran. Les deux protections se désactivent dans les réglages.
- **Entièrement localisable** — chaque texte vit dans des fichiers JSON. Anglais, russe, allemand, espagnol, français et chinois simplifié sont fournis, la langue du système est détectée automatiquement et bascule instantanément sans redémarrage. Déposez un nouveau JSON dans le dossier `languages` de l'app pour ajouter une langue sans toucher au code.
- **Design du niveau de Vercel** — une interface minimale de type dashboard sur un système de design unique pour toutes les plateformes : fond noir pur, bordures fines #262626, typographie Inter (400–700), codes en JetBrains Mono, rayons 4/6/8/12px, contrôles de 36px, transitions 150ms ease-out et zéro ombre, dégradé ou verre. Les animations se désactivent entièrement, et un mode optionnel d'icônes de marque affiche les vrais logos des services ([Simple Icons](https://simpleicons.org), CC0) sur leurs couleurs de marque.

## Captures d'écran

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

Toute la logique métier vit en Rust et est partagée par toutes les plateformes :

```
liauth
├── core/
│   ├── liauth-core      Codes (TOTP, HOTP, Steam), URIs otpauth, recherche, SNTP
│   ├── liauth-crypto    Enveloppes AES-256-GCM, dérivation de clés Argon2id, slots de clés
│   ├── liauth-vault     Coffre chiffré, groupes, réglages, sauvegardes, fusion
│   ├── liauth-import    Importeurs Google Authenticator, Aegis, 2FAS, Authy
│   ├── liauth-sync      Synchronisation locale : découverte mDNS + appariement SPAKE2
│   └── liauth-ffi       Bindings uniffi pour Kotlin et Swift
├── windows/             App Windows (Rust + egui)
├── android/             App Android (Kotlin + Jetpack Compose)
├── apple/               Apps iOS et macOS (Swift + SwiftUI, XcodeGen)
├── localization/        Fichiers de langue JSON partagés (en, ru, de, es, fr, zh)
├── branding/            Logo et configuration d'identité
└── scripts/             Génération d'icônes et outillage
```

Le coffre est un unique fichier chiffré. Une clé de données aléatoire de 256 bits chiffre le contenu ; cette clé est enveloppée dans un ou plusieurs *slots de clés* : votre mot de passe (via Argon2id) et, en option, une clé d'appareil conservée dans le matériel sécurisé de la plateforme pour le déverrouillage biométrique. Changer le mot de passe rescelle le coffre et révoque tous les slots d'appareil.

## Obtenir les apps

Des binaires prêts à l'emploi accompagnent chaque [release](https://github.com/liwidale/liauth/releases) : un `.apk` pour Android, un `.dmg` pour macOS, un `.ipa` pour iOS et un `.exe` pour Windows.

Les builds macOS et Windows ne sont pas signés avec un certificat développeur payant : le système peut donc afficher un avertissement à la première ouverture. C'est attendu pour un logiciel open source sans identité de signature commerciale. Si vous préférez ne pas dépendre de binaires téléchargés, compilez les apps depuis les sources comme décrit ci-dessous — les sommes de contrôle des releases permettent de vérifier vos téléchargements.

## Compiler depuis les sources

### Prérequis

- Rust 1.85+ (`rustup`)
- Android : JDK 17, Android SDK + NDK, `cargo-ndk` (`cargo install cargo-ndk`)
- iOS / macOS : Xcode 15+, [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)

### Cœur

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

Le build Gradle compile le cœur Rust pour toutes les ABIs Android via `cargo-ndk` et génère automatiquement les bindings Kotlin.

### iOS et macOS

```sh
bash apple/scripts/build-xcframework.sh
cd apple
xcodegen generate
open LiAuth.xcodeproj
```

Compilez le schéma `LiAuth-iOS` ou `LiAuth-macOS`.

## Localisation

Les langues vivent dans [`localization/`](localization) sous forme de fichiers JSON plats (`en.json`, `ru.json`, `de.json`, `es.json`, `fr.json`, `zh.json`). Pour ajouter une langue :

1. Copiez `en.json` vers `<code>.json` (par exemple `it.json`).
2. Traduisez les valeurs et mettez dans `language.name` le nom de la langue dans cette langue.
3. Ouvrez une pull request, ou déposez le fichier dans le dossier `languages` de l'app sur votre appareil : il apparaît immédiatement dans les réglages, sans recompilation.

## Identité visuelle

Tout le graphisme est piloté par deux fichiers dans [`branding/`](branding) ; remplacer ces fichiers suffit à rebrander l'ensemble du projet :

```
branding/logo.png        1024 x 1024, PNG, fond transparent — la marque de l'app
branding/text.png        blanc sur PNG transparent          — le logotype LiAuth
```

La marque alimente les icônes du lanceur Android, le jeu d'icônes Apple, l'icône de l'exécutable Windows et les visuels dans l'app ; le logotype s'affiche sur les écrans d'accueil et de verrouillage. Les deux sont teintés à l'exécution : un original blanc sur transparent s'adapte de lui-même aux thèmes clair et sombre.

Après remplacement d'un fichier, lancez `bash scripts/generate-icons.sh` (nécessite ImageMagick) pour régénérer les icônes de plateforme, ou poussez simplement : la CI s'en charge automatiquement. Les chemins sont déclarés dans [`branding/branding.json`](branding/branding.json).

## Modèle de sécurité

- Les secrets sont chiffrés au repos en AES-256-GCM ; les clés sont dérivées avec Argon2id (64 Mio / 3 itérations sur desktop, paramètres adaptés sur mobile).
- Le matériel de clés est remis à zéro en mémoire après usage.
- La synchronisation locale apparie les appareils via SPAKE2 avec un code à 6 chiffres à usage unique, dérive une clé de session via HKDF-SHA256 et chiffre chaque trame en AES-256-GCM. Un mauvais code interrompt la session ; chaque code ne sert qu'une fois.
- Les sauvegardes sont des enveloppes chiffrées autonomes, de même construction que le coffre.
- Aucune télémétrie, aucune analyse, aucun accès réseau hormis la synchronisation locale que vous lancez explicitement.

Vous avez trouvé une vulnérabilité ? Merci d'ouvrir un avis de sécurité privé sur GitHub plutôt qu'une issue publique.

## Développement

```sh
cargo fmt --all            # formatage
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace     # 70 tests unitaires, dont les vecteurs RFC 4226/6238
```

Le dépôt fournit `.editorconfig`, `rustfmt.toml` et une CI qui impose formatage, lints et tests à chaque push.

## Liens

- Projet : [github.com/liwidale/liauth](https://github.com/liwidale/liauth)
- Développeur : [github.com/liwidale](https://github.com/liwidale)

## Licence

[MIT](LICENSE) — libre d'utilisation, de modification et de distribution.
