# Quality Guidelines

> Hard rules for `web/`. There are **no web unit tests** — verification is
> lint, typecheck, build, the migration check, and (when UI behavior is in
> scope) Playwright E2E. Quality is enforced by convention and review.

---

## Verification (the only gates)

| Command | Gate |
|---------|------|
| `bun run lint` | ESLint (next core-web-vitals + typescript) |
| `bun run typecheck` | `tsc --noEmit` |
| `bun run build` | asset copies + `next build` |
| `bun run test:migrations` | `drizzle-kit migrate` + `scripts/check-migrations.ts` |
| `bun run test:teams` | T4 domain invariants + T7 FR-2 Teamboard pagination / multi-value `groupIds` + T8 `listMyInvitations` + T9 §12 (creator admin, rename/avatar, subadmin cap **and** remaining Team/Group ops, public→private Teamboard, delete-after-disband) (`scripts/check-teams-invariants.ts`). Needs `DATABASE_URL` (local OrbStack `tokens-postgres` is `postgresql://tokens:tokens@127.0.0.1:5433/tokens`). |
| `bun run test:e2e` | Playwright LocaleToggle, full-page copy, Privacy/Terms English-precedence, Teamboard, Profile membership, maintenance-cron auth (`tests/e2e/cron.spec.ts`, `web/features/maintenance-cron.feature`), and T9 acceptance (`tests/e2e/t9-acceptance.spec.ts`: `/shame` 404, Leaderboard column order, Docs remaining vs removed sections, banned profile + login 403, unsigned Settings→login, embed/badge SVG 200, archive POST 401). Specs live at repo-root `tests/e2e/`; config is `web/playwright.config.ts`. Owner approved 2026-09-14. Locale copy coverage is `tests/e2e/i18n-locale.spec.ts` (`web/features/i18n.feature`). Leaderboard `thead th` includes a mobile `Usage` cell (`sm:hidden`); column-order assertions use `thead th:visible` under the Desktop Chrome project — do not delete that header or filter it out of the expected sequence. |

Do not add a general **unit-test** framework without a team decision — the upstream tests were
deliberately removed (`docs/upstream_policy.md`). `test:teams` is the T4 exception
for INV/permission checks, the T7 exception for Teamboard loader regressions
(51-member pagination, multi-value `groupIds`), the T8 exception for `listMyInvitations`,
and the T9 exception for §12 domain evidence (subadmin positive ops need more than
cap/forbid/disband). Playwright E2E is the T7/T8/T9/T10/T11 exception for user-visible
browser journeys; do not add `*.test.ts` under `web/`. Reports under
`tests/e2e/reports/` are gitignored runner output. Formal Playwright HTML is a
named `playwright-report-*.html` next to the same-stem `.md` under
`tests/e2e/reports/html/`.

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

## UI copy (i18n)

User-visible JSX, `aria-label`, toast, empty/error, and form labels go through
`web/src/lib/i18n/t.ts`.

- Client: `useI18n().t(key, vars?)`.
- Server: `parseLocale((await cookies()).get(LOCALE_COOKIE)?.value)` then
  `t(locale, key, vars?)`.
- `zh` is typed `Record<keyof typeof en, string>` so missing keys fail
  `bun run typecheck`.
- English dictionary values match the previous rendered English copy
  verbatim. Do not invent `getDictionary`, next-intl, or URL locale prefixes.
- Email templates stay English. Do not add `users.locale`.
- Leave CLI commands, vendor/product identifiers, format masks
  (`XXXX-XXXX`), and generated embed/SVG output literal.
- Privacy / Terms Chinese pages must keep `legal.enPrevails`
  (「本页内容以英文版本为准。」).
- Dates and numbers follow `tt_locale` through `intlTag` or `useFormat`.
  Do not call bare `toLocaleDateString()` / `toLocaleString()` without a
  locale tag. Shared `formatCurrency` / `formatNumber` from `@/lib/utils`
  take `locale` as the last argument; `useFormat()` already binds locale
  and its second argument is compact, not locale. Leave embed/SVG/OG
  output on the English default. `format.ts` must not import `t.ts`:
  embed/SVG/OG renderers import `format.ts` for compact numbers and XML
  escape. Dictionary-backed relative time lives in
  `formatRelativeTime.ts`. Do not re-export it from `format.ts` or `utils.ts`.


## Upstream policy (Tokscale fork)

`docs/upstream_policy.md` is the rulebook for merging from upstream:

- **Never merge**: branding/naming/copy (`tokscale` strings, logos), frontend
  styling/components/layout, TUI/report-command features, upstream `groups`
  tables (this fork's `groups` belong to `teams`), tests.
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
| `revalidate` + `searchParams` on a page | `export const dynamic = 'force-dynamic'` |
| HeroUI / styled-components | shadcn + Tailwind tokens + `tw()` leftover only |
| `next/image` optimization | Disabled; static assets + plain `<img>` |
| Assuming `@/hooks` exists | Hooks live in `lib/use*.ts` |
| Runtime-string Tailwind variants inside `tw()` | Normal component with `cva`/`cn()` |
