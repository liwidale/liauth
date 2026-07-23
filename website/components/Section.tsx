import type { ReactNode } from "react";
import { Reveal } from "./Reveal";

function CrossMark({ position }: { position: "left" | "right" }) {
  return (
    <span
      aria-hidden
      className={`pointer-events-none absolute -top-[4.5px] hidden lg:block ${
        position === "left" ? "-left-[4.5px]" : "-right-[4.5px]"
      }`}
    >
      <svg width="9" height="9" viewBox="0 0 9 9" fill="none">
        <path d="M4.5 0V9M0 4.5H9" stroke="white" strokeOpacity="0.35" />
      </svg>
    </span>
  );
}

type SectionProps = {
  id?: string;
  eyebrow: string;
  title: string;
  sub?: string;
  children: ReactNode;
  headerBorder?: boolean;
};

export function Section({
  id,
  eyebrow,
  title,
  sub,
  children,
  headerBorder = true,
}: SectionProps) {
  return (
    <section id={id} className="relative border-b border-line">
      <CrossMark position="left" />
      <CrossMark position="right" />
      <Reveal>
        <header
          className={`glow-corner px-6 py-16 md:px-10 md:py-24 ${
            headerBorder ? "border-b border-line" : ""
          }`}
        >
          <p className="font-mono text-sm font-semibold uppercase tracking-[0.3em] text-fg-3">
            {eyebrow}
          </p>
          <h2 className="mt-6 max-w-3xl text-4xl font-extrabold tracking-[-0.02em] md:text-5xl">
            {title}
          </h2>
          {sub ? (
            <p className="mt-6 max-w-xl text-base leading-relaxed text-fg-2 md:text-lg">
              {sub}
            </p>
          ) : null}
        </header>
      </Reveal>
      {children}
    </section>
  );
}
