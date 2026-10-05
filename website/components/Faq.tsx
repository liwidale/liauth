"use client";

import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { useState } from "react";
import { Section } from "./Section";

const ITEMS = [
  {
    q: "Does LiAuth ever connect to the internet?",
    a: "No. There is no telemetry, no analytics and no cloud. The only network activity in the entire app is the local-network sync you explicitly start - and that traffic never leaves your Wi-Fi.",
  },
  {
    q: "What happens if I lose my device?",
    a: "Restore from an encrypted backup file, or from a second device you previously paired over local sync. There is deliberately no cloud recovery: if a copy of your secrets doesn't exist, nobody - including us - can produce one.",
  },
  {
    q: "Can I switch from my current authenticator?",
    a: "Yes. LiAuth imports Google Authenticator migration QR codes, Aegis and 2FAS backups (plain or encrypted), Authy exports, Microsoft Authenticator accounts via otpauth links, and any plain list of otpauth:// URIs.",
  },
  {
    q: "How does local sync stay private?",
    a: "Devices pair with a one-time six-digit code using SPAKE2, derive a session key via HKDF-SHA256, and encrypt every frame with AES-256-GCM. A wrong code aborts the session, each code works exactly once, and nothing ever leaves the local network.",
  },
  {
    q: "Is LiAuth really free?",
    a: "Yes. It is MIT-licensed open source with no accounts, no subscriptions and no premium tier. Every feature on this page ships in the free app.",
  },
  {
    q: "Why does my computer warn me on first launch?",
    a: "The macOS and Windows builds are not signed with a paid developer certificate, which is normal for open-source software, so the operating system may warn you on first launch. If you prefer not to rely on downloaded binaries, you can always build LiAuth from source - the repository documents the full build for every platform.",
  },
];

export function Faq() {
  const [open, setOpen] = useState<number | null>(null);
  const reduceMotion = useReducedMotion();

  return (
    <Section
      id="faq"
      eyebrow="FAQ"
      title="Questions, answered."
      headerBorder={false}
    >
      <ul className="border-t border-line">
        {ITEMS.map((item, i) => {
          const isOpen = open === i;
          return (
            <li key={item.q} className={i > 0 ? "border-t border-line" : ""}>
              <h3>
                <button
                  type="button"
                  aria-expanded={isOpen}
                  aria-controls={`faq-panel-${i}`}
                  id={`faq-button-${i}`}
                  onClick={() => setOpen(isOpen ? null : i)}
                  className="flex w-full items-center justify-between gap-6 px-6 py-6 text-left transition-colors duration-200 hover:bg-surface-2 md:px-10"
                >
                  <span className="text-base font-semibold md:text-lg">
                    {item.q}
                  </span>
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 14 14"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    aria-hidden
                    className={`shrink-0 transition-transform duration-200 ${
                      isOpen ? "rotate-45" : ""
                    }`}
                  >
                    <path d="M7 0V14M0 7H14" />
                  </svg>
                </button>
              </h3>
              <AnimatePresence initial={false}>
                {isOpen ? (
                  <motion.div
                    id={`faq-panel-${i}`}
                    role="region"
                    aria-labelledby={`faq-button-${i}`}
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: reduceMotion ? 0 : 0.2, ease: "easeOut" }}
                    className="overflow-hidden"
                  >
                    <p className="max-w-3xl px-6 pb-8 text-sm leading-relaxed text-fg-2 md:px-10 md:text-[15px]">
                      {item.a}
                    </p>
                  </motion.div>
                ) : null}
              </AnimatePresence>
            </li>
          );
        })}
      </ul>
    </Section>
  );
}
