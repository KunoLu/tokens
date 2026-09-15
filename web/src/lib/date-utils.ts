import { intlTag, type Locale } from "@/lib/i18n/locale";

function dateFromString(dateStr: string): Date {
  const [year, month, day] = dateStr.split("-").map(Number);
  return new Date(year, month - 1, day);
}

function getContributionLocalDate(contrib: { date: string; timestampMs?: number | null }): string {
  if (contrib.timestampMs != null) {
    const d = new Date(contrib.timestampMs);
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${y}-${m}-${day}`;
  }
  return contrib.date;
}

export function formatContributionDate(contrib: { date: string; timestampMs?: number | null }, locale: Locale = "en"): string {
  return new Intl.DateTimeFormat(intlTag(locale), {
    day: "numeric",
    month: "short",
    year: "numeric",
  }).format(dateFromString(getContributionLocalDate(contrib)));
}

export function formatContributionDateFull(contrib: { date: string; timestampMs?: number | null }, locale: Locale = "en"): string {
  return new Intl.DateTimeFormat(intlTag(locale), {
    day: "numeric",
    month: "long",
    year: "numeric",
  }).format(dateFromString(getContributionLocalDate(contrib)));
}

/** Year-less short date, e.g. "Sep 15" / "9月15日" — for tight stat subtexts. */
export function formatContributionDateShort(contrib: { date: string; timestampMs?: number | null }, locale: Locale = "en"): string {
  return new Intl.DateTimeFormat(intlTag(locale), {
    day: "numeric",
    month: "short",
  }).format(dateFromString(getContributionLocalDate(contrib)));
}
