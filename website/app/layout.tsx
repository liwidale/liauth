import type { Metadata, Viewport } from "next";
import localFont from "next/font/local";
import "./globals.css";

const onest = localFont({
  src: "./fonts/Onest-VariableFont_wght.ttf",
  variable: "--font-onest",
  weight: "100 900",
  display: "swap",
});

const jbMono = localFont({
  src: "./fonts/JetBrainsMono-VariableFont_wght.ttf",
  variable: "--font-jb-mono",
  weight: "100 800",
  display: "swap",
});

const description =
  "A modern, open-source authenticator for Android, iOS, macOS and Windows. Offline-first, end-to-end encrypted, built around a shared Rust core.";

export const metadata: Metadata = {
  metadataBase: new URL("https://liwidale.github.io/liauth"),
  title: {
    default: "LiAuth - Your codes. Your device. Nothing else.",
    template: "%s - LiAuth",
  },
  description,
  keywords: [
    "authenticator",
    "2FA",
    "TOTP",
    "HOTP",
    "FIDO2",
    "WebAuthn",
    "open source",
    "offline",
    "encryption",
    "AES-256-GCM",
    "Argon2id",
    "Rust",
  ],
  authors: [{ name: "liwidale", url: "https://github.com/liwidale" }],
  openGraph: {
    title: "LiAuth - Your codes. Your device. Nothing else.",
    description,
    type: "website",
    siteName: "LiAuth",
    images: [{ url: "/branding/logo.png", width: 1024, height: 1024 }],
  },
  twitter: {
    card: "summary",
    title: "LiAuth - Your codes. Your device. Nothing else.",
    description,
  },
  robots: { index: true, follow: true },
};

export const viewport: Viewport = {
  themeColor: "#000000",
  colorScheme: "dark",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" className={`${onest.variable} ${jbMono.variable}`}>
      <body className="font-sans">{children}</body>
    </html>
  );
}
