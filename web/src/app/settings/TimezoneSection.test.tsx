/// <reference types="bun-types" />
import { afterAll, afterEach, describe, expect, mock, test } from "bun:test";
import { GlobalRegistrator } from "@happy-dom/global-registrator";

// DOM globals must exist before react-dom or the component module load, so
// everything below that touches them is a deliberate dynamic import (module
// loading boundary — static imports would hoist above the registration).
// All happy-dom test files share one bun process; register exactly once.
if (!GlobalRegistrator.isRegistered) {
  GlobalRegistrator.register();
}
(globalThis as Record<string, unknown>).IS_REACT_ACT_ENVIRONMENT = true;

mock.module("@/components/layout/LocaleProvider", () => ({
  useLocale: () => "en",
  useT: () => (key: string) => key,
}));

const { default: TimezoneSection } = await import("./TimezoneSection");
const { createRoot } = await import("react-dom/client");
const { act } = await import("react");

const REAL_SUPPORTED_VALUES_OF = Intl.supportedValuesOf;
const REAL_RESOLVED_OPTIONS = Intl.DateTimeFormat.prototype.resolvedOptions;
const REAL_FETCH = globalThis.fetch;

// The saved setting arrives over fetch, same as in production. Each test
// sets `savedTimezone` for itself so shuffled order cannot cross-contaminate.
let savedTimezone: string | null = null;
globalThis.fetch = (async (input: RequestInfo | URL) => {
  const url = String(input);
  if (url === "/api/settings/timezone") {
    return new Response(JSON.stringify({ timezone: savedTimezone }), { status: 200 });
  }
  return new Response("not found", { status: 404 });
}) as typeof fetch;

/**
 * Emulate a browser whose list carries only the new IANA spellings and whose
 * Intl canonicalizes renamed zones onto them — the combination that must not
 * strand a saved old spelling on the auto-detect entry.
 */
function pretendBrowserCanonicalizes() {
  Intl.supportedValuesOf = (() => [
    "Asia/Kolkata",
    "Asia/Shanghai",
    "America/New_York",
    "UTC",
  ]) as typeof Intl.supportedValuesOf;
  const canonical: Record<string, string> = {
    "Asia/Calcutta": "Asia/Kolkata",
    "Asia/Kolkata": "Asia/Kolkata",
  };
  Intl.DateTimeFormat.prototype.resolvedOptions = function (
    this: Intl.DateTimeFormat,
    ...args: Parameters<Intl.DateTimeFormat["resolvedOptions"]>
  ) {
    const resolved = REAL_RESOLVED_OPTIONS.apply(this, args);
    const zone = resolved.timeZone;
    return { ...resolved, timeZone: (zone && canonical[zone]) ?? zone };
  };
}

const roots: Array<{ root: { unmount: () => void }; container: HTMLElement }> = [];

async function flushEffects() {
  for (let round = 0; round < 5; round += 1) {
    await act(async () => {
      const { promise, resolve } = Promise.withResolvers<void>();
      setTimeout(resolve, 0);
      await promise;
    });
  }
}

async function renderSection() {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(<TimezoneSection />);
  });
  await flushEffects();
  roots.push({ root, container });
  return container;
}

function optionValues(select: HTMLSelectElement) {
  return Array.from(select.querySelectorAll("option")).map(
    (option) => option.value,
  );
}

afterEach(() => {
  savedTimezone = null;
  Intl.supportedValuesOf = REAL_SUPPORTED_VALUES_OF;
  Intl.DateTimeFormat.prototype.resolvedOptions = REAL_RESOLVED_OPTIONS;
  while (roots.length > 0) {
    const { root, container } = roots.pop()!;
    act(() => {
      root.unmount();
    });
    container.remove();
  }
});

afterAll(() => {
  globalThis.fetch = REAL_FETCH;
  (globalThis as Record<string, unknown>).IS_REACT_ACT_ENVIRONMENT = false;
  // Whichever happy-dom file runs first registers for both (module-level
  // guard), so each file must clean up after itself: under --randomize any
  // non-DOM file can run after this one, and leaked window/Request globals
  // break it. Guarded because the other file may have unregistered already.
  if (GlobalRegistrator.isRegistered) {
    GlobalRegistrator.unregister();
  }
});

describe("TimezoneSection saved value display", () => {
  test("a saved old IANA spelling matches the new-spelling option, not auto-detect", async () => {
    // The server on Node stores "Asia/Calcutta"; this browser's list only
    // knows "Asia/Kolkata". Matching happens after both sides go through the
    // same runtime's resolvedOptions spelling, so the select shows the
    // new-spelling option — not the auto-detect entry, not an appended alias.
    pretendBrowserCanonicalizes();
    savedTimezone = "Asia/Calcutta";
    const container = await renderSection();

    const select = container.querySelector("select");
    expect(select).not.toBeNull();
    expect(select!.value).toBe("Asia/Kolkata");
    const values = optionValues(select!);
    expect(values).toContain("Asia/Kolkata");
    expect(values).not.toContain("Asia/Calcutta");
    // The auto-detect entry stays available.
    expect(values).toContain("");
  });

  test("a saved zone no option can match is appended so it stays visible", async () => {
    pretendBrowserCanonicalizes();
    savedTimezone = "Pacific/Kiritimati";
    const container = await renderSection();

    const select = container.querySelector("select");
    expect(select).not.toBeNull();
    expect(select!.value).toBe("Pacific/Kiritimati");
    expect(optionValues(select!)).toContain("Pacific/Kiritimati");
  });
});
