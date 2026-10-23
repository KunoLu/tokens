import { intlTag, type Locale } from "@/lib/i18n/locale";
import { t } from "@/lib/i18n/t";


export function escapeXml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

function safeNumber(value: number): number {
  return Number.isFinite(value) ? value : 0;
}

function numberLocale(locale: Locale): string {
  return intlTag(locale);
}

export function formatCompact(
  value: number,
  kind: "number" | "currency",
  locale: Locale = "en"
): string {
  const clamped = Math.max(0, safeNumber(value));
  const tag = numberLocale(locale);

  if (kind === "currency") {
    const formatted = new Intl.NumberFormat(tag, {
      notation: "compact",
      maximumFractionDigits: clamped >= 100 ? 1 : 2,
    }).format(clamped);
    return `$${formatted}`;
  }

  return new Intl.NumberFormat(tag, {
    notation: "compact",
    maximumFractionDigits: clamped >= 100 ? 1 : 2,
  }).format(Math.round(clamped));
}

export function formatNumber(
  value: number,
  compact = false,
  locale: Locale = "en"
): string {
  if (compact) return formatCompact(value, "number", locale);
  return new Intl.NumberFormat(numberLocale(locale)).format(
    Math.max(0, Math.round(safeNumber(value)))
  );
}

export function formatCurrency(
  value: number,
  compact = false,
  locale: Locale = "en"
): string {
  if (compact) return formatCompact(value, "currency", locale);
  return new Intl.NumberFormat(numberLocale(locale), {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Math.max(0, safeNumber(value)));
}

/**
 * Format a 0–100 share with one decimal. Non-zero shares below 0.05% stay visible.
 */
export function formatPercent(percent: number, locale: Locale = "en"): string {
  const small = percent > 0 && percent < 0.05;
  const value = new Intl.NumberFormat(numberLocale(locale), {
    style: "percent",
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  }).format((small ? 0.1 : percent) / 100);
  return small ? t(locale, "profile.percent.lessThan", { value }) : value;
}

/**
 * Format 0–100 shares as whole percents that add up to their own rounded total
 * (largest remainder), so the parts of a split never disagree with the whole.
 * A non-zero share that rounds to 0 shows as "<1%".
 */
export function formatWholePercents(percents: readonly number[], locale: Locale = "en"): string[] {
  const whole = percents.map((percent) => Math.floor(percent));
  let left =
    Math.round(percents.reduce((sum, percent) => sum + percent, 0)) -
    whole.reduce((sum, value) => sum + value, 0);
  const byRemainder = percents
    .map((percent, index) => ({ index, remainder: percent - whole[index] }))
    .sort((a, b) => b.remainder - a.remainder);
  for (const { index } of byRemainder) {
    if (left <= 0) break;
    whole[index] += 1;
    left -= 1;
  }
  const formatter = new Intl.NumberFormat(numberLocale(locale), {
    style: "percent",
    maximumFractionDigits: 0,
  });
  return whole.map((value, index) =>
    value === 0 && percents[index] > 0
      ? t(locale, "profile.percent.lessThan", { value: formatter.format(0.01) })
      : formatter.format(value / 100)
  );
}


/**
 * Format milliseconds into a human-readable duration string.
 * e.g. 3661000 → "1h 1m", 90000 → "1m 30s", 500 → "<1s"
 */
export function formatDuration(ms: number | null | undefined): string {
  if (ms == null || !Number.isFinite(ms) || ms <= 0) return "—";

  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
  }
  if (minutes > 0) {
    return seconds > 0 && minutes < 10 ? `${minutes}m ${seconds}s` : `${minutes}m`;
  }
  if (seconds > 0) {
    return `${seconds}s`;
  }
  return "<1s";
}
