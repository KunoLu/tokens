# Error Handling

> Errors are part of the UI, not an afterthought: a root boundary for render
> failures, JSON error bodies from API routes, inline Alerts for recoverable
> states, toasts for mutation results, and graceful degradation when the
> database is absent.

---

## Patterns

### Root error boundary

`web/src/app/error.tsx` is a client boundary rendering **inside** the root
layout, so header/footer survive. `/leaderboard`, `/shame`, and
`/u/[username]` all do live database work during render, so any of them can
throw — without this boundary the user lands on Next's default error screen
with no way back. It offers the two useful actions (`reset()` and a link to
the leaderboard) and shows `error.digest` — the only support handle on a
specific failure, and it carries nothing sensitive. New routes that can throw
rely on this boundary; add a route-level `error.tsx` only if recovery differs.

### Not found

`app/u/[username]/not-found.tsx` handles unknown profiles with custom copy.
(Known inconsistency: it does not reuse `CONTAINER`/shared `Button` — match
the shared layout pieces when touching it.)

### API errors

Route handlers return `NextResponse.json({ error: '…' }, { status })` from
try/catch with `console.error` — e.g. `getPublicProfileResponse` maps
`AmbiguousUsernameError` to 409 and everything else to a generic 500
(`web/src/lib/publicProfileData.ts:671-684`). Never leak stack traces or
internal messages into the JSON body.

### Inline and toast feedback

- Recoverable page states use the shadcn `Alert` — profile resubmit banner,
  shame page, settings load failures.
- Mutation results use `toast.success` / `toast.error` (react-toastify via
  `ThemedToastContainer`) — see `app/settings/SettingsClient.tsx`.

### Graceful degradation

- **Missing `DATABASE_URL`** (local dev without a database): pages check
  `isMissingDatabaseUrl` and render empty data instead of throwing — e.g. the
  leaderboard renders an empty table (`app/(main)/leaderboard/page.tsx`).
- **Non-critical sections fail soft**: the profile's devices fetch is wrapped
  in try/catch and falls back to `[]` (`app/u/[username]/page.tsx`).

## Rules

1. A page that reads the database during render must be reachable through the
   root boundary — never let a render throw escape to the default screen.
2. API errors are typed `{ error: string }` JSON with a correct status code;
   clients narrow on `status`.
3. Distinguish "failed to load" from "empty/signed-out" in client fetches
   (the `DeviceClient` pattern).
4. Log server-side with `console.error` including the error object; the
   digest surfaces to the user, details stay in logs.
