# Hook Guidelines

> Hooks live in `web/src/lib/use*.ts`. There is intentionally no
> `src/hooks/` directory — `components.json` still declares an `@/hooks` alias
> from the shadcn scaffold, but it is unused. Do not create `src/hooks/`.

---

## The reference hook: `useSettings`

`web/src/lib/useSettings.ts` is the pattern for cross-route client
preferences:

- `"use client"` module exporting `useSettings()`.
- Backed by `useSyncExternalStore` over `localStorage`
  (`STORAGE_KEY = "tokens-settings"`), with a custom event
  (`tokens-settings-changed`) so all hook instances re-render together.
- Reads a legacy key (`tokscale-settings`) once and migrates it, so renames do
  not reset user preferences.
- Mirrors `leaderboardSortBy` into a cookie (`SORT_BY_COOKIE_NAME`) so the
  server can render with the same sort — localStorage alone is invisible to
  RSC.

## Conventions

- **Server components never use hooks.** Data comes from async server
  components and `lib/` loaders; hooks are for client interaction state.
- **Hydration-safe dates**: values that depend on the viewer's clock/timezone
  are set in `useEffect` after mount, not during render — see
  `viewerLocalDate` in `app/u/[username]/ProfilePageClient.tsx` — to avoid
  hydration mismatches.
- **Theme** goes through `useTheme()` from `next-themes` (e.g.
  `components/layout/Navigation.tsx`).
- **Navigation** uses `useRouter` from `nextjs-toploader/app` (not
  `next/navigation`) so the top loader fires.
- Keep hooks generic and small; data loading belongs in RSC + `lib/`, not in
  client `useEffect` fetch chains (see [Data Fetching](./data-fetching.md)).
