/// <reference types="bun-types" />
import { describe, expect, test } from "bun:test";
import {
  addDaysToDateString,
  dateStringInTimeZone,
  isValidIanaTimeZone,
  resolveViewerTimeZone,
} from "./timezone";

describe("isValidIanaTimeZone", () => {
  test("accepts real IANA names", () => {
    expect(isValidIanaTimeZone("Asia/Shanghai")).toBe(true);
    expect(isValidIanaTimeZone("America/New_York")).toBe(true);
    expect(isValidIanaTimeZone("UTC")).toBe(true);
    expect(isValidIanaTimeZone("Pacific/Kiritimati")).toBe(true);
  });

  test("rejects non-IANA strings", () => {
    expect(isValidIanaTimeZone("Not/AZone")).toBe(false);
    expect(isValidIanaTimeZone("")).toBe(false);
    expect(isValidIanaTimeZone("UTC+8")).toBe(false);
    expect(isValidIanaTimeZone("Asia")).toBe(false);
  });

  // Only canonical names from Intl.supportedValuesOf("timeZone") plus "UTC"
  // are accepted. The Intl constructor is more lenient than that: it takes
  // offset forms, any casing, and alias (linked) names.
  test("rejects offset forms the Intl constructor would accept", () => {
    expect(isValidIanaTimeZone("GMT+8")).toBe(false);
    expect(isValidIanaTimeZone("+08:00")).toBe(false);
    expect(isValidIanaTimeZone("UTC+8")).toBe(false);
  });

  test("rejects non-canonical casing", () => {
    expect(isValidIanaTimeZone("asia/shanghai")).toBe(false);
    expect(isValidIanaTimeZone("AMERICA/NEW_YORK")).toBe(false);
  });

  test("rejects alias names in favor of their canonical targets", () => {
    expect(isValidIanaTimeZone("Asia/Chungking")).toBe(false);
    expect(isValidIanaTimeZone("Asia/Chongqing")).toBe(false);
    expect(isValidIanaTimeZone("US/Eastern")).toBe(false);
    expect(isValidIanaTimeZone("GMT")).toBe(false);
  });

  test("rejects empty and over-long strings", () => {
    expect(isValidIanaTimeZone("")).toBe(false);
    expect(isValidIanaTimeZone("A".repeat(200))).toBe(false);
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
