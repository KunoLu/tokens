import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Dev-only: the site is served as localhost:3000 but often browsed via
  // 127.0.0.1:3000; allow both origins for /_next/* dev resources so the dev
  // badge doesn't count a cross-origin warning as an issue.
  allowedDevOrigins: ["127.0.0.1", "localhost"],

  images: {
    // Every `next/image` source is a local, pre-optimized static asset
    // (svg/webp/png) and GitHub avatars go through plain <img>, so the image
    // optimizer is opted out — one less moving part on the self-hosted
    // Node server.
    unoptimized: true,
    remotePatterns: [
      {
        protocol: "https",
        hostname: "avatars.githubusercontent.com",
        pathname: "/**",
      },
      {
        protocol: "https",
        hostname: "github.com",
        pathname: "/**",
      },
    ],
  },

  // Next streams metadata into <body> for clients it does not recognise as
  // bots, and only blocks to put it in <head> for the ones it does. Profile
  // HTML is cached at the edge by URL alone, so whichever reader fills the
  // cache decides what every crawler gets — and a browser-filled entry put
  // og:image ~14MB into the document, far past where X, Slack, Discord or
  // Facebook stop reading. Treating every client as a bot keeps it in <head>.
  htmlLimitedBots: /.*/,

  // Security headers for production
  headers: async () => [
    {
      source: "/:path*",
      headers: [
        {
          key: "X-DNS-Prefetch-Control",
          value: "on",
        },
        {
          key: "X-Frame-Options",
          value: "SAMEORIGIN",
        },
        {
          key: "X-Content-Type-Options",
          value: "nosniff",
        },
        {
          key: "Referrer-Policy",
          value: "strict-origin-when-cross-origin",
        },
      ],
    },
  ],

  // Experimental features
  experimental: {
    // Enable server actions
    serverActions: {
      bodySizeLimit: "2mb",
    },
  },
};

export default nextConfig;
