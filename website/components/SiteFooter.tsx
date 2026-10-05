import Image from "next/image";
import {
  DEVELOPER_URL,
  GITHUB_URL,
  LICENSE_URL,
  RELEASES_URL,
  SECURITY_URL,
} from "@/lib/site";

const COLUMNS = [
  {
    heading: "Project",
    links: [
      { label: "GitHub", href: GITHUB_URL },
      { label: "Releases", href: RELEASES_URL },
      { label: "Security advisories", href: SECURITY_URL },
      { label: "Developer", href: DEVELOPER_URL },
    ],
  },
  {
    heading: "Legal",
    links: [{ label: "MIT License", href: LICENSE_URL }],
  },
];

export function SiteFooter() {
  return (
    <footer className="relative">
      <div className="grid grid-cols-1 gap-px bg-line md:grid-cols-2">
        <div className="bg-black px-6 py-12 md:px-10 md:py-16">
          <div className="flex items-center gap-3">
            <Image src="/branding/logo.png" alt="" width={24} height={24} />
            <Image
              src="/branding/text.png"
              alt="LiAuth"
              width={90}
              height={28}
              className="h-5 w-auto"
            />
          </div>
          <p className="mt-6 max-w-xs text-sm leading-relaxed text-fg-2">
            Your codes. Your device. Nothing else.
          </p>
          <p className="mt-8 font-mono text-xs font-medium tracking-[0.08em] text-fg-3">
            OFFLINE-FIRST · E2E ENCRYPTED · RUST CORE
          </p>
        </div>
        <div className="grid grid-cols-2 gap-px bg-line">
          {COLUMNS.map((column) => (
            <nav
              key={column.heading}
              aria-label={column.heading}
              className="bg-black px-6 py-12 md:px-10 md:py-16"
            >
              <p className="font-mono text-xs font-semibold uppercase tracking-[0.25em] text-fg-3">
                {column.heading}
              </p>
              <ul className="mt-6 space-y-3">
                {column.links.map((link) => (
                  <li key={link.label}>
                    <a
                      href={link.href}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-sm text-fg-2 transition-colors duration-150 hover:text-white"
                    >
                      {link.label}
                    </a>
                  </li>
                ))}
              </ul>
            </nav>
          ))}
        </div>
      </div>
      <div className="flex flex-col gap-2 border-t border-line px-6 py-6 font-mono text-xs font-medium tracking-[0.08em] text-fg-3 sm:flex-row sm:items-center sm:justify-between md:px-10">
        <p>© 2026 LIWIDALE - MIT LICENSE</p>
        <p>NO TELEMETRY · NO ANALYTICS · NO CLOUD</p>
      </div>
    </footer>
  );
}
