# Quality Guidelines

> Hard rules for `web/`. There are **no web unit tests** — verification is
> lint, typecheck, build, and the migration check. Quality is enforced by
> convention and review.

---

## Verification (the only gates)

| Command | Gate |
|---------|------|
| `bun run lint` | ESLint (next core-web-vitals + typescript) |
| `bun run typecheck` | `wrangler types` → `cloudflare-env.d.ts` + `tsc --noEmit` |
| `bun run build` | asset copies + `next build` |
| `bun run test:migrations` | `drizzle-kit migrate` + `scripts/check-migrations.ts` |

Do not add a unit-test setup without a team decision — the upstream tests were
deliberately removed (`docs/upstream_policy.md`), and the current safety net
is the four commands above plus code review. Small testability affordances
exist (e.g. `formatRelativeTime` takes an injectable `now`,
`scripts/migrate-retry.ts` is extracted for testability), but no `*.test.ts`
files exist under `web/`.

## Hard rules

1. **Semantic tokens only.** `bg-background`, `text-muted-foreground`,
   `border`, … — no hardcoded hex, no manual `dark:` branches. Both themes
   must fall out of the tokens in `globals.css` (`@theme`); a class that
   silently does nothing usually means the token is missing.
2. **Vendored shadcn only.** Components come from `web/src/components/ui/`
   (`@base-ui/react` + CVA). No new third-party component library; no new
   styled-components — remaining `tw()` usage (`web/src/lib/tw.tsx`) is a
   leftover that shrinks as pages are rewritten.
3. **Numbers use `.tabular`** so columns line up.
4. **RSC never fetches own API.** Pages call `lib/` loaders directly — see
   [Data Fetching](./data-fetching.md).
5. **Submission contract moves in lockstep with the CLI.** Zod schemas in
   `web/src/lib/validation/submission.ts` must match the CLI's camelCase
   payload structs, including the Cursor legacy `premium-tool-call` carve-out
   (`CURSOR_LEGACY_TOKENLESS_MODELS`, `submission.ts:420-429` ↔
   `cli/tokens-cli/src/main.rs:1931+`).
6. **Migrations are additive and hand-reviewed** — see
   [Database Guidelines](./database-guidelines.md).

## Upstream policy (Tokscale fork)

`docs/upstream_policy.md` is the rulebook for merging from upstream:

- **Never merge**: branding/naming/copy (`tokscale` strings, logos), frontend
  styling/components/layout, TUI/report-command features, groups, tests.
- **Always merge**: new providers/client scanners, parser fixes, submit
  pipeline and correctness fixes.
- **Frontend data capability: merge the capability, rewrite the
  implementation.** Take the data logic, redraw it with our components —
  never cherry-pick upstream UI. Two design languages at once is the debt
  this rule exists to prevent.
- **Migrations: review each by hand** before landing.
- Path translation: `crates/tokscale-core` → `cli/tokens-core`,
  `packages/frontend` → `web`. Skipped commits get listed with a reason in
  the sync PR body.
- After a sync: `cargo check --manifest-path cli/Cargo.toml --workspace
  --all-targets` and `bun run typecheck` in `web/` are the only automated
  checks.

## Anti-patterns observed (do not reintroduce)

| Anti-pattern | Correct pattern |
|--------------|-----------------|
| Server-side `fetch` to own `/api/*` from RSC | Shared `lib/` function (`loadPublicProfileForPage`) |
| Hyperdrive stacked on Neon's PgBouncer pooler | Point Hyperdrive at the direct endpoint |
| DB pool reused across CF requests | Per-request client keyed by `ctx` (WeakMap in `lib/db/index.ts`) |
| `revalidate` + `searchParams` on Workers | `export const dynamic = 'force-dynamic'` |
| HeroUI / styled-components | shadcn + Tailwind tokens + `tw()` leftover only |
| `next/image` optimization on Workers | Disabled; static assets + plain `<img>` |
| Assuming `@/hooks` exists | Hooks live in `lib/use*.ts` |
| Runtime-string Tailwind variants inside `tw()` | Normal component with `cva`/`cn()` |
