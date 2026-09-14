# T4 implement

## Order

1. `web/src/lib/teams/visibility.ts` — `canViewTeam` (members or public+active only)
2. `web/src/lib/teams/service.ts` — create/list/get/patch/disband/delete team; members; groups; invitations; search users; leave; accept/decline
3. `web/src/lib/email/send.ts` — team invite template (English, same as other mail). Link is `/login` if the invitee already has an account, `/register` otherwise. Do **not** put `invite=` or the raw token in the URL: login only reads `returnTo`, and accept is `POST /api/invitations/:id/accept`. T5 owns the post-login accept UI via `GET /api/me/invitations`.
4. Route handlers matching PRD §9.2 (thin: session + call service + JSON)
5. Hook `linkPendingInvitationsForEmail` after successful verify-email (independent txn; not register)
6. Daily cron: three independent `waitUntil` tasks — `refreshAllSocialLinks`, `deleteExpiredEmailTokens`, `expireInvitations`
7. `web/scripts/check-teams-invariants.ts` + `package.json` script `test:teams`
8. `bun run lint` `bun run typecheck` `DATABASE_URL=... bun run test:teams`

## Validation

- `bun run lint`
- `bun run typecheck`
- `DATABASE_URL=postgresql://tokens:tokens@127.0.0.1:5433/tokens bun run test:teams`
- `bun run build` — pass (2026-09-14). `cf:preview` not run.

## Rollback

Revert in this order so typecheck still passes:
1. Restore `web/src/app/api/auth/verify-email/route.ts` (remove `linkPendingInvitationsForEmail`).
2. Restore `web/worker.ts` (remove `expireInvitations` and restore sequential token cleanup).
3. Restore `web/src/lib/email/send.ts` (remove `sendTeamInviteEmail`).
4. Delete new routes under `app/api/teams/`, `app/api/me/invitations/`, `app/api/invitations/`, `app/api/users/search/`.
5. Delete `lib/teams/{errors,http,service,visibility}.ts`. Keep `lib/teams/types.ts` (T3).
6. Delete `web/scripts/check-teams-invariants.ts` and the `test:teams` script in `web/package.json`.
No schema rollback (`0025` stays).

## BDD

Existing `web/features/team-management.feature` + `group-management.feature` are SOT. Language: Chinese scenario text + English keywords (existing). Trace `check-teams-invariants.ts` case names to scenarios. No Gherkin runner.
