<div align="center">

# <img width="25%" height="1000" alt="text" src="branding/text.png" />


**Ваши коды. Ваше устройство. И ничего лишнего.**

Современный аутентификатор с открытым исходным кодом для Android, iOS, macOS и Windows.
Работает офлайн, со сквозным шифрованием, построен вокруг общего ядра на Rust.

[![CI](https://github.com/liwidale/liauth/actions/workflows/ci.yml/badge.svg)](https://github.com/liwidale/liauth/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-white.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-Android%20%7C%20iOS%20%7C%20macOS%20%7C%20Windows-lightgrey)

[English](README.md) · Русский · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [简体中文](README.zh.md)

</div>

---

## Возможности

- **Одноразовые коды** — коды по времени и по счётчику (RFC 6238 TOTP, RFC 4226 HOTP), а также коды Steam. Всё определяется автоматически из QR-кодов и ссылок; опытные пользователи могут задать свои параметры (8 знаков, SHA-256/SHA-512, свой интервал) при ручном добавлении.
- **Системное автозаполнение на Android** — приложение регистрируется как autofill-сервис, и одноразовые коды предлагаются прямо в формах входа.
- **Коррекция дрейфа часов** — встроенный SNTP-клиент измеряет, насколько «уплыли» часы устройства, и корректирует генерацию кодов, не трогая системное время.
- **Поиск с опечатками** — нечеткое сравнение найдёт «GitHub» даже по запросу «gthub», а совпавшие символы подсвечиваются в списке.
- **Корзина** — удалённые аккаунты хранятся в разделе «Недавно удалённые» 30 дней и только потом исчезают навсегда: случайное нажатие не будет стоить вам доступа.
- **Заметки и коды восстановления** — к каждому аккаунту можно добавить произвольные заметки и список кодов восстановления; они шифруются вместе с секретами.
- **Защита от перебора** — неудачные попытки разблокировки включают прогрессивно растущую задержку, которая переживает перезапуск приложения.
- **Офлайн прежде всего** — без облака, серверов и учётных записей. Все данные остаются на устройстве.
- **Сильное шифрование** — хранилище запечатано AES-256-GCM; ключ выводится из вашего пароля через Argon2id. Ключи разблокировки устройства защищены Android Keystore, Связкой ключей Apple с биометрией на базе Secure Enclave и хранилищем учётных данных Windows.
- **Биометрическая разблокировка** — Face ID на iOS, Touch ID на macOS, отпечаток или лицо на Android, быстрая разблокировка на Windows.
- **Локальная синхронизация** — переносите аккаунты между устройствами по собственному Wi-Fi. Устройства сопрягаются одноразовым 6-значным кодом (SPAKE2), канал зашифрован сквозным AES-256-GCM. Ничего не покидает локальную сеть.
- **Зашифрованные резервные копии** — экспортируйте и импортируйте файлы копий под паролем, храните автоматическую зашифрованную копию в выбранной папке после каждого изменения или отправляйте копии на собственный Nextcloud/NAS по WebDAV — сервер видит только шифртекст.
- **Импорт из других приложений** — Google Authenticator (миграционный QR), Aegis (открытый и зашифрованный), 2FAS (открытый и зашифрованный), экспорт Authy, аккаунты Microsoft Authenticator по otpauth-ссылкам и любой список URI `otpauth://`.
- **Группы и массовые операции** — распределяйте аккаунты по своим группам (Финансы, Игры, Соцсети, Разработка) и удаляйте или перемещайте сразу несколько аккаунтов.
- **Защита приватности** — приложение скрывает содержимое в переключателе задач и блокирует скриншоты и запись экрана. Обе защиты можно отключить в настройках.
- **Полная локализация** — все строки лежат в JSON-файлах. По умолчанию доступны английский, русский, немецкий, испанский, французский и упрощённый китайский; язык системы определяется автоматически и переключается мгновенно без перезапуска. Положите новый JSON в папку `languages` приложения — и язык появится без правки кода.
- **Дизайн уровня Vercel** — минималистичный интерфейс в стиле дашборда на единой дизайн-системе для всех платформ: чисто чёрный фон, волосяные границы #262626, типографика Inter (400–700), коды в JetBrains Mono, радиусы 4/6/8/12px, контролы 36px, переходы 150ms ease-out и ноль теней, градиентов и «стекла». Анимации полностью отключаемы, а опция фирменных иконок показывает настоящие логотипы сервисов ([Simple Icons](https://simpleicons.org), CC0) на фирменных цветах.

## Скриншоты

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

## Архитектура

Вся бизнес-логика написана на Rust и общая для всех платформ:

```
liauth
├── core/
│   ├── liauth-core      Коды (TOTP, HOTP, Steam), otpauth-URI, поиск, SNTP
│   ├── liauth-crypto    Конверты AES-256-GCM, вывод ключей Argon2id, слоты ключей
│   ├── liauth-vault     Зашифрованное хранилище, группы, настройки, копии, слияние
│   ├── liauth-import    Импортеры Google Authenticator, Aegis, 2FAS, Authy
│   ├── liauth-sync      Локальная синхронизация: обнаружение mDNS + сопряжение SPAKE2
│   └── liauth-ffi       uniffi-биндинги для Kotlin и Swift
├── windows/             Приложение для Windows (Rust + egui)
├── android/             Приложение для Android (Kotlin + Jetpack Compose)
├── apple/               Приложения для iOS и macOS (Swift + SwiftUI, XcodeGen)
├── localization/        Общие JSON-файлы языков (en, ru, de, es, fr, zh)
├── branding/            Логотип и конфигурация айдентики
└── scripts/             Генерация иконок и инструменты
```

Хранилище — один зашифрованный файл. Случайный 256-битный ключ данных шифрует содержимое; сам он завёрнут в один или несколько *слотов ключей*: ваш пароль (через Argon2id) и, опционально, ключ устройства в защищённом железе платформы для биометрической разблокировки. Смена пароля перезапечатывает хранилище и отзывает все слоты устройства.

## Как получить приложения

Готовые сборки прикреплены к каждому [релизу](https://github.com/liwidale/liauth/releases): `.apk` для Android, `.dmg` для macOS, `.ipa` для iOS и `.exe` для Windows.

Сборки для macOS и Windows не подписаны платным сертификатом разработчика, поэтому система может показать предупреждение при первом запуске. Для open-source-программ без коммерческой подписи это ожидаемо. Если не хотите полагаться на скачанные бинарники — соберите приложения из исходников по инструкции ниже; контрольные суммы релиза позволяют проверить скачанное.

## Сборка из исходников

### Требования

- Rust 1.85+ (`rustup`)
- Android: JDK 17, Android SDK + NDK, `cargo-ndk` (`cargo install cargo-ndk`)
- iOS / macOS: Xcode 15+, [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)

### Ядро

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

Gradle-сборка компилирует Rust-ядро под все ABI Android через `cargo-ndk` и автоматически генерирует Kotlin-биндинги.

### iOS и macOS

```sh
bash apple/scripts/build-xcframework.sh
cd apple
xcodegen generate
open LiAuth.xcodeproj
```

Соберите схему `LiAuth-iOS` или `LiAuth-macOS`.

## Локализация

Языки лежат в [`localization/`](localization) плоскими JSON-файлами (`en.json`, `ru.json`, `de.json`, `es.json`, `fr.json`, `zh.json`). Чтобы добавить язык:

1. Скопируйте `en.json` в `<код>.json` (например, `it.json`).
2. Переведите значения и укажите в `language.name` название языка на нём самом.
3. Откройте pull request или положите файл в папку `languages` приложения на устройстве — язык сразу появится в настройках, пересборка не нужна.

## Айдентика

Вся графика управляется двумя файлами в [`branding/`](branding), поэтому проект можно полностью ребрендировать их заменой:

```
branding/logo.png        1024 x 1024, PNG, прозрачный фон — знак приложения
branding/text.png        белый на прозрачном PNG            — логотип-надпись LiAuth
```

Знак используется для иконок Android, набора иконок Apple, иконки exe в Windows и графики внутри приложения; надпись показывается на главном экране и экране блокировки. Оба тонируются в рантайме, поэтому белый на прозрачном источник сам подстраивается под светлую и тёмную темы.

После замены файла запустите `bash scripts/generate-icons.sh` (нужен ImageMagick), чтобы перегенерировать иконки платформ, или просто сделайте push — CI сделает это автоматически. Пути объявлены в [`branding/branding.json`](branding/branding.json).

## Модель безопасности

- Секреты шифруются в покое AES-256-GCM; ключи выводятся Argon2id (64 МиБ / 3 итерации на десктопе, мобильные параметры на телефонах).
- Ключевой материал зануляется в памяти после использования.
- Локальная синхронизация сопрягает устройства по SPAKE2 через одноразовый 6-значный код, выводит сеансовый ключ через HKDF-SHA256 и шифрует каждый кадр AES-256-GCM. Неверный код обрывает сессию; каждый код работает один раз.
- Резервные копии — самодостаточные зашифрованные конверты той же конструкции, что и хранилище.
- Ни телеметрии, ни аналитики, ни доступа в сеть, кроме локальной синхронизации, которую вы запускаете сами.

Нашли уязвимость? Пожалуйста, откройте приватный security advisory на GitHub, а не публичный issue.

## Разработка

```sh
cargo fmt --all            # форматирование
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace     # 70 юнит-тестов, включая векторы RFC 4226/6238
```

В репозитории есть `.editorconfig`, `rustfmt.toml` и CI, который проверяет форматирование, линты и тесты на каждый push.

## Ссылки

- Проект: [github.com/liwidale/liauth](https://github.com/liwidale/liauth)
- Разработчик: [github.com/liwidale](https://github.com/liwidale)

## Лицензия

[MIT](LICENSE) — свободно используйте, изменяйте и распространяйте.
