# Authentication (`web/src/lib/auth/`, `app/api/auth/`)

> Email/password web login. Device flow and `tt_session` are unchanged.
> GitHub OAuth is gone. The verified-by-social-links badge is gone.

---

## Scenario: Email password auth

### 1. Scope / Trigger

- Trigger: new/changed auth API, password hashing, email delivery, `users`
  password columns, or login UI.
- Session cookie (`tt_session`), device-flow JSON, personal API tokens,
  `/api/submit`, and `/api/me/stats` stay as they are. Do not change
  `lib/auth/session.ts` signatures.

### 2. Signatures

- `hashPassword(plain: string): Promise<string>` /
  `verifyPassword(plain, stored): Promise<boolean>` in
  `lib/auth/password.ts`. Stored form:
  `pbkdf2$sha256$210000$<salt_b64>$<hash_b64>` (WebCrypto PBKDF2-HMAC-SHA256,
  210k iterations, 16-byte salt). Unknown prefix → `false`, do not throw.
  Strict parse: iterations field must match `/^[0-9]+$/` and equal 210000,
  salt must decode to 16 bytes, hash to 32 bytes — anything else → `false`.
- `validatePassword(plain)` — ≥ 8 chars, upper, lower, one special. Shared by
  register and reset.
- `issueEmailToken(userId, purpose)` /
  `consumeResetTokenAndSetPassword(token, passwordHash)` /
  `consumeVerifyTokenAndMarkVerified(token)` / `findEmailToken` /
  `deleteExpiredEmailTokens()` in `lib/auth/emailTokens.ts`.
  `purpose` ∈ `verify_email` | `reset_password`. DB stores SHA-256 hex only.
  Issue locks the user row (`FOR UPDATE`) and replaces old unconsumed tokens
  in one transaction — one active token per user+purpose. Consume commits in
  one transaction with the follow-up writes (reset: `password_hash` + delete
  `sessions`; verify: set `email_verified_at`); the route hashes the password
  BEFORE the transactional call.
- `sendVerificationEmail` / `sendPasswordResetEmail` in `lib/email/send.ts` —
  Resend HTTP `fetch`, `waitUntil`, never throws, never rolls back rows.
- `avatarUrlFor({ username, avatarUrl })` in `lib/avatar.ts` — stored URL or
  initials SVG data URI. Never `github.com/<user>.png`.
- `POST /api/auth/{register,login,forgot-password,reset-password,verify-email,resend-verification}`
- Pages: `/register`, `/login`, `/forgot-password`, `/reset-password`,
  `/verify-email`. Unauthenticated `/settings` → `/login?returnTo=`.
  Navigation "Sign in" uses `returnTo=/leaderboard` on the auth pages
  themselves (coming "back" would loop), the current pathname elsewhere;
  `/login`'s forgot-password link carries `returnTo` along.

### 3. Contracts

**Request (JSON unless noted)**

| Route | Body | Session |
|---|---|---|
| register | `{ email, username, password }` | none |
| login | `{ email, password }` | none |
| forgot-password | `{ email }` | none |
| reset-password | `{ token, password }` | none |
| verify-email | `{ token }` | none |
| resend-verification | empty | **required cookie** |

Mutating POSTs need `hasAllowedOrigin`. Missing/disallowed Origin → 403.
`AUTH_RATE_LIMITER` (wrangler `ratelimits`, ns `1001`, 10/60s) on register,
login, forgot-password, resend-verification. Missing binding (local
`next dev`) skips the limiter. The binding is duplicated into
`env.production` with the same stable `namespace_id` "1001" — wrangler env
blocks override rather than merge, and the id must never change per deploy.

Login with no user or no `password_hash` (legacy OAuth account) still runs
PBKDF2 against a constant dummy hash derived once per isolate, so timing does
not reveal registered emails; the 401 string is identical either way.

**Response:** `{ ok: true }` or `{ error: string }` / `{ error, details: string[] }`.

**Env (secrets via `wrangler secret put`, never in git):** `RESEND_API_KEY`,
`EMAIL_FROM`. Optional: `NEXT_PUBLIC_URL` for email links.

**DB (`0024_add_password_auth.sql`):** `users.password_hash`,
`users.email_verified_at`, `users.github_id` nullable (UNIQUE kept),
partial unique `users_email_lower_unique` on `lower(email) WHERE email IS NOT NULL`,
table `email_verification_tokens`.

Unverified users **may hold a session**. `email_verified_at` only records
that the emailed link was followed.

### 4. Validation & Error Matrix

| Condition | Status | Body |
|---|---|---|
| Bad Origin | 403 | `{ error: "Forbidden" }` |
| Rate limited | 429 | `{ error: "Too many requests" }` |
| Bound limiter throws | 429 | `{ error: "Too many requests" }` (fail-closed; log `[auth] AUTH_RATE_LIMITER.limit failed`) |
| Weak password | 400 | `{ error, details }` |
| Bad username / email | 400 | `{ error }` |
| Duplicate email/username | 409 | `{ error }` |
| Login: no user / no hash / wrong password | 401 | `{ error: "Invalid email or password" }` (one string) |
| Login/resend: banned | 403 | `{ error: "Account banned" }` |
| Resend: no session | 401 | `{ error: "Not authenticated" }` |
| Resend: already verified | 200 | `{ ok: true }` (no send) |
| Forgot-password any email | 200 | `{ ok: true }` (send only if unbanned account exists) |
| Bad/expired token | 400 | `{ error }` |
| Verify replay of consumed token, user already verified | 200 | `{ ok: true }` |

Forgot-password issues **only** `reset_password` tokens. It never sets
`email_verified_at`. Lost verification email →
`POST /api/auth/resend-verification` after sign-in.

### 5. Good/Base/Bad Cases

- Good: register → session cookie + verify email queued; login mixed-case
  email; resend while signed in; reset revokes all `sessions` rows (not
  `api_tokens`); verify-email retry after a lost 200 returns 200.
- Base: missing Resend secrets → register/forgot still 200, send skipped
  (logged); user recovers via resend.
- Bad: `github.com/${username}.png` fallback; GitHub OAuth routes; treating
  forgot-password as email verify; `bun run db:generate` in the live
  migrations dir to invent `0024` snapshot; bcrypt/argon2/Resend SDK
  dependencies.

### 6. Tests Required

No web unit-test runner. Gates: `bun run lint`, `typecheck`, `build`,
`test:migrations`. Assert:

- `0024` applied; `github_id` nullable; `users_email_lower_unique` UNIQUE on
  `lower((email)::text)`; `email_verification_tokens` exists.
- Build route table includes the six auth POSTs and **omits**
  `/api/auth/github*`.
- `rg VerifiedBadge|socialVerification|verifiedExpr web/src web/worker.ts`
  is empty.
- BDD: `features/email-auth.feature` (Chinese scenario text, English
  keywords). CLI device-flow scenario is `@todo` unless actually run.

### 7. Wrong vs Correct

#### Wrong

```ts
// send.ts claiming forgot-password verifies email
sendVerificationEmail(to, link); // then tell the user to use /forgot-password
```

```ts
return user.avatarUrl || `https://github.com/${user.username}.png`;
```

#### Correct

```ts
// Lost verify email: signed-in POST /api/auth/resend-verification
const link = await issueEmailToken(session.id, "verify_email");
sendVerificationEmail(user.email, link);
```

```ts
import { avatarUrlFor } from "@/lib/avatar";
src={avatarUrlFor(user)}
```

---

## Design Decision: PBKDF2 not bcrypt

Workers cannot load bcrypt/argon2 native modules. WebCrypto PBKDF2 is
zero-dependency and CPU-bounded. Prefix the stored hash so a later algorithm
can migrate.

## Design Decision: Fire-and-forget email

A failed Resend call must not roll back the user row. Recovery is
resend-verification, not register-rollback.

## Don't

- Do not add npm auth/email dependencies.
- Do not put `/login` (or other auth pages) in `worker.ts` `PAGE_CACHEABLE`.
- Do not delete `githubSocials.ts`, `isValidGitHubUsername`, `users.github_id`,
  or `ProfileSocialLinks` — they are profile/social-link, not login.
- Do not implement invitation-token cleanup until `team_invitations` exists
  (T3). Cron may only sweep `email_verification_tokens`.
