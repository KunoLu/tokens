# Web Frontend Guidelines (`@tokens/web`)

> Conventions for the Next.js app under `web/` — the Tokens social platform
> (leaderboard, public profiles, settings). Deployed to Cloudflare Workers via
> OpenNext, with Drizzle/Postgres behind Hyperdrive. There is no separate web
> backend package: API route handlers, the Drizzle schema, and data loaders
> live inside this app, mostly under `web/src/lib/`.

---

## Stack

| Layer | Choice |
|-------|--------|
| Framework | Next.js 16 App Router, React 19 (`web/package.json`) |
| Deploy | `@opennextjs/cloudflare` → Cloudflare Worker (`web/worker.ts` wraps `.open-next/worker.js`) |
| UI | shadcn v4 (`style: base-nova` in `web/components.json`), `@base-ui/react` primitives, Tailwind CSS v4, CVA, lucide-react |
| Theming | `next-themes`; semantic tokens in `web/src/app/globals.css` |
| Database | Drizzle ORM + postgres-js; Hyperdrive binding in production, `DATABASE_URL` locally (`web/src/lib/db/index.ts`) |
| Validation | Zod (`web/src/lib/validation/submission.ts`) |
| Toasts | react-toastify via `ThemedToastContainer` |

---

## Guidelines Index

| Guide | Description |
|-------|-------------|
| [Directory Structure](./directory-structure.md) | `app/` vs `components/` vs `lib/` ownership |
| [Component Guidelines](./component-guidelines.md) | shadcn in `components/ui/`, feature folders, `CONTAINER`, `tw()` legacy helper |
| [Hook Guidelines](./hook-guidelines.md) | Hooks live in `lib/use*.ts`; the `useSettings` pattern |
| [State Management](./state-management.md) | RSC `initialData`, URL query state, localStorage + cookie, local `useState` |
| [Type Safety](./type-safety.md) | `lib/types.ts`, Zod submission schemas, const-array unions, Drizzle inference |
| [Data Fetching](./data-fetching.md) | RSC calls lib functions directly; `unstable_cache` + tags; client fetch for mutations only |
| [Error Handling](./error-handling.md) | `error.tsx` boundary, API JSON errors, Alert/toast, missing-`DATABASE_URL` fallback |
| [Database Guidelines](./database-guidelines.md) | `lib/db/schema.ts`, Hyperdrive vs `DATABASE_URL`, per-request WeakMap, additive migrations |
| [Authentication](./auth.md) | Email/password APIs, PBKDF2, Resend, session cookie unchanged, no GitHub OAuth |
| [Cloudflare Deployment](./cloudflare-deployment.md) | wrangler, OpenNext cache topology, `worker.ts` edge cache + cron |
| [Quality Guidelines](./quality-guidelines.md) | No web unit tests, verification commands, upstream merge policy |

---

## Verification commands

From `web/package.json` — these are the only automated gates (there are no
web unit tests):

| Command | Purpose |
|---------|---------|
| `bun run lint` | ESLint (next core-web-vitals + typescript) |
| `bun run typecheck` | `wrangler types` → `cloudflare-env.d.ts`, then `tsc --noEmit` |
| `bun run build` | copies `install.sh`/client assets into `public/`, then `next build` |
| `bun run test:migrations` | `drizzle-kit migrate` + `scripts/check-migrations.ts` |
| `bun run cf:build` / `cf:preview` / `cf:deploy` | OpenNext Cloudflare bundle, local preview, `wrangler deploy` |

## Pre-Development Checklist

1. Read [Directory Structure](./directory-structure.md) to place the change:
   route in `app/`, primitive in `components/ui/`, feature component in a
   feature folder, shared logic in `lib/`.
2. Read [Data Fetching](./data-fetching.md) before adding any data load —
   RSC pages call `lib/` functions directly and never HTTP-fetch their own
   `/api` routes.
3. If the change touches login, registration, passwords, or email delivery,
   read [Authentication](./auth.md). Do not change `lib/auth/session.ts`
   signatures or device-flow JSON.
4. If the change touches the submission contract, read
   `web/src/lib/validation/submission.ts` together with the CLI side
   (`.trellis/spec/tokens-cli/backend/quality-guidelines.md`) — both must move
   together.

## Quality Check

- `bun run lint` and `bun run typecheck` pass.
- Styling uses semantic tokens (`bg-background`, `text-muted-foreground`,
  `border`) — no hardcoded hex, no manual `dark:` branches.
- New components come from `components/ui/` (vendored shadcn) — no new
  third-party component library, no styled-components.
- DB schema changes are additive migrations, hand-reviewed per
  [Database Guidelines](./database-guidelines.md).

---

**Language**: All documentation is written in **English**.
