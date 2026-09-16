import type { NextConfig } from "next";
import { PHASE_DEVELOPMENT_SERVER, PHASE_PRODUCTION_BUILD } from "next/constants";
import { initOpenNextCloudflareForDev } from "@opennextjs/cloudflare";


const nextConfig: NextConfig = {
  images: {
    // Workers has no Vercel image optimizer. Every `next/image` source in this
    // app is a local, pre-optimized static asset (svg/webp/png) served straight
    // from Cloudflare's edge, and GitHub avatars go through plain <img>, so
    // opting out costs nothing and avoids a Cloudflare Images bill.
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

// Makes the Cloudflare bindings (Hyperdrive, R2, Durable Objects) available to
// `next dev` and `next build` (prerendered pages query the DB through
// Hyperdrive's localConnectionString). Production `next start` is the
// self-host path and must talk to `DATABASE_URL` only. The function form is
// the supported way to read the phase — `process.env.NEXT_PHASE` is not set
// when the config loads under `next start`.
export default function config(phase: string): NextConfig {
  if (phase === PHASE_DEVELOPMENT_SERVER || phase === PHASE_PRODUCTION_BUILD) {
    initOpenNextCloudflareForDev();
  }
  return nextConfig;
}


