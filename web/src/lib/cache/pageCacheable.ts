/**
 * Signed-out HTML routes the Worker may put in `caches.default`.
 *
 * `/teamboard` is intentionally absent: public→private must 404 immediately
 * and `revalidateTag("leaderboard")` does not purge the Cache API.
 */
export const PAGE_CACHEABLE = /^\/(leaderboard)?$/;

/** Same predicate `worker.ts` `sharedCacheKey` uses for signed-out HTML. */
export function workerSharesSignedOutHtml(
  pathname: string,
  hasSession: boolean
): boolean {
  return !hasSession && PAGE_CACHEABLE.test(pathname);
}
