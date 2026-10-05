import Image from "next/image";
import { RELEASES_URL } from "@/lib/site";
import { AppRender } from "./AppRender";
import { Reveal } from "./Reveal";
import { RotatingWord } from "./RotatingWord";

export function Hero() {
  return (
    <section id="top" className="glow-hero relative border-b border-line">
      <div className="flex flex-col items-center px-6 pt-20 text-center md:pt-28">
        <Reveal>
          <Image
            src="/branding/logo.png"
            alt=""
            width={64}
            height={64}
            priority
          />
        </Reveal>
        <Reveal delay={0.05}>
          <h1 className="mt-10 max-w-5xl text-5xl font-extrabold leading-[1.08] tracking-[-0.03em] md:text-7xl">
            Your <RotatingWord />.
            <br />
            Nothing else.
          </h1>
        </Reveal>
        <Reveal delay={0.1}>
          <p className="mt-8 max-w-2xl text-base leading-relaxed text-fg-2 md:text-lg">
            A modern, open-source authenticator for Android, iOS, macOS and
            Windows. Offline-first, end-to-end encrypted, built around a shared
            Rust core.
          </p>
        </Reveal>
        <Reveal delay={0.15}>
          <a
            href={RELEASES_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="mt-10 inline-flex h-12 items-center bg-white px-8 text-base font-semibold text-black transition-[background-color,transform] duration-150 hover:-translate-y-0.5 hover:bg-neutral-300"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              className="mr-3"
              aria-hidden
            >
              <path d="M8 1V11M8 11L4 7M8 11L12 7M2 14H14" />
            </svg>
            Download
          </a>
        </Reveal>
        <Reveal delay={0.2} className="mt-16 w-full md:mt-24">
          <AppRender />
        </Reveal>
      </div>
    </section>
  );
}
