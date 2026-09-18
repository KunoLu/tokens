# T11 wrap remaining copy

Parent implement.md T11, FR-8, and `docs/TODO.md` T11 are the source of truth. T10 already shipped `t()` + `web/src/lib/i18n/t.ts` dictionaries (not per-page files). T11 wraps remaining hardcoded UI strings into that file.

## Book Gate Plan

| Skill | Trigger | Phase | State |
|---|---|---|---|
| book-ddd-distilled-modeling | no new domain terms | — | not-required |
| book-ddia-data-design | no schema / cache-key change (T10 already put locale in cache keys) | — | not-required |
| book-legacy-change-safety | existing pages' English rendering | before first wrap | passed |
| book-refactoring-pass | existing production JSX | before those edits | passed |
| book-release-readiness | user-visible copy on production pages | after validation | planned |

grill-with-docs: skipped (logged in `grill.log`). Post-grill DDD not required.

## Change boundary

- In: remaining hardcoded UI strings in Leaderboard, Teamboard leftovers, Teams leftovers, Profile leftovers, Settings, Docs, Privacy, Terms, auth pages, device, empty/error, toast, footer. Privacy/Terms zh note 「英文版本为准」.
- Out: new i18n library; URL locale prefix; `users.locale`; email template translation; CLI; T9 docs/upstream_policy; schema.

## DDD Boundary Review

Not required. No new Team/Group/auth terms. Copy wrapping only.

## Legacy Change Safety Review

Status: characterized

Reviewed 2026-09-15 against T11 target pages (not inherited from T8/T10).

Behavior to change: wrap remaining user-visible hardcoded English chrome into `t()` keys so zh follows `tt_locale`.

Behavior to preserve:
- Leaderboard chrome currently hardcoded in `web/src/components/leaderboard/Leaderboard.tsx`: `PERIODS` labels All time/Today/Week/Month/Last month (L55-60); PageHeader "Leaderboard" / "AI coding token usage, reported by the Tokens CLI." (L372-374); Stat "Developers" / "Your rank" (L393-400); aria-label Period/Sort by/Search developers/Totals/Pagination; placeholder "Search…"; Toggle "Tokens"; heads Developer/Usage/Tokens/Cost; empty "No developers found" / "Nothing recorded"; Previous/Next. Uses `useFormat` not `useI18n`. Default-English after wrap must match these strings character-for-character. Period query values (`all`/`today`/…) stay English URL tokens.
- Docs currently hardcoded in `web/src/app/(main)/docs/page.tsx`: metadata "Docs - Tokens" (L16-20); PageHeader "Docs" / "Get your AI coding usage onto the leaderboard from the terminal." (L130-131); sections "Install the CLI", "Everyday use", "Supported clients" (L137-207). Install commands, client product names (Orca, …), and brew one-liners stay literal.
- Privacy / Terms: `web/src/app/(main)/privacy/page.tsx`, `terms/page.tsx`, `components/legal/LegalPage.tsx` — no `t()` today. `en` dictionary values must equal current English source. zh adds 「英文版本为准」.
- Emails stay English. Brand "Tokens", usernames, numbers, CLI commands unwrapped.
- T10 nav/`html lang`/`tt_locale`/worker `__locale` unchanged. T5/T7/T8 already-keyed strings unchanged.

Current reproduction evidence: Leaderboard.tsx has no `useI18n` import; PERIODS labels and PageHeader are string literals. docs/page.tsx Section titles are string literals. Privacy/Terms grep for `t(` / `useI18n` returned no matches.

Safety net: `web/features/i18n.feature` + `tests/e2e/i18n-locale.spec.ts` (LocaleToggle). After wrap: `rg` JSX text nodes; cheap Playwright copy assertions for Privacy/Terms 英文为准; existing LocaleToggle still green.

Hidden dependencies: `PERIODS` is exported — wrapping `label` must not change `value`. Teamboard reuses Leaderboard period/sort chrome; wrap shared constants once.

Validation: `bun run lint` + `typecheck` + native `bun run test:e2e`.

Review mode: normal.

## Refactoring Review

Status: proceed

Review mode: normal

Existing-code scope: `Leaderboard.tsx` chrome literals; `docs/page.tsx` headers/sections; `LegalPage.tsx` / privacy / terms; SettingsClient / auth / device / error leftovers; `t.ts` key appends only.

Behavior that must remain unchanged: listed above. Period/sort URL values. Docs command blocks. Client logos/names.

Structural friction: none worth extracting first. T10 already chose a single `t.ts`; splitting per-page dictionaries would be a second convention.

Decision: no refactor first. Wrap literals in place. `PERIODS[].label` via `t()` at render, keep `value` as the URL token. Do not rewrite Leaderboard layout, Docs Tabs, or LegalPage structure. Do not add next-intl.

Safety net: i18n.feature + LocaleToggle spec + post-wrap `rg`.

Deferred refactors: none.

## Ponytail

Reuse `t()` / `useI18n()`. One file `t.ts`. No next-intl. Emails stay English literals.

## Copy contract

- `en` values for Docs / Privacy / Terms = current English source, character-for-character.
- Privacy / Terms zh pages show 「英文版本为准」 / "English version prevails".
- Emails: English only.
- Do not wrap CLI, schema comments, or log strings.

## After implement

trellis-check, then `/review`. Fix every finding. Then Phase 3.4. No commit from implement. No merge to main.
