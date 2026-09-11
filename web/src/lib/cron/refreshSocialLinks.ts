import { db, users } from "@/lib/db";
import { syncGitHubSocialLinks } from "@/lib/githubSocials";

const SYNC_CONCURRENCY = 4;

interface RefreshSocialLinksResult {
  users: number;
}

/**
 * Re-sync every user's GitHub social-links snapshot, the source of the
 * profile page's social-links row. Runs in batches so a few hundred users
 * don't open a few hundred simultaneous connections to GitHub.
 *
 * Callers are responsible for keeping the runtime alive for the duration:
 * a Workers isolate stops executing once its response is returned unless the
 * promise is handed to `waitUntil`, so fire-and-forget would be truncated.
 */
export async function refreshAllSocialLinks(): Promise<RefreshSocialLinksResult> {
  const rows = await db.select({ username: users.username }).from(users);

  for (let i = 0; i < rows.length; i += SYNC_CONCURRENCY) {
    const batch = rows.slice(i, i + SYNC_CONCURRENCY);
    await Promise.all(batch.map((row) => syncGitHubSocialLinks(row.username)));
  }

  return { users: rows.length };
}

/**
 * Count of users to report before the sync starts, so the HTTP endpoint can
 * answer 202 with a meaningful body without waiting for the whole run.
 */
export async function countUsers(): Promise<number> {
  const rows = await db.select({ username: users.username }).from(users);
  return rows.length;
}
