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
 * Acceptance and normalization in one rule; PATCH and the `?tz` parameter
 * share it. Three checks, all required:
 *  1. not an offset spelling — no leading "+"/"-", and no "UTC±…"/"GMT±…"
 *     form ("UTC+8", "GMT+8");
 *  2. either literally "UTC" or a name containing "/" — so slash-less
 *     abbreviations like "EST" (and "GMT") are out even when the constructor
 *     would take them;
 *  3. `new Intl.DateTimeFormat("en-US", { timeZone })` must construct.
 * What comes back is the runtime's own resolvedOptions().timeZone spelling,
 * and that is what gets stored — "asia/shanghai" as "Asia/Shanghai", and on
 * Node "Asia/Kolkata" as "Asia/Calcutta" (Bun keeps "Asia/Kolkata").
 *
 * The constructor, not Intl.supportedValuesOf("timeZone"), is the source of
 * truth because that list is runtime-specific: Node 22 lists only the old
 * names (Asia/Calcutta, Europe/Kiev, Asia/Saigon) and omits "UTC", Bun the
 * reverse, so either list rejects users the other runtime accepts.
 */
export function normalizeIanaTimeZone(value: string): string | null {
  // No IANA name starts with a sign; offset forms like "+08:00" do
  // construct, so they are rejected explicitly, as are "UTC+8"/"GMT+8" style
  // spellings (which mostly fail to construct anyway).
  if (value.startsWith("+") || value.startsWith("-")) return null;
  if (/^(?:utc|gmt)[+-]/i.test(value)) return null;
  if (value !== "UTC" && !value.includes("/")) return null;
  try {
    return new Intl.DateTimeFormat("en-US", { timeZone: value })
      .resolvedOptions().timeZone;
  } catch {
    // Empty, over-long, and simply unknown names all land here.
    return null;
  }
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
  return typeof detected === "string" && normalizeIanaTimeZone(detected) !== null
    ? detected
    : null;
}

/**
 * The viewer's effective zone: their saved account setting when present and
 * valid, otherwise their browser's detection. Never the profile owner's —
 * day assignment follows whoever is looking (issue #21, rule 3). The saved
 * value comes back as stored (the server already normalized it on write);
 * validation only decides whether it wins.
 */
export function resolveViewerTimeZone(
  saved: string | null | undefined,
  detected: string | null,
): string | null {
  if (saved && normalizeIanaTimeZone(saved) !== null) return saved;
  return detected;
}
