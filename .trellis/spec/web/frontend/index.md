# Web Frontend Guidelines (`@tokens/web`)

> Conventions for the Next.js app under `web/` — the Tokens social platform
> (leaderboard, public profiles, settings). Self-hosted: `next build` +
> `next start` on a Node server, with Drizzle/Postgres over `DATABASE_URL`.
> There is no separate web backend package: API route handlers, the Drizzle
> schema, and data loaders live inside this app, mostly under `web/src/lib/`.


---

## Stack

| Layer | Choice |
|-------|--------|
| Framework | Next.js 16 App Router, React 19 (`web/package.json`) |
| Deploy | Self-hosted Node: `next build` + `next start` (the Cloudflare Workers layer was removed in the self-host cutover) |
| UI | shadcn v4 (`style: base-nova` in `web/components.json`), `@base-ui/react` primitives, Tailwind CSS v4, CVA, lucide-react |
| Theming | `next-themes`; semantic tokens in `web/src/app/globals.css` |
| Database | Drizzle ORM + postgres-js over `DATABASE_URL` in every environment (`web/src/lib/db/index.ts`) |
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
| [Database Guidelines](./database-guidelines.md) | `lib/db/schema.ts`, `DATABASE_URL` connection lifecycle, additive migrations |
| [Authentication](./auth.md) | Email/password APIs, PBKDF2, Resend, session cookie unchanged, no GitHub OAuth |
| [Self-host Deployment](./self-host-deployment.md) | Node `next build`/`next start`, cron HTTP endpoint, in-process rate limit |
| [Quality Guidelines](./quality-guidelines.md) | No web unit tests, verification commands, upstream merge policy |


---

## Verification commands

From `web/package.json` — these are the only automated gates (there are no
web unit tests; Playwright E2E is the T7/T8/T9/T10/T11 exception for browser journeys):

| Command | Purpose |
|---------|---------|
| `bun run lint` | ESLint (next core-web-vitals + typescript) |
| `bun run typecheck` | `tsc --noEmit` |
| `bun run build` | copies `install.sh`/client assets into `public/`, then `next build` |
| `bun run test:migrations` | `drizzle-kit migrate` + `scripts/check-migrations.ts` |
| `bun run test:teams` | T4 domain invariants (`scripts/check-teams-invariants.ts`); T7 FR-2 Teamboard pagination / multi-value `groupIds`, T8 `listMyInvitations`, and T9 §12 (creator admin, subadmin remaining ops, public→private Teamboard, delete-after-disband) live in the same runner |
| `bun run test:e2e` | Playwright LocaleToggle, full-page copy, Privacy/Terms English-precedence, Teamboard, Profile membership, T9 acceptance (`t9-acceptance.spec.ts`), and maintenance-cron auth (`cron.spec.ts`) (repo-root `tests/e2e/`, config `web/playwright.config.ts`) |



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

- `bun run lint` and `bun run typecheck` pass. Team/group domain and Teamboard loader changes also run `bun run test:teams`. LocaleToggle / full-page copy / Privacy-Terms / Teamboard / Profile membership / T9 acceptance journeys also run `bun run test:e2e`.
- Styling uses semantic tokens (`bg-background`, `text-muted-foreground`,
  `border`) — no hardcoded hex, no manual `dark:` branches.
- New components come from `components/ui/` (vendored shadcn) — no new
  third-party component library, no styled-components.
- DB schema changes are additive migrations, hand-reviewed per
  [Database Guidelines](./database-guidelines.md).

---

**Language**: All documentation is written in **English**.
