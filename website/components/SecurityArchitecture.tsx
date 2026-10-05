import { Icon, type IconName } from "./icons";
import { Reveal } from "./Reveal";
import { Section } from "./Section";

const STAGES = [
  { label: "PASSWORD", note: "known only to you - never stored" },
  { label: "ARGON2ID", note: "memory-hard KDF · 64 MiB · 3 passes" },
  { label: "AES-256-GCM VAULT", note: "one encrypted file · random 256-bit data key" },
  { label: "PLATFORM SECURE STORAGE", note: "Keystore · Secure Enclave · Credential Store" },
  { label: "AUTHENTICATION DATA", note: "TOTP · HOTP · Steam" },
];

const GUARANTEES: { icon: IconName; label: string; note: string }[] = [
  {
    icon: "chip",
    label: "ZEROIZED MEMORY",
    note: "Key material is wiped from memory immediately after use.",
  },
  {
    icon: "once",
    label: "SINGLE-USE PAIRING",
    note: "Sync codes work exactly once; a wrong code aborts the session.",
  },
  {
    icon: "keyhole",
    label: "REVOCABLE KEY SLOTS",
    note: "Changing the password re-seals the vault and revokes all device slots.",
  },
];

export function SecurityArchitecture() {
  return (
    <Section
      id="security"
      eyebrow="Security architecture"
      title="Sealed by construction."
      sub="One password, one encrypted file, hardware-backed key slots. The chain is short enough to audit and strong enough to trust."
    >
      <Reveal>
        <div className="px-6 py-16 md:px-10 md:py-24">
          <ol className="flex flex-col items-stretch lg:flex-row lg:items-stretch">
            {STAGES.map((stage, i) => (
              <li key={stage.label} className="contents">
                <div className="flex flex-1 flex-col justify-between border border-line p-6">
                  <p className="font-mono text-sm font-medium tracking-[0.06em]">
                    {stage.label}
                  </p>
                  <p className="mt-6 font-mono text-xs font-medium leading-relaxed tracking-[0.04em] text-fg-3">
                    {stage.note}
                  </p>
                </div>
                {i < STAGES.length - 1 ? (
                  <div
                    aria-hidden
                    className="flex items-center justify-center self-center px-1 py-2 font-mono text-fg-3 lg:px-2 lg:py-0"
                  >
                    <span className="lg:hidden">↓</span>
                    <span className="hidden lg:inline">→</span>
                  </div>
                ) : null}
              </li>
            ))}
          </ol>
        </div>
      </Reveal>

      <div className="grid grid-cols-1 gap-px border-t border-line bg-line md:grid-cols-3">
        {GUARANTEES.map((item, i) => (
          <div key={item.label} className="bg-black p-6 md:p-10">
            <Reveal delay={i * 0.05}>
              <div className="flex items-center gap-3">
                <Icon name={item.icon} />
                <p className="font-mono text-xs font-semibold uppercase tracking-[0.25em] text-fg-3">
                  {item.label}
                </p>
              </div>
              <p className="mt-4 text-sm leading-relaxed text-fg-2">
                {item.note}
              </p>
            </Reveal>
          </div>
        ))}
      </div>
    </Section>
  );
}
