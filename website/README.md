# LiAuth website

The landing page for LiAuth, built with Next.js (App Router), TypeScript, Tailwind CSS and Framer Motion. Branding is pulled from [`../branding`](../branding) (copies live in `public/branding`), and the Onest / JetBrains Mono fonts come from [`../assets/fonts`](../assets/fonts) (copies live in `app/fonts`).

## Develop

```sh
npm install
npm run dev
```

## Build

```sh
npm run build
```

The site is statically exported to `out/` (`output: "export"`), so it can be served from GitHub Pages or any static host.

If you replace `branding/logo.png` or `branding/text.png` at the repo root, copy them into `public/branding/` here as well.
