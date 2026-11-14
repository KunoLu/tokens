/// <reference types="bun-types" />
import { describe, expect, test } from "bun:test";
import {
  addDaysToDateString,
  dateStringInTimeZone,
  normalizeIanaTimeZone,
  resolveViewerTimeZone,
} from "./timezone";

// What this runtime itself resolves a zone to — expectations must come from
// here, never from a hardcoded spelling, because Node maps renamed zones to
// their old IANA names (Asia/Kolkata → Asia/Calcutta) while Bun keeps the
// submitted spelling.
const runtimeSpelling = (zone: string) =>
  new Intl.DateTimeFormat("en-US", { timeZone: zone }).resolvedOptions()
    .timeZone;

describe("normalizeIanaTimeZone", () => {
  test("accepts real IANA names and returns the runtime's spelling", () => {
    for (const zone of [
      "Asia/Shanghai",
      "America/New_York",
      "UTC",
      "Pacific/Kiritimati",
    ]) {
      expect(normalizeIanaTimeZone(zone)).toBe(runtimeSpelling(zone));
    }
  });

  // Acceptance is constructor-based, not list-based: runtimes disagree on
  // Intl.supportedValuesOf("timeZone") (Node 22 lists only the old names and
  // omits "UTC"; Bun the reverse), so both spellings of a renamed zone must
  // pass on either runtime. What gets stored is the runtime's own spelling —
  // Node canonicalizes to the old name, Bun keeps the submitted one.
  test("accepts old and new IANA spellings on any runtime", () => {
    for (const zone of [
      "Asia/Kolkata",
      "Asia/Calcutta",
      "Europe/Kyiv",
      "Europe/Kiev",
    ]) {
      expect(normalizeIanaTimeZone(zone)).toBe(runtimeSpelling(zone));
    }
  });

  test("a lower-case spelling is accepted and stored canonicalized", () => {
    expect(normalizeIanaTimeZone("asia/shanghai")).toBe(
      runtimeSpelling("asia/shanghai"),
    );
    expect(normalizeIanaTimeZone("asia/shanghai")).toBe(
      normalizeIanaTimeZone("Asia/Shanghai"),
    );
  });

  // The Intl constructor is more lenient than the acceptance rule: it takes
  // offset forms and slash-less abbreviations, and those stay rejected.
  test("rejects offset forms and slash-less abbreviations", () => {
    for (const bad of ["EST", "+08:00", "-05:00", "UTC+8", "GMT+8", "GMT"]) {
      expect(normalizeIanaTimeZone(bad)).toBeNull();
    }
  });

  test("rejects non-IANA strings", () => {
    expect(normalizeIanaTimeZone("Not/AZone")).toBeNull();
    expect(normalizeIanaTimeZone("Asia")).toBeNull();
    expect(normalizeIanaTimeZone("utc")).toBeNull();
  });

  test("rejects empty and over-long strings", () => {
    expect(normalizeIanaTimeZone("")).toBeNull();
    expect(normalizeIanaTimeZone("A".repeat(200))).toBeNull();
  });
});

describe("dateStringInTimeZone", () => {
  // 2026-11-13 16:30 UTC = 2026-11-14 00:30 in UTC+8 (the issue's repro window).
  const shanghaiMorning = new Date("2026-11-13T16:30:00.000Z");
  // 2026-11-14 04:30 UTC = 2026-11-13 23:30 in UTC-5.
  const newYorkLateEvening = new Date("2026-11-14T04:30:00.000Z");

  test("UTC+8 assigns the next calendar day while UTC is still on the previous one", () => {
    expect(dateStringInTimeZone(shanghaiMorning, "Asia/Shanghai")).toBe("2026-11-14");
    expect(dateStringInTimeZone(shanghaiMorning, "UTC")).toBe("2026-11-13");
  });

  test("UTC-5 stays on the previous calendar day after UTC midnight", () => {
    expect(dateStringInTimeZone(newYorkLateEvening, "America/New_York")).toBe("2026-11-13");
    expect(dateStringInTimeZone(newYorkLateEvening, "UTC")).toBe("2026-11-14");
  });

  test("the two hemispheres disagree across the same instant", () => {
    expect(dateStringInTimeZone(newYorkLateEvening, "Asia/Tokyo")).toBe("2026-11-14");
    expect(dateStringInTimeZone(newYorkLateEvening, "America/New_York")).toBe("2026-11-13");
  });

  test("UTC+14 crosses the date line eleven hours before UTC", () => {
    expect(dateStringInTimeZone(new Date("2026-11-13T11:00:00.000Z"), "Pacific/Kiritimati")).toBe("2026-11-14");
  });
});

describe("addDaysToDateString", () => {
  test("walks backwards across month and year boundaries", () => {
    expect(addDaysToDateString("2026-11-14", -6)).toBe("2026-11-08");
    expect(addDaysToDateString("2026-03-01", -1)).toBe("2026-02-28");
    expect(addDaysToDateString("2026-01-01", -1)).toBe("2025-12-31");
  });

  test("walks forwards across a month boundary", () => {
    expect(addDaysToDateString("2026-02-28", 2)).toBe("2026-03-02");
  });
});

describe("resolveViewerTimeZone", () => {
  test("a saved setting overrides the browser detection", () => {
    expect(resolveViewerTimeZone("Asia/Tokyo", "America/New_York")).toBe("Asia/Tokyo");
  });

  test("falls back to the browser detection when nothing is saved", () => {
    expect(resolveViewerTimeZone(null, "America/New_York")).toBe("America/New_York");
    expect(resolveViewerTimeZone(undefined, "UTC")).toBe("UTC");
  });

  test("an invalid saved value never wins over detection", () => {
    expect(resolveViewerTimeZone("Not/AZone", "UTC")).toBe("UTC");
  });

  test("a guest with no detection resolves to nothing", () => {
    expect(resolveViewerTimeZone(null, null)).toBeNull();
  });
});
