import type { Metadata, Viewport } from "next";
import { cookies } from "next/headers";
import { Geist, JetBrains_Mono } from "next/font/google";
import NextTopLoader from "nextjs-toploader";
import { Providers } from "@/lib/providers";
import { Navigation } from "@/components/layout/Navigation";
import { ServiceFooter } from "@/components/layout/ServiceFooter";
import { ThemedToastContainer } from "@/components/layout/ThemedToastContainer";
import "./globals.css";
import "react-toastify/dist/ReactToastify.css";
import { cn } from "@/lib/utils";
import { htmlLang, LOCALE_COOKIE, parseLocale, t } from "@/lib/i18n";
import { SITE_URL } from "@/lib/site";
// Geist carries interface text and JetBrains Mono carries every figure, so
// numeric columns stay aligned when scanned down the page.
const geist = Geist({
  variable: "--font-sans",
  subsets: ["latin"],
  display: "swap",
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  display: "swap",
});


export const metadata: Metadata = {
  title: "Tokens - AI Token Usage Tracker & Leaderboard",
  description: "Track, visualize, and compete on AI coding assistant token usage across Claude Code, Cursor, OpenCode, Codex, Gemini, Kimi, and Qwen. The Kardashev Scale for AI Devs.",
  // Everything relative below (OG images, icons) resolves against this base —
  // the deployment's own origin, never a hardcoded upstream domain.
  metadataBase: new URL(SITE_URL),
  icons: {
    icon: [
      // SVG first so capable browsers get the theme-aware mark; the .ico stays
      { url: "/brand/tokens-favicon.svg?v=3", type: "image/svg+xml" },
      { url: "/favicon.ico?v=3", sizes: "48x48", type: "image/x-icon" },
      { url: "/favicon-16x16.png?v=3", sizes: "16x16", type: "image/png" },
      { url: "/favicon-32x32.png?v=3", sizes: "32x32", type: "image/png" },
    ],
    apple: "/brand/tokens-app-icon-180.png",
  },
  manifest: "/site.webmanifest",
  openGraph: {
    title: "Tokens - AI Token Usage Tracker & Leaderboard",
    description: "Track, visualize, and compete on AI coding assistant token usage across Claude Code, Cursor, OpenCode, Codex, Gemini, Kimi, and Qwen. The Kardashev Scale for AI Devs.",
    type: "website",
    url: SITE_URL,
    siteName: "Tokens",
    // The dynamic renderer, not a static file: it draws the current mark, so
    // the share card cannot drift from the brand the way a checked-in PNG did.
    images: [
      {
        url: "/api/og?title=Tokens&subtitle=The%20leaderboard%20for%20AI%20coding%20usage",
        width: 1200,
        height: 630,
        alt: "Tokens - AI Token Usage Tracker",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Tokens - AI Token Usage Tracker & Leaderboard",
    description: "Track, visualize, and compete on AI coding assistant token usage across Claude Code, Cursor, OpenCode, Codex, Gemini, Kimi, and Qwen.",
    images: ["/api/og?title=Tokens&subtitle=The%20leaderboard%20for%20AI%20coding%20usage"],
  },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
};

export default async function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  const jar = await cookies();
  const locale = parseLocale(jar.get(LOCALE_COOKIE)?.value);
  return (
    <html
      lang={htmlLang(locale)}
      suppressHydrationWarning
      className={cn(geist.variable, jetbrainsMono.variable)}
    >
      <head>
        {/*
          next-themes writes its no-flash theme script inline into the HTML.
          OpenNext bundles the server with esbuild's keepNames, which rewrites
          function declarations to call an `__name` helper — and that helper
          only exists inside the server bundle, not in the serialized script.
          Without this shim the theme script throws before hydration and takes
          the whole client bundle down with it.
        */}
        <script
          dangerouslySetInnerHTML={{
            __html:
              "if(!globalThis.__name){globalThis.__name=function(f){return f}}",
          }}
        />
      </head>
      <body className="flex min-h-dvh flex-col font-sans">
        <NextTopLoader color="#0073FF" showSpinner={false} />
        <a
          href="#main-content"
          className="sr-only rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-[100]"
        >
          {t(locale, "nav.skipToContent")}
        </a>
        <Providers locale={locale}>
          <Navigation />
          <div className="flex flex-1 flex-col">{children}</div>
          <ServiceFooter />
          <ThemedToastContainer />
        </Providers>
      </body>
    </html>
  );
}
