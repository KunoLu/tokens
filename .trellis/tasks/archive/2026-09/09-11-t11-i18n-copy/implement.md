# T11 implement

Active task: `.trellis/tasks/09-11-t11-i18n-copy`

## Steps

1. Inventory remaining hardcoded UI strings (JSX text, `aria-label`, toast, PageHeader, empty/error). Skip already-keyed T5/T7/T8/T10 strings.
2. Append `en` / `zh` keys to `web/src/lib/i18n/t.ts`. Docs/Privacy/Terms `en` values must match the current English source verbatim.
3. Replace those strings with `t()` / `useI18n()`. Privacy/Terms zh: annotate 「英文版本为准」.
4. Leave email templates English. Do not add `users.locale`.
5. `rg` JSX text nodes for leftover user-visible English/Chinese literals (allow proper names, brand "Tokens", format numbers).
6. Playwright: LocaleToggle already covers nav. Add cheap copy assertions for Privacy/Terms 英文为准 and one wrapped page if the existing i18n spec can take them.

## Do not

- next-intl / URL locale prefix / per-page dictionary files
- Translate emails
- Commit, archive, start T9, merge main

## Acceptance

No leftover user-visible hardcoded copy on listed pages. Docs/Privacy/Terms default-English unchanged. Privacy/Terms zh has 英文为准. lint + typecheck + native e2e. Then `/review` and fix every finding.

## Done

Wrapped remaining app/component copy through `web/src/lib/i18n/t.ts`. Client uses `useI18n().t`; RSC uses `cookies()` + `parseLocale` + `t(locale, key)`. `zh` is `Record<keyof typeof en, string>`. Emails stay English. No `getDictionary` / next-intl / URL locale prefix.

BDD: `web/features/i18n.feature` T11 `@todo` scenarios replaced with executable full-page copy, Privacy/Terms English-precedence, and authenticated Settings copy. Trace: `tests/e2e/i18n-locale.spec.ts`.

Checks: `bun run lint` passed (2 pre-existing warnings); `bun run typecheck` passed; native `bun run test:e2e -- i18n-locale.spec.ts` 8/8 after adding Settings. `rtk`: skipped-for-report for E2E; used for lint/typecheck.

Reports (named pair, latest full rerun):

- `tests/e2e/reports/html/playwright-report-i18n-feature-teamboard-teams-auth-2026_09_15-11_16_36.html`
- `tests/e2e/reports/html/playwright-report-i18n-feature-teamboard-teams-auth-2026_09_15-11_16_36.md`

Long-term: `.trellis/spec/web/frontend/quality-guidelines.md` § UI copy (i18n); component-guidelines pointer.

Not done here: commit, archive, T9, merge main.

## P1 TranslationKey gate note (2026-09-15)

Legacy Change Safety Review for the P1 missing-key fix: status **characterized**. Behavior to change: six nonexistent keys (`graph.across`, `graph.clientsCountOne/Many`, `graph.tokensInline`, `graph.activeDaysOne/Many`) rendered as raw key strings via the `?? key` fallback; now pointed at existing `graph.acrossClientsOne/Many`, `tokens.count`, `profile.graph.activeDayOne/activeDaysMany`, plus `TranslationKey = keyof typeof en` typing on `t`/`Translate` so misses fail typecheck. Preserved: English copy verbatim, zh values, no getDictionary/next-intl, emails English, no P2 scope. Safety net: existing i18n-locale E2E (8/8), zh already `Record<keyof typeof en, string>`, post-change key-existence script over all call sites. Helper signatures taking `(key: string) => string` widened to shared `Translate` (type-only). No seam required. Validation: scoped key script; project-wide typecheck by main session.

## P2 remaining-copy gate note (2026-09-15)

Legacy Change Safety Review
Status: characterized
Behavior to change: graph/profile/local UI numbers still default to en; legal/banned/graph/today/chart dates still English-literal or YYYY-MM-DD on zh; social ARIA, palette labels, relative time, reset-password zh CTA, DeviceClient/DataInput English error fallbacks, zh `teamboard.sortCost`, and E2E reset-password heading/copy collision.
Behavior to preserve: English source copy verbatim (including legal "25 July 2026"); embed/SVG/CLI still unlocalized; `formatCurrency`/`formatNumber` locale remains optional; emails English; TranslationKey from P1; no getDictionary/next-intl/URL prefixes.
Current reproduction evidence: Graph/profile/local call `formatCurrency`/`formatTokenCount`/`formatNumber` without locale; LegalPage interpolates hardcoded `25 July 2026`; TokenGraph3D dateRange is `MM/DD`; ProfileToday/UsageChart tooltip pass raw ISO; `formatRelativeTime` English-only; zh `teamboard.sortCost` is still "Cost"; reset-password E2E heading===copy.
Safety net: existing `tests/e2e/i18n-locale.spec.ts` (reset-password row made distinct); scoped node script for relative-time + legal date + new keys. No new production seam.
Hidden dependencies / seam: none. Embed still imports `format.ts` without locale.
Validation plan: scoped node script; E2E row uniqueness; main session typecheck.
Review mode: normal

Refactoring Review
Status: proceed
Review mode: normal
Existing-code scope: listed P2 UI files + `t.ts` key appends + optional-locale helpers.
Behavior that must remain unchanged: English copy; embed/SVG number format; locale optional on shared formatters.
Structural friction: none worth extracting. Reuse `t()` / `useI18n()` / `useFormat()` / `intlTag`.
Decision: no refactor first. Pass `locale`/`useFormat()` at UI sites; dictionary keys for palettes/website/HTTP error; locale-aware `formatRelativeTime` without importing `t.ts` into embed.
Safety net: i18n-locale E2E reset-password row + scoped script.
Deferred refactors: none.
