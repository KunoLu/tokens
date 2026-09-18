# Self-host Deployment

> The app self-hosts on a Node server: `next build` + `next start`, with
> Postgres over `DATABASE_URL`. The Cloudflare Workers layer (wrangler.jsonc,
> OpenNext cache topology, `worker.ts` edge cache + cron) was removed in the
> self-host cutover — this file is its replacement.

---

## Runtime (`web/`)

- `bun run build` — copies `install.sh`/client assets into `public/`, then
  `next build`. No secrets and no `DATABASE_URL` are required at build time:
  pages that touch the database are server-rendered on demand, and the
  leaderboard tolerates a missing `DATABASE_URL`.
- `bun run start` — the production process. Required env:
  `NODE_ENV=production`, `DATABASE_URL`, `NEXT_PUBLIC_URL=https://<origin>`
  (device-flow links, email links, invite links, and the CSRF origin list all
  derive from it), and `DATABASE_SSL=disable` unless the database has TLS
  (production defaults to `require`).
- Serve HTTPS through a reverse proxy in front of `next start`. The CLI
  device flow rejects non-loopback HTTP verification URLs, and production
  session cookies are `Secure`.
- `web/middleware.ts` runs the `/settings` session gate; everything else is
  route-level.

## Database connection (`web/src/lib/db/index.ts`)

- One process-wide pool (singleton on `globalThis`, HMR-safe in dev). Default
  `max=5`, overridable with `DATABASE_POOL_MAX` (clamped 1..5).
- TLS: `DATABASE_SSL` (`disable`/`require`); production defaults to
  `require`.
- `prepare: false` — `max_lifetime` recycles connections and prepared
  statements are connection-scoped.

## Daily maintenance cron

There is no managed scheduler. Run the maintenance endpoint from the same
host, on a schedule, against loopback so no public proxy timeout can cut a
run:

```
POST http://127.0.0.1:3000/api/cron/refresh-social-links
Authorization: Bearer $CRON_SECRET
```

- 401 without/with a wrong secret; 503 when `CRON_SECRET` is unset.
- Awaits all three jobs (social-links refresh, expired email-token sweep,
  invitation expiry). 200 with per-job counts only when all succeeded; 500
  when any failed — the scheduler retries on non-2xx. All three are
  idempotent sweeps, so retries are safe.

## Rate limiting

`authRateLimitAllowed` is an in-process fixed window (10 requests / 60s per
client IP) on register, login, forgot-password, and resend-verification. The
client key is `X-Forwarded-For` ONLY, and only because the reverse proxy
overwrites it — never honor `CF-Connecting-IP` on this path, or a direct
client can forge its way into fresh buckets. Counters are per-process —
horizontal scaling needs a shared store.


## Caching

Next's default cache handler (`.next/cache` filesystem + in-memory) backs
`unstable_cache` and `revalidateTag`/`revalidatePath`. There is no edge HTML
cache: every request renders at origin. Put a CDN in front if that becomes
the bottleneck.

## Email

Resend over HTTPS. Without `RESEND_API_KEY` / `EMAIL_FROM` sends are logged
and skipped — register/forgot still succeed; team email invitations are
effectively disabled (invitees never see mail, and email-targeted invites
require a verified address at accept time).

## Gotchas already paid for

- `revalidate` + `searchParams` on pages flapped between 200/500 in the
  Workers era; pages that depend on `searchParams` use
  `export const dynamic = 'force-dynamic'` (kept — harmless and safer).
- `next/image` optimization is off (`images.unoptimized: true` in
  `next.config.ts`) — static assets and plain `<img>` for avatars.
- Don't create per-request DB pools — see
  [Database Guidelines](./database-guidelines.md).
