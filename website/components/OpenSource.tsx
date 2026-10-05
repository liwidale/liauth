import { GITHUB_URL } from "@/lib/site";
import { Reveal } from "./Reveal";
import { Section } from "./Section";

const CRATES = [
  { name: "liauth-core", note: "TOTP · HOTP · Steam · otpauth" },
  { name: "liauth-crypto", note: "AES-256-GCM envelopes · Argon2id · key slots" },
  { name: "liauth-vault", note: "encrypted vault · groups · settings · backups" },
  { name: "liauth-import", note: "Google Authenticator · Aegis · 2FAS · Authy" },
  { name: "liauth-sync", note: "mDNS discovery · SPAKE2 pairing" },
  { name: "liauth-ffi", note: "uniffi bindings for Kotlin and Swift" },
];

export function OpenSource() {
  return (
    <Section
      id="open-source"
      eyebrow="Open source"
      title="One Rust core, shared by every app."
      sub="All business logic - codes, crypto, vault, importers, sync - lives in a single Rust workspace, compiled natively for each platform and bridged to Kotlin and Swift. The interface is the only thing each platform writes itself."
      headerBorder={false}
    >
      <div className="grid grid-cols-1 gap-px border-t border-line bg-line lg:grid-cols-2">
        <div className="bg-black p-6 md:p-10">
          <Reveal className="flex h-full flex-col">
            <p className="font-mono text-xs font-semibold uppercase tracking-[0.25em] text-fg-3">
              MIT Licensed
            </p>
            <p className="mt-5 max-w-md text-sm leading-relaxed text-fg-2 md:text-[15px]">
              Free to use, modify and distribute. The repository ships with CI
              that enforces formatting, lints and tests on every push - 52 unit
              tests including the official RFC 4226 and RFC 6238 vectors, and
              clippy running with warnings as errors.
            </p>
            <div className="mt-8 space-y-2 font-mono text-xs font-medium tracking-[0.06em] text-fg-3">
              <p>cargo fmt --all</p>
              <p>cargo clippy --workspace -- -D warnings</p>
              <p>cargo test --workspace</p>
            </div>
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="mt-auto inline-flex items-center gap-2 pt-10 text-sm font-semibold transition-colors duration-150 hover:text-fg-2"
            >
              View source on GitHub <span aria-hidden>→</span>
            </a>
          </Reveal>
        </div>
        <div className="bg-black p-6 md:p-10">
          <Reveal>
            <p className="font-mono text-xs font-semibold uppercase tracking-[0.25em] text-fg-3">
              core/
            </p>
            <ul className="mt-5 font-mono text-[13px] leading-relaxed">
              {CRATES.map((crate, i) => (
                <li
                  key={crate.name}
                  className="flex flex-col gap-1 border-t border-line-soft py-3 sm:flex-row sm:items-baseline sm:justify-between sm:gap-6"
                >
                  <span className="whitespace-nowrap">
                    <span aria-hidden className="text-fg-3">
                      {i === CRATES.length - 1 ? "└── " : "├── "}
                    </span>
                    {crate.name}
                  </span>
                  <span className="text-xs text-fg-3 sm:text-right">
                    {crate.note}
                  </span>
                </li>
              ))}
            </ul>
          </Reveal>
        </div>
      </div>
    </Section>
  );
}
