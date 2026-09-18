import { db, users } from "@/lib/db";
import { syncGitHubSocialLinksStrict } from "@/lib/githubSocials";

const SYNC_CONCURRENCY = 4;

interface RefreshSocialLinksResult {
  users: number;
  failed: number;
}

/**
 * Re-sync every user's GitHub social-links snapshot, the source of the
 * profile page's social-links row. Runs in batches so a few hundred users
 * don't open a few hundred simultaneous connections to GitHub.
 *
 * Uses the strict per-user sync and counts failures: the cron caller turns
 * `failed > 0` into a 500 so the scheduler retries instead of treating
 * silently stale snapshots as success.
 */
export async function refreshAllSocialLinks(): Promise<RefreshSocialLinksResult> {
  const rows = await db.select({ username: users.username }).from(users);

  let failed = 0;
  for (let i = 0; i < rows.length; i += SYNC_CONCURRENCY) {
    const batch = rows.slice(i, i + SYNC_CONCURRENCY);
    const results = await Promise.all(
      batch.map((row) =>
        syncGitHubSocialLinksStrict(row.username).then(
          () => true,
          () => false,
        ),
      ),
    );
    failed += results.filter((ok) => !ok).length;
  }

  return { users: rows.length, failed };
}
