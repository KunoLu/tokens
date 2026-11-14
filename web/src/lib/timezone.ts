/**
 * Viewer-timezone day assignment (issue #21).
 *
 * Daily contribution rows are bucketed by calendar date, so "which day is
 * today" is a question only a time zone can answer. The server cannot know
 * the viewer's zone on a cached page, so the rule is resolved here and
 * shared by both sides: an explicit account setting wins, the browser's
 * detected zone is the fallback, and "UTC" is what a guest with neither
 * effectively gets.
 */

/**
 * Canonical acceptance list: exactly what Intl.supportedValuesOf("timeZone")
 * reports, plus "UTC" (which a runtime may legitimately leave out of that
 * list). The Intl constructor alone is too lenient — it also takes offset
 * forms ("+08:00"), any casing, and alias names ("US/Eastern") — so the
 * canonical set is checked instead of constructor success.
 */
const CANONICAL_TIME_ZONES = new Set(Intl.supportedValuesOf("timeZone"));

export function isValidIanaTimeZone(value: string): boolean {
  return value === "UTC" || CANONICAL_TIME_ZONES.has(value);
}

/**
 * The calendar date (YYYY-MM-DD) that `now` falls on in `timeZone`.
 *
 * `toISOString().slice(0, 10)` would answer in UTC — the exact bug this
 * module exists to replace. formatToParts is used instead of a formatted
 * string so no locale's digit or ordering conventions leak into the result.
 */
export function dateStringInTimeZone(now: Date, timeZone: string): string {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(now);
  let year = "";
  let month = "";
  let day = "";
  for (const part of parts) {
    if (part.type === "year") year = part.value;
    else if (part.type === "month") month = part.value;
    else if (part.type === "day") day = part.value;
  }
  return `${year}-${month}-${day}`;
}

/**
 * Day arithmetic on a YYYY-MM-DD calendar date. Done in UTC so subtracting
 * a day is plain arithmetic — no DST hour can land on the wrong side of
 * midnight.
 */
export function addDaysToDateString(dateString: string, days: number): string {
  const [year, month, day] = dateString.split("-").map(Number);
  return new Date(Date.UTC(year, month - 1, day + days))
    .toISOString()
    .slice(0, 10);
}

/** The browser's detected zone, or null when detection yields nothing valid. */
export function detectBrowserTimeZone(): string | null {
  const detected = Intl.DateTimeFormat().resolvedOptions().timeZone;
  return typeof detected === "string" && isValidIanaTimeZone(detected)
    ? detected
    : null;
}

/**
 * The viewer's effective zone: their saved account setting when present and
 * valid, otherwise their browser's detection. Never the profile owner's —
 * day assignment follows whoever is looking (issue #21, rule 3).
 */
export function resolveViewerTimeZone(
  saved: string | null | undefined,
  detected: string | null,
): string | null {
  if (saved && isValidIanaTimeZone(saved)) return saved;
  return detected;
}
