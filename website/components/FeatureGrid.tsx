import { Icon, type IconName } from "./icons";
import { Reveal } from "./Reveal";
import { Section } from "./Section";

type Feature = {
  icon: IconName;
  tag: string;
  title: string;
  body: string;
  spec: string[];
  wide?: boolean;
};

// Order matters: the cards are packed onto a 6-column grid, so each row has
// to add up to exactly 6 - either two wide cards or three narrow ones.
const FEATURES: Feature[] = [
  {
    icon: "clock",
    tag: "TOTP · HOTP",
    title: "One-time codes",
    body: "Time-based and counter-based codes, plus Steam Guard. Accounts are detected automatically from QR codes and otpauth links - you never configure algorithms, digits or periods by hand.",
    spec: ["RFC 6238", "RFC 4226", "STEAM", "QR AUTO-DETECT"],
    wide: true,
  },
  {
    icon: "cloud-off",
    tag: "OFFLINE",
    title: "Offline-first",
    body: "No cloud, no servers, no accounts. Every secret is created, stored and used on your device - nothing to sign up for, nothing to go down.",
    spec: ["NO CLOUD", "NO SERVERS", "NO ACCOUNTS"],
    wide: true,
  },
  {
    icon: "lock",
    tag: "AES-256",
    title: "Strong encryption",
    body: "The vault is sealed with AES-256-GCM under a key derived from your password with Argon2id. Device keys live in Android Keystore, the Apple Secure Enclave and the Windows credential store.",
    spec: ["AES-256-GCM", "ARGON2ID", "SECURE ENCLAVE"],
    wide: true,
  },
  {
    icon: "sync",
    tag: "SPAKE2",
    title: "Local sync",
    body: "Move accounts between devices over your own Wi-Fi. Devices pair with a one-time six-digit code, the channel is end-to-end encrypted with AES-256-GCM, and data never leaves the local network.",
    spec: ["SPAKE2", "AES-256-GCM", "LAN ONLY"],
    wide: true,
  },
  {
    icon: "archive",
    tag: "BACKUP",
    title: "Encrypted backups",
    body: "Export and import password-protected backup files - self-contained encrypted envelopes with the same construction as the vault. No cloud involved.",
    spec: ["PORTABLE", "PASSWORD-SEALED"],
    wide: true,
  },
  {
    icon: "import",
    tag: "MIGRATE",
    title: "Import",
    body: "Bring everything with you: Google Authenticator migration QR, Aegis and 2FAS (plain or encrypted), Authy exports, Microsoft Authenticator accounts and any list of otpauth:// URIs.",
    spec: ["GOOGLE AUTH", "AEGIS", "2FAS", "AUTHY", "OTPAUTH://"],
    wide: true,
  },
  {
    icon: "fingerprint",
    tag: "BIOMETRY",
    title: "Biometric unlock",
    body: "Face ID on iOS, Touch ID on macOS, fingerprint or face on Android, Windows Hello on desktop. Convenience without weakening the vault.",
    spec: ["FACE ID", "TOUCH ID", "WINDOWS HELLO"],
  },
  {
    icon: "groups",
    tag: "GROUPS",
    title: "Groups",
    body: "Organize accounts into custom groups such as Finance, Gaming, Social Media or Development, and filter the list with one tap.",
    spec: ["CUSTOM GROUPS", "ONE-TAP FILTER"],
  },
  {
    icon: "shield",
    tag: "SHIELDED",
    title: "Privacy protection",
    body: "The app hides its content in the task switcher and blocks screenshots and screen recording. Both protections can be turned off in Settings.",
    spec: ["TASK-SWITCHER MASK", "NO SCREENSHOTS"],
  },
  {
    icon: "globe",
    tag: "I18N",
    title: "Localization",
    body: "Every string lives in flat JSON files. English, Russian, German, Spanish, French and Chinese ship by default, the system language is detected automatically, and a new language is one dropped-in file - no rebuild.",
    spec: ["6 LANGUAGES", "AUTO-DETECT", "JSON DROP-IN"],
    wide: true,
  },
  {
    icon: "frame",
    tag: "DESIGN",
    title: "Industrial design",
    body: "A strict pure black-and-white interface: square geometry, ExtraBold Onest typography, codes in JetBrains Mono, and glass surfaces reserved for overlays only.",
    spec: ["MONOCHROME", "SQUARE GEOMETRY"],
    wide: true,
  },
];

const COLUMNS = 6;

// Tailwind only ships classes it can find as literal strings.
const SPAN_CLASS: Record<number, string> = {
  2: "lg:col-span-2",
  3: "lg:col-span-3",
  4: "lg:col-span-4",
  5: "lg:col-span-5",
  6: "lg:col-span-6",
};

/**
 * Wide cards take 3 of the 6 columns, the rest take 2. A row that does not
 * add up to 6 leaves the grid's hairline background showing through as an
 * empty grey cell, so the last card of any short row absorbs the remainder.
 */
function columnSpans(features: Feature[]): number[] {
  const spans = features.map((feature) => (feature.wide ? 3 : 2));
  let used = 0;
  for (let i = 0; i < spans.length; i += 1) {
    if (used + spans[i] > COLUMNS) {
      spans[i - 1] += COLUMNS - used;
      used = 0;
    }
    used += spans[i];
  }
  if (used > 0 && used < COLUMNS) {
    spans[spans.length - 1] += COLUMNS - used;
  }
  return spans;
}

const SPANS = columnSpans(FEATURES);

export function FeatureGrid() {
  return (
    <Section
      id="features"
      eyebrow="Capabilities"
      title="Everything ships in the box."
      sub="Every module below is part of the core app - no extensions, no add-ons, no premium tier."
      headerBorder={false}
    >
      <div className="grid grid-cols-1 gap-px border-t border-line bg-line md:grid-cols-2 lg:grid-cols-6">
        {FEATURES.map((feature, i) => (
          <article
            key={feature.title}
            className={`group bg-black p-6 transition-colors duration-200 hover:bg-surface-2 md:p-10 ${
              // Two columns at md, so an odd card count would leave a hole
              // in the last row unless the final card spans both.
              i === FEATURES.length - 1 && FEATURES.length % 2 === 1
                ? "md:col-span-2"
                : ""
            } ${SPAN_CLASS[SPANS[i]]}`}
          >
            <Reveal delay={(i % 3) * 0.05} className="flex h-full flex-col">
              <div className="flex items-center gap-3">
                <Icon name={feature.icon} />
                <p className="font-mono text-xs font-semibold uppercase tracking-[0.25em] text-fg-3">
                  {feature.tag}
                </p>
              </div>
              <h3 className="mt-5 text-xl font-extrabold tracking-[-0.01em] md:text-2xl">
                {feature.title}
              </h3>
              <p className="mt-4 text-sm leading-relaxed text-fg-2 md:text-[15px]">
                {feature.body}
              </p>
              <p className="mt-auto pt-8 font-mono text-xs font-medium leading-relaxed tracking-[0.08em] text-fg-3">
                {feature.spec.join("  ·  ")}
              </p>
            </Reveal>
          </article>
        ))}
      </div>
    </Section>
  );
}
