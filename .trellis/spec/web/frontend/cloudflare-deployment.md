# Cloudflare Deployment

> The app deploys to a Cloudflare Worker via OpenNext. The moving parts:
> `web/wrangler.jsonc` (bindings), `web/open-next.config.ts` (cache topology),
> and `web/worker.ts` (custom entrypoint with edge caching and cron).

---

## The Worker (`web/wrangler.jsonc`)

- **The top-level config IS production.** The Worker is named
  `tokens-staging` for historical reasons, but tokens.ci is bound to it and
  the database behind it was loaded from the production dump — every deploy of
  this config goes straight to live traffic. The `env.production` block is
  unused (a second Worker named `tokens` that was never created); do not read
  it as "the real one".
- `compatibility_flags: ["nodejs_compat", "global_fetch_strictly_public"]` —
  the Postgres driver, `node:crypto`, and the Next.js runtime need Node
  built-ins; the second flag makes OpenNext's server-side fetches go over the
  public internet instead of looping in-process.
- **Placement is pinned `targeted` to `aws:us-west-2`**, the database's
  region. Pages issue several sequential queries and measured edge placement
  spent 96% of wall time waiting on the database (p50 218ms vs 8ms CPU).
  `targeted`, not `smart`: smart needs sustained multi-region traffic and the
  answer is already known. This lives in config because wrangler treats the
  file as source of truth and overwrites dashboard settings.
- **Hyperdrive** points at Neon's *direct* endpoint, not the `-pooler` one —
  Hyperdrive already pools, and stacking it on PgBouncer adds a hop for
  nothing. Query caching (`max-age 60, swr 30`) and
  `origin_connection_limit` live on the Hyperdrive config, set via
  `wrangler hyperdrive update`.
- `WORKER_SELF_REFERENCE` service binding lets the ISR queue re-invoke the
  Worker to regenerate pages; the service name must equal the worker name.

## Cache topology (`web/open-next.config.ts`)

Every read goes through `unstable_cache` (60s revalidate) and every accepted
submission fans out `revalidateTag` calls, so the topology is chosen for
write-heavy invalidation:

| Concern | Override | Why |
|---------|----------|-----|
| Incremental (ISR/`unstable_cache`) payloads | R2 (`NEXT_INC_CACHE_R2_BUCKET`) | Cheap, strongly consistent, unbounded vs KV |
| Revalidation queue | `DOQueueHandler` Durable Object | Dedupes concurrent time-based revalidations — one regeneration per cache miss, not one per request |
| Tag cache | `DOShardedTagCache` (`baseShardSize: 12`) | Sharded so concurrent submits invalidating overlapping tags don't serialize on one DO; D1 variant is for lighter loads |
| Edge cache purge | `purgeCache({ type: "durableObject" })` via `BucketCachePurge` | Batches CDN purges per invalidated tag; needs no zone API token |

## Custom entrypoint (`web/worker.ts`)

Wraps `./.open-next/worker.js` for two reasons:

1. **Explicit edge caching.** Cloudflare does not put Worker responses in the
   edge cache on its own — `Cache-Control` headers are honored by browsers and
   nothing else. The worker reads/writes `caches.default` for:
   - `/api/og`, `/api/embed/*/svg`, `/api/badge/*/svg` (`CACHEABLE`) — pure
     functions of their URL that cost real CPU/DB work;
   - `/` and `/leaderboard`, `/shame` (`PAGE_CACHEABLE`) — signed-out readers
     only, because these pages personalize from the session;
   - `/u/*` (`PROFILE_CACHEABLE`) — cacheable for everyone (no per-reader
     identity), with unknown query params dropped from the cache key and only
     200s stored (the case-canonicalizing 308 must not be cached).
2. **The daily cron** (`20 3 * * *`) runs `refreshAllSocialLinks` in-process
   for the verified-badge refresh — no public endpoint, no `CRON_SECRET`
   round trip, no WAF exception for GitHub runner IPs.

## Commands

| Command | Purpose |
|---------|---------|
| `bun run cf:build` | asset copies + `opennextjs-cloudflare build` |
| `bun run cf:preview` | build + local Cloudflare preview |
| `bun run cf:deploy` | build + `wrangler deploy` (**production**) |
| `bun run cf:typegen` | regenerate `cloudflare-env.d.ts` from bindings |
| `bun run dev` | `next dev` with CF bindings via `initOpenNextCloudflareForDev` |

## Gotchas already paid for

- `revalidate` + `searchParams` on Workers flaps between 200/500 — use
  `force-dynamic` (shame page).
- `next/image` optimization is off (`images.unoptimized: true` in
  `next.config.ts`) — use static assets and plain `<img>` for avatars.
- Don't create per-request DB pools across requests — see
  [Database Guidelines](./database-guidelines.md).
