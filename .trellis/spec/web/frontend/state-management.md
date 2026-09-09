# State Management

> No global client store. State lives in four places, chosen deliberately:
> the server (RSC `initialData`), the URL, localStorage (+cookie mirror), and
> local `useState`.

---

## The four homes for state

| Kind | Mechanism | Example |
|------|-----------|---------|
| Server data | Async server component loads via `lib/`, passes `initialData` to the client shell | `app/u/[username]/page.tsx` → `ProfilePageClient initialData/initialDevices`; leaderboard page → `LeaderboardClient initialData` |
| Shareable view state | URL query string (`useRouter` + `useSearchParams`) | Profile period: `onPeriodChange` → `router.push('/u/...?period=')`; leaderboard period/sort/search in the query string |
| Cross-route user preferences | `useSettings()` → localStorage, with `leaderboardSortBy` mirrored to a cookie for SSR | `web/src/lib/useSettings.ts` |
| Ephemeral UI state | Local `useState` | Modals, embed dialog, contribution selection, debounced search input |

## Rules

1. **If it should survive a share/reload, it goes in the URL.** Filters,
   periods, and sorts are query params, not component state — that is what
   makes leaderboard and profile views linkable.
2. **Do not duplicate server data into client state.** Client components
   receive `initialData` and re-render from server navigations; there is no
   client-side cache layer (no SWR/React Query) to keep in sync.
3. **Preferences that affect SSR need the cookie mirror.** localStorage is
   read client-side only; anything the server must render consistently (sort
   order) is mirrored to a cookie, as `useSettings` does for
   `leaderboardSortBy`.
4. **`useState` is for UI-only concerns.** If closing the tab may lose it,
   it is `useState`; otherwise it belongs in one of the other three homes.
5. **Session state comes from `/api/auth/session`** — fetched by the
   components that need it (`components/layout/Navigation.tsx`,
   `app/device/DeviceClient.tsx`), not held globally.
