# T10 implement

## Order

1. `lib/i18n/{locale,t,I18nProvider,index}.ts`
2. `format.ts` optional locale param (default en)
3. `app/layout.tsx` cookies → html lang + Providers
4. Navigation globe toggle left of theme
5. `worker.ts` and `loadPublicProfileForPage` cache keys

## Validation

- `bun run lint`
- `bun run typecheck`
- Reviewer pass (required for this and later slices)

## Rollback

Delete `lib/i18n/`. Restore layout, Navigation, Providers, worker, format.ts, publicProfileData, u/[username]/page.tsx.
