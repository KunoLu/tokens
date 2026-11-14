import { revalidatePath, revalidateTag } from "next/cache";
import { DEFAULT_LOCALE, LOCALES } from "@/lib/i18n/config";

/**
 * The one tag every leaderboard cache entry carries, whatever its own key:
 * the shared period rankings (`period-ranking:*`), the all-time and
 * directive-search listings (`leaderboard:*`), the all-time per-user ranks
 * (`user-rank:*`) and the teamboard aggregates each add their own narrower
 * tags, but all of them — every period, date-range, sort, page, search and
 * viewer variant — hang on this one, so a single revalidateTag drops the lot.
 * The cached data is locale-independent (translations happen at render), so
 * this one tag covers the data behind all three languages' boards.
 */
export const LEADERBOARD_CACHE_TAG = "leaderboard";

/**
 * The public addresses of the board, one per language. The default language
 * answers unprefixed; the rest live under their prefix, and the middleware
 * rewrites those to the single internal route — so the route path alone never
 * matches the copies a shared edge cache stores under the prefixed URLs.
 * Purging the board's pages has to name each language's address, and a path
 * purge covers every query-string variant (period, sort, page, search, date
 * range) the edge stored under that address.
 */
export const LEADERBOARD_PAGE_PATHS: readonly string[] = LOCALES.map(
  (locale) => (locale === DEFAULT_LOCALE ? "/leaderboard" : `/${locale}/leaderboard`),
);

/**
 * Drop every cached copy of the board at once: the tagged data entries (all
 * period, date-range, sort, page and search variants, all languages' data in
 * one tag) plus the per-language page copies. Used when a user's visibility
 * changes — a profile going private must leave the board now, not after the
 * edge's stale window runs out. Callers keep their own best-effort try/catch,
 * matching the surrounding invalidation idiom.
 */
export function revalidateLeaderboard(): void {
  revalidateTag(LEADERBOARD_CACHE_TAG, { expire: 0 });
  for (const path of LEADERBOARD_PAGE_PATHS) {
    revalidatePath(path);
  }
}
