<div align="center">

# <img width="25%" height="1000" alt="text" src="branding/text.png" />


**Tus códigos. Tu dispositivo. Nada más.**

Un autenticador moderno y de código abierto para Android, iOS, macOS y Windows.
Offline primero, cifrado de extremo a extremo, construido sobre un núcleo compartido en Rust.

[![CI](https://github.com/liwidale/liauth/actions/workflows/ci.yml/badge.svg)](https://github.com/liwidale/liauth/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-white.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-Android%20%7C%20iOS%20%7C%20macOS%20%7C%20Windows-lightgrey)

[English](README.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · Español · [Français](README.fr.md) · [简体中文](README.zh.md)

</div>

---

## Funciones

- **Códigos de un solo uso** — códigos por tiempo y por contador (RFC 6238 TOTP, RFC 4226 HOTP), además de códigos de Steam. Todo se detecta automáticamente desde códigos QR y enlaces; los usuarios avanzados pueden fijar parámetros propios (8 dígitos, SHA-256/SHA-512, intervalos personalizados) al añadir códigos a mano.
- **Autocompletado del sistema en Android** — la app se registra como servicio de autofill, de modo que los códigos se ofrecen directamente en los formularios de inicio de sesión.
- **Corrección de la deriva del reloj** — un cliente SNTP integrado mide cuánto se ha desviado el reloj del dispositivo y corrige la generación de códigos sin tocar jamás el reloj del sistema.
- **Búsqueda tolerante a erratas** — la coincidencia difusa encuentra «GitHub» aunque escribas «gthub», y los caracteres coincidentes se resaltan en la lista.
- **Papelera** — las cuentas eliminadas reposan 30 días en la sección de eliminadas recientemente antes de desaparecer para siempre: un toque accidental nunca cuesta un acceso.
- **Notas y códigos de recuperación** — cada cuenta puede llevar notas libres y una lista de códigos de recuperación, cifrados junto con los secretos.
- **Bloqueo antifuerza bruta** — los intentos fallidos de desbloqueo activan una demora progresiva que sobrevive a los reinicios de la app.
- **Offline primero** — sin nube, sin servidores, sin cuentas. Todos los datos permanecen en el dispositivo.
- **Cifrado fuerte** — el almacén se sella con AES-256-GCM; la clave se deriva de tu contraseña con Argon2id. Las claves de desbloqueo del dispositivo las protegen Android Keystore, el llavero de Apple con biometría respaldada por Secure Enclave y el almacén de credenciales de Windows.
- **Desbloqueo biométrico** — Face ID en iOS, Touch ID en macOS, huella o cara en Android, desbloqueo rápido en Windows.
- **Sincronización local** — traslada cuentas entre dispositivos por tu propia Wi-Fi. Los dispositivos se emparejan con un código de 6 dígitos de un solo uso (SPAKE2) y el canal va cifrado de extremo a extremo con AES-256-GCM. Nada sale jamás de la red local.
- **Copias de seguridad cifradas** — exporta e importa archivos protegidos por contraseña, guarda automáticamente una copia cifrada en la carpeta que elijas tras cada cambio o envía las copias a tu propio Nextcloud/NAS por WebDAV: el servidor solo ve texto cifrado.
- **Importación desde otras apps** — Google Authenticator (QR de migración), Aegis (plano y cifrado), 2FAS (plano y cifrado), exportaciones de Authy, cuentas de Microsoft Authenticator vía enlaces otpauth y cualquier lista de URIs `otpauth://`.
- **Grupos y acciones en lote** — organiza las cuentas en grupos propios como Finanzas, Juegos, Redes o Desarrollo, y elimina o mueve muchas cuentas a la vez.
- **Protección de privacidad** — la app oculta su contenido en el selector de tareas y bloquea capturas y grabaciones de pantalla. Ambas cosas se pueden desactivar en Ajustes.
- **Totalmente localizable** — cada texto vive en archivos JSON. Inglés, ruso, alemán, español, francés y chino simplificado vienen de serie, el idioma del sistema se detecta automáticamente y cambia al instante sin reiniciar. Suelta un JSON nuevo en la carpeta `languages` de la app para añadir un idioma sin tocar el código.
- **Diseño de nivel Vercel** — una interfaz minimalista de estilo dashboard sobre un único sistema de diseño para todas las plataformas: fondo negro puro, bordes finísimos #262626, tipografía Inter (400–700), códigos en JetBrains Mono, radios de 4/6/8/12px, controles de 36px, transiciones de 150ms ease-out y cero sombras, degradados o cristal. Las animaciones pueden desactivarse por completo y un modo opcional de iconos de marca muestra logotipos reales de los servicios ([Simple Icons](https://simpleicons.org), CC0) sobre sus colores de marca.

## Capturas

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

## Arquitectura

Toda la lógica de negocio vive en Rust y la comparten todas las plataformas:

```
liauth
├── core/
│   ├── liauth-core      Códigos (TOTP, HOTP, Steam), URIs otpauth, búsqueda, SNTP
│   ├── liauth-crypto    Sobres AES-256-GCM, derivación de claves Argon2id, ranuras de clave
│   ├── liauth-vault     Almacén cifrado, grupos, ajustes, copias, fusión
│   ├── liauth-import    Importadores de Google Authenticator, Aegis, 2FAS, Authy
│   ├── liauth-sync      Sincronización local: descubrimiento mDNS + emparejamiento SPAKE2
│   └── liauth-ffi       Bindings uniffi para Kotlin y Swift
├── windows/             App de Windows (Rust + egui)
├── android/             App de Android (Kotlin + Jetpack Compose)
├── apple/               Apps de iOS y macOS (Swift + SwiftUI, XcodeGen)
├── localization/        Archivos JSON de idioma compartidos (en, ru, de, es, fr, zh)
├── branding/            Logotipo y configuración de identidad
└── scripts/             Generación de iconos y herramientas
```

El almacén es un único archivo cifrado. Una clave de datos aleatoria de 256 bits cifra el contenido; esa clave va envuelta en una o varias *ranuras de clave*: tu contraseña (vía Argon2id) y, opcionalmente, una clave de dispositivo guardada en el hardware seguro de la plataforma para el desbloqueo biométrico. Cambiar la contraseña vuelve a sellar el almacén y revoca todas las ranuras de dispositivo.

## Obtener las apps

Cada [versión publicada](https://github.com/liwidale/liauth/releases) incluye binarios: un `.apk` para Android, un `.dmg` para macOS, un `.ipa` para iOS y un `.exe` para Windows.

Las compilaciones de macOS y Windows no están firmadas con un certificado de desarrollador de pago, por lo que el sistema puede mostrar una advertencia la primera vez que las abras. Es lo esperable en software de código abierto sin identidad de firma comercial. Si prefieres no depender de binarios descargados, compila las apps desde el código fuente como se describe abajo: las sumas de comprobación de cada versión permiten verificar lo descargado.

## Compilar desde el código fuente

### Requisitos

- Rust 1.85+ (`rustup`)
- Android: JDK 17, Android SDK + NDK, `cargo-ndk` (`cargo install cargo-ndk`)
- iOS / macOS: Xcode 15+, [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)

### Núcleo

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

La compilación de Gradle construye el núcleo Rust para todas las ABIs de Android mediante `cargo-ndk` y genera los bindings de Kotlin automáticamente.

### iOS y macOS

```sh
bash apple/scripts/build-xcframework.sh
cd apple
xcodegen generate
open LiAuth.xcodeproj
```

Compila el esquema `LiAuth-iOS` o `LiAuth-macOS`.

## Localización

Los idiomas viven en [`localization/`](localization) como archivos JSON planos (`en.json`, `ru.json`, `de.json`, `es.json`, `fr.json`, `zh.json`). Para añadir un idioma:

1. Copia `en.json` a `<código>.json` (por ejemplo `it.json`).
2. Traduce los valores y pon en `language.name` el nombre del idioma en sí mismo.
3. Abre un pull request o deja el archivo en la carpeta `languages` de la app en tu dispositivo: aparecerá en Ajustes al instante, sin recompilar.

## Identidad visual

Todo el arte se controla con dos archivos en [`branding/`](branding), así que el proyecto entero se puede rebrandear reemplazándolos:

```
branding/logo.png        1024 x 1024, PNG, fondo transparente — la marca de la app
branding/text.png        blanco sobre PNG transparente        — el logotipo LiAuth
```

La marca alimenta los iconos del lanzador de Android, el set de iconos de Apple, el icono del ejecutable de Windows y el arte dentro de la app; el logotipo aparece en las pantallas de inicio y de bloqueo. Ambos se tiñen en tiempo de ejecución, por lo que un original blanco sobre transparente se adapta solo a los temas claro y oscuro.

Tras reemplazar un archivo, ejecuta `bash scripts/generate-icons.sh` (requiere ImageMagick) para regenerar los iconos de plataforma, o simplemente haz push: la CI lo hace automáticamente. Las rutas están declaradas en [`branding/branding.json`](branding/branding.json).

## Modelo de seguridad

- Los secretos se cifran en reposo con AES-256-GCM; las claves se derivan con Argon2id (64 MiB / 3 iteraciones en escritorio, parámetros ajustados en móviles).
- El material de claves se pone a cero en memoria tras su uso.
- La sincronización local empareja dispositivos con SPAKE2 sobre un código de 6 dígitos de un solo uso, deriva una clave de sesión vía HKDF-SHA256 y cifra cada trama con AES-256-GCM. Un código erróneo aborta la sesión; cada código funciona una vez.
- Las copias de seguridad son sobres cifrados autónomos con la misma construcción que el almacén.
- Sin telemetría, sin analíticas, sin acceso a la red salvo la sincronización local que tú mismo inicias.

¿Encontraste una vulnerabilidad? Abre un aviso de seguridad privado en GitHub en lugar de un issue público.

## Desarrollo

```sh
cargo fmt --all            # formato
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace     # 70 tests unitarios, incluidos los vectores RFC 4226/6238
```

El repositorio incluye `.editorconfig`, `rustfmt.toml` y una CI que exige formato, lints y tests en cada push.

## Enlaces

- Proyecto: [github.com/liwidale/liauth](https://github.com/liwidale/liauth)
- Desarrollador: [github.com/liwidale](https://github.com/liwidale)

## Licencia

[MIT](LICENSE) — libre para usar, modificar y distribuir.
