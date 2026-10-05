type IconName =
  | "clock"
  | "key"
  | "cloud-off"
  | "lock"
  | "fingerprint"
  | "sync"
  | "archive"
  | "import"
  | "groups"
  | "shield"
  | "globe"
  | "frame"
  | "chip"
  | "once"
  | "keyhole";

// Filled monochrome glyphs with negative-space cutouts, drawn for the
// square LiAuth geometry. currentColor only - they invert with the text.
const GLYPHS: Record<IconName, React.ReactNode> = {
  clock: (
    <path
      fillRule="evenodd"
      d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20Zm-1 4h2v6.6l4.4 2.5-1 1.74L11 13.7V6Z"
    />
  ),
  key: (
    <>
      <path
        fillRule="evenodd"
        d="M7 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10Zm0 3.2a1.8 1.8 0 1 1 0 3.6 1.8 1.8 0 0 1 0-3.6Z"
      />
      <path d="M12 11h10v2h-2v4h-2v-4h-3v4h-2v-4h-1v-2Z" />
    </>
  ),
  "cloud-off": (
    <>
      <path d="M17.6 18H7a4.5 4.5 0 0 1-.9-8.91A6 6 0 0 1 17.7 7.9 4.6 4.6 0 0 1 17.6 18Z" />
      <path d="M4 4.4 5.4 3l15.6 15.6-1.4 1.4L4 4.4Z" fill="black" />
      <path d="M3 5.4 4.4 4 20 19.6 18.6 21 3 5.4Z" />
    </>
  ),
  lock: (
    <path
      fillRule="evenodd"
      d="M12 2.5A4.5 4.5 0 0 1 16.5 7v2.5H18a1 1 0 0 1 1 1V20a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1v-9.5a1 1 0 0 1 1-1h1.5V7A4.5 4.5 0 0 1 12 2.5Zm0 2A2.5 2.5 0 0 0 9.5 7v2.5h5V7A2.5 2.5 0 0 0 12 4.5Zm0 8.5a1.8 1.8 0 0 1 .9 3.36V18h-1.8v-1.64A1.8 1.8 0 0 1 12 13Z"
    />
  ),
  fingerprint: (
    <>
      <circle cx="12" cy="12" r="10" />
      <g fill="none" stroke="black" strokeWidth="1.6">
        <path d="M5.8 13.5a6.2 6.2 0 0 1 12.4 0" />
        <path d="M8.8 13.5a3.2 3.2 0 0 1 6.4 0" />
        <path d="M12 13.2v4.3" />
      </g>
    </>
  ),
  sync: (
    <>
      <path d="M3 7h11V4.2L21 8l-7 3.8V9H3V7Z" />
      <path d="M21 15H10v-2.8L3 16l7 3.8V17h11v-2Z" />
    </>
  ),
  archive: (
    <>
      <path d="M3 4h18v4H3V4Z" />
      <path
        fillRule="evenodd"
        d="M5 9.5h14V20H5V9.5Zm4 2.5v2h6v-2H9Z"
      />
    </>
  ),
  import: (
    <>
      <path d="M11 3h2v7.5h3.2L12 15 7.8 10.5H11V3Z" />
      <path d="M4 14h2v5h12v-5h2v7H4v-7Z" />
    </>
  ),
  groups: (
    <path d="M4 4h7v7H4V4Zm9 0h7v7h-7V4ZM4 13h7v7H4v-7Zm9 0h7v7h-7v-7Z" />
  ),
  shield: (
    <path
      fillRule="evenodd"
      d="M12 2l8 3v6.1c0 4.9-3.4 9.1-8 10.9-4.6-1.8-8-6-8-10.9V5l8-3Zm-1.2 13.2 5.4-5.4-1.4-1.4-4 4-1.6-1.6-1.4 1.4 3 3Z"
    />
  ),
  globe: (
    <>
      <circle cx="12" cy="12" r="10" />
      <g fill="none" stroke="black" strokeWidth="1.5">
        <ellipse cx="12" cy="12" rx="4.6" ry="10" />
        <path d="M2.5 9h19M2.5 15h19" />
      </g>
    </>
  ),
  frame: (
    <>
      <path fillRule="evenodd" d="M4 4h16v16H4V4Zm4 4v8h8V8H8Z" />
      <path d="M10.6 10.6h2.8v2.8h-2.8v-2.8Z" />
    </>
  ),
  chip: (
    <>
      <path
        fillRule="evenodd"
        d="M6 6h12v12H6V6Zm3 3v6h6V9H9Z"
      />
      <path d="M10 2h1.5v3H10V2Zm2.5 0H14v3h-1.5V2ZM10 19h1.5v3H10v-3Zm2.5 0H14v3h-1.5v-3ZM2 10h3v1.5H2V10Zm0 2.5h3V14H2v-1.5ZM19 10h3v1.5h-3V10Zm0 2.5h3V14h-3v-1.5Z" />
    </>
  ),
  once: (
    <path
      fillRule="evenodd"
      d="M4 4h16v16H4V4Zm8.9 3.2h-1.6L8.9 8.6v1.9l2-1v7.3h2V7.2Z"
    />
  ),
  keyhole: (
    <path
      fillRule="evenodd"
      d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20Zm0 5a2.6 2.6 0 0 1 1.3 4.85L14.2 17H9.8l.9-5.15A2.6 2.6 0 0 1 12 7Z"
    />
  ),
};

export function Icon({
  name,
  size = 20,
  className,
}: {
  name: IconName;
  size?: number;
  className?: string;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
      aria-hidden
      className={className}
    >
      {GLYPHS[name]}
    </svg>
  );
}

export type { IconName };
