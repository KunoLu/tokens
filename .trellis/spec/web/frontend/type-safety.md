# Type Safety

> TypeScript everywhere; the submission contract is validated at runtime with
> Zod. Types are shared through `lib/`, not duplicated per page.

---

## Where types live

- **Shared domain types**: `web/src/lib/types.ts` —
  `SUPPORTED_CLIENT_TYPES` (a `as const` array, lines 1-46), `ClientType`
  (including the `` `cc-mirror/${string}` `` template-literal variant),
  `TokenBreakdown` (`input/output/cacheRead/cacheWrite/reasoning`),
  contribution shapes.
- **Feature types**: colocated with the feature —
  `lib/leaderboard/types.ts` (`LeaderboardData`, `Period`, `SortBy`),
  `components/profile/types.ts` (`ProfileUser`, `ModelUsage`).
- **Page-local interfaces**: fine when a shape is only a page's contract with
  its client shell (`ProfileData` in `ProfilePageClient.tsx`,
  `SettingsDevice` in `SettingsClient.tsx` — with a comment when it mirrors an
  API response shape).

## Conventions

1. **Const arrays → union types.** Enumerations are `as const` arrays with a
   derived union (`typeof SUPPORTED_CLIENT_TYPES[number]`), parsed at runtime
   with `includes` guards (e.g. `PROFILE_PERIODS`, `VALID_PERIODS`). This
   keeps the runtime list and the type in one place.
2. **Zod at the trust boundary.** Every submission payload is validated by
   `validateSubmission` in `web/src/lib/validation/submission.ts` before it
   touches the database; the schemas are the single source of truth the CLI's
   camelCase JSON must match
   (`.trellis/spec/tokens-cli/backend/quality-guidelines.md`). Legacy client
   ids are normalized in `z.preprocess` (`LEGACY_CLIENT_ALIASES`,
   `submission.ts:172-189`), not rejected.
3. **Drizzle inference for DB rows.** Row types come from the schema
   (`$inferSelect` on tables in `web/src/lib/db/schema.ts`); queries use typed
   `sql<>` aggregates rather than `any`.
4. **Route handlers return `NextResponse.json`** with typed error bodies
   (`{ error: string }`), so client `fetch` code can narrow on `status`.
5. **`cn()`-style class strings are just strings** — no prop-type gymnastics;
   variant props are typed with `VariantProps<typeof …Variants>` from CVA (see
   `components/ui/button.tsx`).

## Anti-patterns

- Re-declaring a client/model union locally instead of importing from
  `lib/types.ts` — the server rejects client ids outside
  `SUPPORTED_CLIENT_TYPES`, so drift breaks submissions.
- `as any` around Drizzle results; use `sql<…>` type parameters.
- New ambient global types when a `lib/` export works — `src/types/` holds
  only true ambient declarations (`static-images.d.ts`).
