# T8 implement

Active task: `.trellis/tasks/09-11-t8-profile-membership`

## Steps

1. Page-only membership query (not `getPublicProfileResponse`): team `{id,name,slug}` + group `{id,name}|null` for the profile user. Soft-fail to null when `DATABASE_URL` is missing.
2. Pass membership into `ProfilePageClient` / `ProfileView`. Hide the whole block when membership is null.
3. Client: `GET /api/auth/session`; `isOwner` = username match. Only then render Leave Team / Leave Group.
4. Leave: existing DELETE `/api/teams/:teamId/members/me` and `/api/teams/:teamId/groups/:groupId/members/me`. Toast errors. 409 admin copy = transfer or disband.
5. `leaveTeam` / `leaveGroup`: after the txn commits, SELECT `users.username` by the same `userId` (never a client username), then `bumpLeaderboard()` and `revalidateTag('user:' + normalizeUsernameCacheKey(username), 'max')`.
6. `t()` keys for new copy. Playwright covering `web/features/profile-team-membership.feature`.

## Do not

- New leave routes
- Team/Group on GET `/api/users/[username]`
- Bake `isOwner` into cached RSC HTML
- Commit, archive, start T11, merge main

## Acceptance

Visitor sees Team/Group names, never leave. Owner sees leave. No membership → no block. Admin leave Team 409 and still a member. Leave Team clears Group (INV-8). lint + typecheck + test:teams + native e2e.
