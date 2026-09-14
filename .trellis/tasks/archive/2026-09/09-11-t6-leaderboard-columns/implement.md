# T6 implement

## Order

1. `lib/leaderboard/types.ts` — `team` / `group` refs
2. `lib/leaderboard/getLeaderboard.ts` — LEFT JOIN on both query paths + user rank
3. `components/leaderboard/MembershipCells.tsx` — desktop cells + mobile badges
4. `Leaderboard.tsx` / `LeaderboardSkeleton.tsx`

## Validation

- `bun run lint`
- `bun run typecheck`
- `bun run test:teams` not required (no domain writes)
- `bun run build` if time

## Rollback

Revert the five files above. Schema `0025` stays.
