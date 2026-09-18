import type { Locale } from "@/lib/i18n/locale";
import { t } from "@/lib/i18n/t";

/**
 * Format an ISO timestamp as a short relative time, e.g. "just now",
 * "5m ago", "3h ago", "12d ago", "2mo ago", "1y ago". Returns "never"
 * for null/invalid input so callers can render it directly.
 *
 * `now` is injectable for tests; future timestamps clamp to "just now".
 * Lives outside `format.ts` so embed/SVG/OG renderers can import compact
 * number helpers without pulling the dictionary.
 */
export function formatRelativeTime(
  iso: string | null | undefined,
  now: Date = new Date(),
  locale: Locale = "en",
): string {
  if (!iso) return t(locale, "relative.never");
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return t(locale, "relative.never");

  const diffMs = now.getTime() - date.getTime();
  if (diffMs < 60_000) return t(locale, "relative.justNow");

  const minutes = Math.floor(diffMs / 60_000);
  if (minutes < 60) return t(locale, "relative.minutesAgo", { n: minutes });

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return t(locale, "relative.hoursAgo", { n: hours });

  const days = Math.floor(hours / 24);
  if (days < 30) return t(locale, "relative.daysAgo", { n: days });
  if (days < 365) {
    const months = Math.floor(days / 30);
    return t(locale, "relative.monthsAgo", { n: months });
  }

  const years = Math.floor(days / 365);
  return t(locale, "relative.yearsAgo", { n: years });
}
