# Directory Structure: `web/`

> Where code lives in the Next.js app. Path alias `@/*` → `src/*`
> (`web/tsconfig.json`).

---

## Map

```
web/
├── src/
│   ├── app/                      # App Router routes + API handlers
│   │   ├── layout.tsx            # Root layout: fonts, Providers, Navigation
│   │   ├── error.tsx             # Root error boundary (client)
│   │   ├── globals.css           # Tailwind v4 + shadcn theme tokens (@theme)
│   │   ├── (main)/               # Leaderboard home, teamboard, docs, legal
│   │   ├── u/[username]/         # Public profile: RSC page + ProfilePageClient
│   │   ├── settings/             # Thin RSC page → SettingsClient
│   │   ├── login/, register/, forgot-password/, reset-password/, verify-email/
│   │   ├── device/, local/       # Device-link / local client flows (not GitHub OAuth)
│   │   └── api/                  # Route handlers: JSON, SVG embeds, OG images
│   ├── components/
│   │   ├── ui/                   # Vendored shadcn primitives (button, card, table, …)
│   │   ├── layout/               # Navigation, Container (CONTAINER), PageHeader, footer
│   │   ├── profile/              # Profile feature components + barrel index.tsx
│   │   ├── leaderboard/          # Leaderboard table + skeleton
│   │   ├── docs/, legal/         # Feature folders per route
│   │   └── Graph*.tsx            # Shared visualization components
│   ├── lib/                      # Server + shared client logic
│   │   ├── db/                   # Drizzle: schema.ts, migrations/, usernameLookup.ts, index.ts
│   │   ├── auth/                 # Session, tokens, bearer, password, emailTokens, rateLimit
│   │   ├── email/                # Resend HTTP send helpers (no SDK)
│   │   ├── avatar.ts             # Initials SVG fallback; never github.com png
│   │   ├── leaderboard/          # getLeaderboard.ts, types, constants, dateRange
│   │   ├── validation/           # Zod submission schemas (submission.ts)
│   │   ├── embed/                # SVG embed renderers + generated logos
│   │   ├── providers/            # ThemeProvider wrapper
│   │   ├── types.ts              # SUPPORTED_CLIENT_TYPES, ClientType, TokenBreakdown
│   │   ├── utils.ts              # Domain formatters + cn()
│   │   ├── format.ts             # Display formatting (compact numbers, XML escape)
│   │   ├── tw.tsx                # tw() styled-element helper (styled-components leftover)
│   │   └── useSettings.ts        # Cross-route client preferences hook
│   └── types/                    # Ambient declarations (static-images.d.ts)
├── scripts/                      # Migration checks, codegen (excluded from tsc)
├── drizzle.config.ts
├── wrangler.jsonc                # Cloudflare Worker config (production!)
├── open-next.config.ts           # OpenNext cache topology
├── next.config.ts                # unoptimized images, security headers,
│                                 # initOpenNextCloudflareForDev
└── worker.ts                     # Edge cache for OG/embed/HTML + daily cron
```

## Ownership rules

- **`app/` is routing, not logic.** Pages are thin: an async server component
  loads data via `lib/` and passes `initialData` to a client component
  (`app/u/[username]/page.tsx` → `ProfilePageClient`). Heavy mutation logic
  lives in route handlers (`app/api/submit/route.ts`) plus `lib/`.
- **`components/ui/` is vendored shadcn** — edit like shadcn code, import via
  `@/components/ui/*`. Feature components live in a folder per route/domain
  (`profile/`, `leaderboard/`), with a barrel `index.tsx` where the
  surface is large (`components/profile/index.tsx`).
- **`lib/` holds all shared logic, including hooks.** `components.json`
  declares an `@/hooks` alias but there is **no `src/hooks/` directory** —
  hooks are `lib/use*.ts` (`useSettings.ts`). Do not create `src/hooks/`.
- **Server-only modules stay out of client bundles**: data loaders like
  `lib/publicProfileData.ts` and everything under `lib/db/` are imported by
  RSC pages and route handlers, not by client components.

## Where new things go

| New thing | Location |
|-----------|----------|
| Page/route | `src/app/…`, with data loading in an async server component |
| API endpoint | `src/app/api/…/route.ts`, sharing logic with pages through `lib/` |
| Reusable primitive | `src/components/ui/` (shadcn pattern) |
| Feature component | `src/components/<feature>/` |
| Data loader / query | `src/lib/` (with `unstable_cache` where cacheable) |
| Client hook | `src/lib/use*.ts` |
| DB table/column | `src/lib/db/schema.ts` + additive migration (see [Database Guidelines](./database-guidelines.md)) |
