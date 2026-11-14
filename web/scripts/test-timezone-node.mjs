/**
 * Node-side run of the time zone acceptance rule (issue #21, PR D). The web
 * suite runs under Bun, but the API serves on Node in production — and the
 * two runtimes resolve renamed IANA zones to different spellings (Node maps
 * Asia/Kolkata to Asia/Calcutta, Bun keeps Asia/Kolkata), so the rule must
 * be exercised against the real module under Node too. Deliberately named
 * outside the *.test.* / *_test.* patterns so `bun test` does not collect
 * it; run via `bun run test:node` (node --test).
 */
import assert from "node:assert/strict";
import { test } from "node:test";
import { normalizeIanaTimeZone } from "../src/lib/timezone.ts";

// Expectations come from this runtime's own resolvedOptions, never a
// hardcoded spelling: Node and Bun canonicalize renamed zones differently.
const runtimeSpelling = (zone) =>
  new Intl.DateTimeFormat("en-US", { timeZone: zone }).resolvedOptions()
    .timeZone;

test("accepts IANA names, both spellings of renamed zones, lower case", () => {
  for (const zone of [
    "Asia/Kolkata",
    "Asia/Calcutta",
    "Europe/Kyiv",
    "Europe/Kiev",
    "UTC",
    "asia/shanghai",
    "Asia/Shanghai",
    "America/New_York",
    "Pacific/Kiritimati",
  ]) {
    assert.equal(normalizeIanaTimeZone(zone), runtimeSpelling(zone));
  }
});

test("rejects offset forms, slash-less abbreviations, and junk", () => {
  for (const bad of [
    "EST",
    "+08:00",
    "-05:00",
    "UTC+8",
    "GMT+8",
    "GMT",
    "utc",
    "Not/AZone",
    "Asia",
    "",
    "A".repeat(200),
  ]) {
    assert.equal(normalizeIanaTimeZone(bad), null);
  }
});
