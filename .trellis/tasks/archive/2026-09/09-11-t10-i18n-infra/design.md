# T10 i18n infrastructure

Parent implement.md T10 and docs PRD FR-8 are the source of truth.

## Book Gate Plan

| Skill | Trigger | Phase | State |
|---|---|---|---|
| book-ddd-distilled-modeling | on-demand | — | not-required |
| book-ddia-data-design | cache keys include locale | before implement | passed |
| book-legacy-change-safety | layout, Navigation, worker, format.ts | before first existing-file edit | passed |
| book-refactoring-pass | same files | before those edits | passed |
| book-release-readiness | worker cache + html lang | after validation | passed |

grill-with-docs: skipped. FR-8 already decides cookie, no URL prefix, no next-intl.

## Change boundary

- In: `lib/i18n/`, `tt_locale` cookie, `<html lang>`, nav globe toggle, worker `__locale` cache key, profile `unstable_cache` locale, `formatNumber`/`formatCurrency` optional `locale` (default `en`).
- Out: wrapping remaining page copy (T11). Mail stays English.

## Format locale

No module-global locale. `formatNumber(value, compact, locale = "en")` uses `intlTag(locale)` (`en` → `en-US`, `zh` → `zh-CN`). Leaderboard client rows call `useFormat()`. Compact currency still prefixes `$` (not `style: currency`) so OG/embed keep a dollar sign; non-compact currency uses `style: currency` and will show `US$` in zh-CN. Column alignment uses `.tabular`. Relative-time copy is T11.

## Cache

`worker.ts` `pageCacheKey` / `profileCacheKey` append `__locale` from `tt_locale`. `loadPublicProfileForPage` cache key includes locale. Leaderboard data cache is numbers, not copy — unchanged.

## Release Readiness Review

Status: ready

Production path and affected users / systems: signed-out HTML for `/`, `/leaderboard`, `/teamboard` via `worker.ts` `caches.default`; public profile `unstable_cache`; root `<html lang>` and nav chrome; leaderboard number formatting.

Failure modes and safeguards: malformed `tt_locale` is `parseLocale`d to `en` so cache keys cannot fragment; locale variants are isolated (`__locale=en|zh` only). NumberFormat follows `intlTag(locale)`.

Capacity / backpressure / limits: two extra cache variants (en, zh), not unbounded. Cookie is not HttpOnly (client toggle); SameSite=Lax; Max-Age 1y.

Observability / alerts / runbook: none new. Missed locale isolation would show mixed nav copy; rollback is revert worker + layout + i18n module.

Rollout / migration / rollback / cleanup: additive cookie + cache key. No schema. Rollback: revert commit; old cache entries expire.

Required validation and result:
- `bun run lint`: 0 errors, 2 pre-existing warnings (`docs/page.tsx` no-img-element; `worker.ts` anonymous default export)
- `bun run typecheck`: pass
- `bun run cf:build`: OpenNext worker saved after intlTag + Leaderboard `useFormat` (earlier T10 pass)
- preview HTTP (pinned OpenNext 1.20.2, then stopped): `GET /leaderboard` en/zh/zh-CN isolated; empty leaderboard so no compact `万`/`US$` in HTML
- ego-browser LocaleToggle: click Language → 中文 → `lang=zh-CN`, cookie `tt_locale=zh`, nav 排行榜/团队榜/文档
- native `bun run test:e2e` (rtk skipped-for-report): 4 passed (8.5s). Formal reports: `tests/e2e/reports/html/playwright-report-i18n-feature_teamboard-teams-auth-2026_09_14-17_27_37.html` + same-stem `.md`

Optional checks, accountable owner acceptance, and residual risk: Playwright residual cleared 2026-09-14. `@playwright/test` is a `web/` devDependency; `web/playwright.config.ts` runs `tests/e2e/i18n-locale.spec.ts` against `bun run dev` (port 3000). HTML reporter temp is `tests/e2e/reports/.playwright-html-current/`; named HTML+md live under `tests/e2e/reports/html/`. Four LocaleToggle click/persist scenarios in `web/features/i18n.feature` are not `@todo`; T11 remaining-copy / Privacy / Terms stay `@todo`. NumberFormat E2E and worker `__locale` isolation are not covered by this Playwright run (worker isolation was preview HTTP). Remaining page copy is T11. Dirty worktree: local-only evidence.

