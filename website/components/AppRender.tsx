"use client";

import Image from "next/image";
import { useEffect, useRef, useState } from "react";

const BASE_ACCOUNTS = [
  { initial: "G", name: "GitHub", sub: "user@example.com" },
  { initial: "U", name: "user@example.com", sub: null },
];

const EXTRA_ACCOUNT = { initial: "P", name: "Proton", sub: "user@proton.me" };

// Deterministic pseudo-codes for the demo window; changes every 30 s like real TOTP.
function demoCode(period: number, index: number): string {
  let x = (Math.imul(period, 2654435761) ^ Math.imul(index + 1, 40503)) >>> 0;
  x = Math.imul(x ^ (x >>> 15), 2246822519);
  x = Math.imul(x ^ (x >>> 13), 3266489917);
  x = (x ^ (x >>> 16)) >>> 0;
  const digits = (x % 1000000).toString().padStart(6, "0");
  return `${digits.slice(0, 3)} ${digits.slice(3)}`;
}

function useDemoTotp(count: number) {
  const [state, setState] = useState<{
    codes: string[];
    remaining: number;
    fraction: number;
  } | null>(null);

  useEffect(() => {
    const tick = () => {
      const now = Date.now() / 1000;
      const period = Math.floor(now / 30);
      const left = 30 - (now % 30);
      setState({
        codes: Array.from({ length: count }, (_, i) => demoCode(period, i)),
        remaining: Math.ceil(left),
        fraction: left / 30,
      });
    };
    tick();
    const id = setInterval(tick, 250);
    return () => clearInterval(id);
  }, [count]);

  return state;
}

export function AppRender() {
  const [accounts, setAccounts] = useState(BASE_ACCOUNTS);
  const [copied, setCopied] = useState<number | null>(null);
  const copyTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const totp = useDemoTotp(accounts.length);

  useEffect(() => {
    return () => {
      if (copyTimer.current) clearTimeout(copyTimer.current);
    };
  }, []);

  const copyCode = (index: number) => {
    const code = totp?.codes[index];
    if (code) {
      navigator.clipboard?.writeText(code.replace(" ", "")).catch(() => {});
    }
    setCopied(index);
    if (copyTimer.current) clearTimeout(copyTimer.current);
    copyTimer.current = setTimeout(() => setCopied(null), 1400);
  };

  const addAccount = () => {
    setAccounts((current) =>
      current.length === BASE_ACCOUNTS.length
        ? [...current, EXTRA_ACCOUNT]
        : current,
    );
  };

  return (
    <div className="mx-auto w-full max-w-[440px] border-x border-t border-line bg-black text-left">
      {/* Title bar */}
      <div
        aria-hidden
        className="flex h-10 items-center gap-2.5 border-b border-line-soft bg-surface-2 px-4"
      >
        <Image src="/branding/logo.png" alt="" width={14} height={14} />
        <span className="text-xs text-fg-3">LiAuth</span>
        <div className="ml-auto flex items-center gap-5 text-fg-3">
          <svg width="10" height="10" viewBox="0 0 10 10" stroke="currentColor" strokeWidth="1">
            <path d="M0 5H10" />
          </svg>
          <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1">
            <rect x="0.5" y="0.5" width="8" height="8" />
          </svg>
          <svg width="10" height="10" viewBox="0 0 10 10" stroke="currentColor" strokeWidth="1">
            <path d="M0 0L10 10M10 0L0 10" />
          </svg>
        </div>
      </div>

      {/* App body */}
      <div className="space-y-5 p-5 md:p-6">
        <div className="flex items-center justify-between">
          <Image
            src="/branding/text.png"
            alt=""
            width={96}
            height={30}
            className="h-[22px] w-auto"
          />
          <div className="flex gap-2">
            <button
              type="button"
              onClick={addAccount}
              aria-label="Add a demo account"
              className="flex h-10 w-10 items-center justify-center border border-line transition-colors duration-150 hover:bg-surface-2 disabled:opacity-40"
              disabled={accounts.length > BASE_ACCOUNTS.length}
            >
              <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="white" strokeWidth="1.5" aria-hidden>
                <path d="M9 3V15M3 9H15" />
              </svg>
            </button>
            <div aria-hidden className="flex h-10 w-10 items-center justify-center border border-line">
              <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="white" strokeWidth="1.5">
                <path d="M3 6H13M13 6L10 3M13 6L10 9M15 12H5M5 12L8 9M5 12L8 15" />
              </svg>
            </div>
            <div aria-hidden className="flex h-10 w-10 items-center justify-center border border-line">
              <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="white" strokeWidth="1.5">
                <path d="M3 5H15M3 9H15M3 13H15M6 3.5V6.5M12 7.5V10.5M8 11.5V14.5" />
              </svg>
            </div>
          </div>
        </div>

        <div aria-hidden className="flex h-11 items-center bg-surface-3 px-4 text-sm text-fg-3">
          Search
        </div>

        <div aria-hidden className="flex gap-2">
          <span className="flex h-8 items-center bg-white px-4 text-xs font-semibold text-black">
            All
          </span>
          <span className="flex h-8 items-center border border-line px-4 text-xs text-fg-2">
            Groups…
          </span>
        </div>

        <div className="space-y-3">
          {accounts.map((account, i) => (
            <button
              key={account.initial}
              type="button"
              onClick={() => copyCode(i)}
              aria-label={`Copy the one-time code for ${account.name}`}
              className={`block w-full border p-4 text-left transition-colors duration-150 hover:bg-surface-2 active:translate-y-px ${
                copied === i ? "border-white/40" : "border-line"
              }`}
            >
              <div className="flex items-start gap-4">
                <div className="flex h-11 w-11 shrink-0 items-center justify-center border border-line text-base font-semibold">
                  {account.initial}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-semibold leading-tight">
                    {account.name}
                  </p>
                  {account.sub ? (
                    <p className="truncate text-xs text-fg-3">{account.sub}</p>
                  ) : null}
                  <p className="mt-1 font-mono text-[26px] font-medium leading-none tracking-[0.04em] tabular-nums">
                    {totp ? totp.codes[i] : "000 000"}
                  </p>
                </div>
                <span
                  aria-hidden
                  className={`font-mono text-xs tabular-nums ${
                    copied === i ? "font-semibold text-white" : "text-fg-3"
                  }`}
                >
                  {copied === i
                    ? "COPIED"
                    : totp
                      ? String(totp.remaining).padStart(2, "0")
                      : "30"}
                </span>
              </div>
              <div aria-hidden className="mt-4 h-px w-full bg-white/15">
                <div
                  className="h-px bg-white"
                  style={{ width: `${(totp ? totp.fraction : 1) * 100}%` }}
                />
              </div>
            </button>
          ))}
        </div>

        <div className="h-6" />
      </div>
    </div>
  );
}
