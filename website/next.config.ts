import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Static export so the site can be served from GitHub Pages or any static host.
  output: "export",
  images: { unoptimized: true },
  trailingSlash: true,
};

export default nextConfig;
