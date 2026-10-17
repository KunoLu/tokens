# Vercel deploy (Hobby)

[中文](./README.md)

Project name: **`kunolu-tokens`**. Until a domain is purchased, use the free address **`https://kunolu-tokens.vercel.app`**. This path is separate from Docker/self-host. `docs/deploy/docker/prod/` is unchanged. Self-host defaults stay the in-process limiter, pool max 5, and a system cron `POST`.

## Project settings

| Setting | Value |
|---|---|
| Root Directory | `web` |
| Include files outside of the Root Directory in the Build Step | **enabled** |
| Framework | Next.js |

The build copies `install.sh`, `pre-install-tokens.sh`, `pre-install-tokens.ps1`, `enable-tokens-service.sh`, `register-tokens-submit-task.ps1`, and `.github/assets/client-*` from the repo root. With Root Directory `web`, turning that toggle off drops those files from the build and `bun run build` fails. `web/vercel.json` `installCommand` also expects the parent `bun.lock`.

## Neon: separate Preview and Production branches

- Production and Preview each get their own Neon branch. Preview `DATABASE_URL` must not point at the production branch.
- Prefer the Neon–Vercel integration: each Preview deployment forks its own branch from production (schema as of the fork) instead of connecting to the production database.
- Runtime `DATABASE_URL` is the **pooled** connection string (the `-pooler` host). `drizzle-kit migrate` needs the **direct / unpooled** connection; session features on the pooler make migrations fail. The Neon–Vercel integration sets `DATABASE_URL_UNPOOLED`.
- Leave `DATABASE_SSL` unset or set `require`. Do not set `disable` (Neon needs TLS; the code defaults to `require` in production when the variable is unset).
- **`DATABASE_POOL_MAX=1`** on both Production and Preview. The code default is 5, for the long-lived Docker process (clamped 1..5). Each Vercel isolate opens its own pool; the default of 5 exhausts Neon connections. Do not change the code default.

## Migrations only on Production deploys

`web/vercel.json` `buildCommand`:

```sh
if [ "$VERCEL_ENV" = "production" ]; then
  DATABASE_URL="${DATABASE_URL_UNPOOLED:-$DATABASE_URL}" bun run db:migrate
fi && bun run build
```

- `bun run db:migrate` (`drizzle-kit migrate`) runs only when `VERCEL_ENV=production`. Preview builds do not migrate.
- Migrate prefers `DATABASE_URL_UNPOOLED`, then `DATABASE_URL`.
- Production `DATABASE_URL` / `DATABASE_URL_UNPOOLED` must be available **at build time**, or the production build fails at migrate. Do not put the production connection string in Preview env.
- A long-lived Preview branch does not receive production migrations automatically. Use the integration's per-deploy fork, or re-fork from production after a migration. Do not run `db:migrate` against Preview, and do not run `test:migrations` against a cloud database.

## Environment variables

Set these on both Production and Preview (values differ by branch):

| Variable | Notes |
|---|---|
| `NEXT_PUBLIC_URL` | Until a domain is purchased: `https://kunolu-tokens.vercel.app`. If the dashboard shows a different `*.vercel.app` host, use that host; do not guess a team suffix |
| `DATABASE_URL` | Pooled connection string for that environment's Neon branch |
| `DATABASE_URL_UNPOOLED` | Direct connection string. The production build migrate uses it |
| `DATABASE_POOL_MAX` | `1` |
| `CRON_SECRET` | Random string, at least 16 characters. Vercel Cron sends it as `Authorization: Bearer`. Manual POST retries use the same secret |
| `UPSTASH_REDIS_REST_URL` | Upstash Redis REST URL |
| `UPSTASH_REDIS_REST_TOKEN` | Upstash Redis REST token |

Optional: `RESEND_API_KEY` / `EMAIL_FROM`, `CONTACT_EMAIL`, `GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET`.

### Upstash

Both variables must be non-empty to leave the in-process limiter. Setting only one keeps the in-process implementation (same as the self-host default: 10 requests / 60 seconds / client IP). On Vercel the in-process map lives in one isolate and does not limit across instances, so set both on Production and Preview.

Redis keys use the prefix `tokens:auth:${VERCEL_ENV}`, or `tokens:auth:local` when `VERCEL_ENV` is unset. Preview and Production share one Upstash Redis; the env segment keeps their buckets apart.

When Upstash is configured, a Redis error or timeout **fails closed** (that auth request is denied). It does not fall through to the process map, which would give each isolate its own budget. Do not set these on the Docker compose stack. That path leaves them unset and keeps the in-process limiter.

### Client IP

The platform sets `VERCEL=1`. The rate-limit key then prefers the first hop of `x-vercel-forwarded-for` (written by Vercel; a proxy in front of Vercel can rewrite `x-forwarded-for`), then `x-real-ip`, then the first `X-Forwarded-For` hop. `CF-Connecting-IP` is never honored.

Self-host sets neither `VERCEL` nor `TRUST_PROXY`, so the key stays the first `X-Forwarded-For` hop. `TRUST_PROXY=1|true|yes` enables the same platform-header order as Vercel. Do not set it on the Docker stack.

## Cron

`web/vercel.json` schedules `GET /api/cron/refresh-social-links` at `17 3 * * *` (once per day). Hobby rejects expressions that run more than once per day. Timing precision is hourly (±59 minutes).

GET and POST share the Bearer check and the three jobs (social-link refresh, expired email tokens, invitation expiry). All succeeded → 200; any failed → 500; secret unset → 503; wrong secret → 401. Self-host system cron still `POST`s to loopback. Vercel Cron uses GET and does not retry failures; check the function logs.

## Custom domain

Until a domain is purchased, use the free address `https://tokens-blush.vercel.app` (project name `tokens`). Set `NEXT_PUBLIC_URL` to that address and deploy once.

Changing the domain later:

1. Attach the domain on the Vercel project `tokens`.
2. Set `NEXT_PUBLIC_URL` to `https://<new-domain>`.
3. **Redeploy.** `NEXT_PUBLIC_*` is inlined at build time. Changing the env var without a new build leaves device links, verification mail, invite links, the CSRF origin allowlist, OG cards, and embeds on the old origin.

Also point `TOKENS_API_URL` on user machines at the new domain. Credentials do not store the API base, so users who logged in against the old host must run `tokens login` again.

## Manual Production deploys

`web/vercel.json` sets `"git": { "deploymentEnabled": { "main": false } }`: pushes to `main` no longer go live, while PR and other branch Preview deploys continue. Trigger Production by hand with `vercel deploy --prod`, or create a deployment through the API with an explicit branch and SHA. Dashboard Redeploy only rebuilds the old deployment's commit. After deploying, check that the live deployment's SHA matches `main`.
