# Lessons (short entry)

Read this file at task start. Full details: `.trellis/lessons/index.md` then the matched topic.

<!-- lessons:640:start -->

- After a drop migration, search `web/scripts/check-migrations.ts` for the old table/index names. After merging SQL, the newest `meta/*_snapshot.json` idx must equal `_journal.json` tail. Do not run `bun run db:generate` in the live migrations directory to fill missing historical snapshots — that emits a new migration. Detail: `LESSON-20260911-640-migration-checker-dropped-tables`.

<!-- lessons:640:end -->
