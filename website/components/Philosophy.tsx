import { Reveal } from "./Reveal";

const THESES = [
  "No telemetry.",
  "No analytics.",
  "No cloud.",
  "No tracking.",
  "No advertisements.",
];

export function Philosophy() {
  return (
    <section id="philosophy" className="relative border-b border-line">
      <div className="glow-corner px-6 py-16 md:px-10 md:py-24">
        <Reveal>
          <p className="font-mono text-sm font-semibold uppercase tracking-[0.3em] text-fg-3">
            Philosophy
          </p>
          <h2 className="mt-6 text-5xl font-extrabold tracking-[-0.03em] md:text-7xl">
            Privacy is the default.
          </h2>
        </Reveal>
      </div>
      <ul>
        {THESES.map((thesis, i) => (
          <li key={thesis} className="border-t border-line">
            <Reveal delay={i * 0.04}>
              <p className="px-6 py-5 text-3xl font-extrabold tracking-[-0.02em] text-fg-3 transition-colors duration-200 hover:text-white md:px-10 md:py-7 md:text-5xl">
                {thesis}
              </p>
            </Reveal>
          </li>
        ))}
      </ul>
      <div className="border-t border-line px-6 py-16 md:px-10 md:py-24">
        <Reveal>
          <p className="max-w-4xl text-3xl font-extrabold leading-tight tracking-[-0.02em] md:text-5xl">
            Your secrets never leave your device.
          </p>
        </Reveal>
      </div>
    </section>
  );
}
