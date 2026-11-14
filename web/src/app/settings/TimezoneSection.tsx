"use client";

import { useCallback, useEffect, useId, useMemo, useState } from "react";
import { useT } from "@/components/layout/LocaleProvider";
import { detectBrowserTimeZone } from "@/lib/timezone";

const Section = "mb-6 rounded-xl border bg-card p-6";
const SectionTitle = "mb-4 text-lg font-semibold text-foreground";

/**
 * One spelling to compare by: both the saved value and every list entry go
 * through this runtime's resolvedOptions canonicalization, so an old IANA
 * spelling stored by the server (Node resolves Asia/Kolkata to
 * Asia/Calcutta) still matches a browser list that only carries the new one.
 */
function canonicalTimeZoneName(zone: string): string {
  try {
    return new Intl.DateTimeFormat("en-US", { timeZone: zone })
      .resolvedOptions().timeZone;
  } catch {
    return zone;
  }
}

/**
 * Account time zone override (issue #21): empty means auto-detect from the
 * browser. The saved value wins everywhere the viewer looks at a profile —
 * their own or anyone else's.
 */
export default function TimezoneSection() {
  const t = useT();
  const selectId = useId();
  const errorId = useId();
  const [timeZone, setTimeZone] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);
  const [loadFailed, setLoadFailed] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [detected, setDetected] = useState<string | null>(null);

  useEffect(() => {
    setDetected(detectBrowserTimeZone());
  }, []);

  const zones = useMemo<string[]>(() => {
    const listed =
      typeof Intl.supportedValuesOf === "function"
        ? Intl.supportedValuesOf("timeZone")
        : [];
    return listed.length > 0 ? listed : detected ? [detected] : [];
  }, [detected]);

  // The option matching the saved value once both sides share one spelling.
  // A saved value no option matches even then (a zone this browser's list
  // does not carry at all) is appended raw, so the select can still show it
  // instead of silently displaying the auto-detect entry.
  const selectedZone = useMemo<string>(() => {
    if (timeZone === null) return "";
    const target = canonicalTimeZoneName(timeZone);
    return (
      zones.find((zone) => canonicalTimeZoneName(zone) === target) ?? timeZone
    );
  }, [timeZone, zones]);

  const options = useMemo<string[]>(
    () =>
      selectedZone !== "" && !zones.includes(selectedZone)
        ? [...zones, selectedZone]
        : zones,
    [selectedZone, zones],
  );

  const load = useCallback(async () => {
    setLoadFailed(false);
    const response = await fetch("/api/settings/timezone");
    if (!response.ok) {
      setLoadFailed(true);
      return;
    }
    const data = (await response.json()) as { timezone?: unknown };
    setTimeZone(typeof data.timezone === "string" ? data.timezone : null);
    setLoaded(true);
  }, []);

  useEffect(() => {
    void load().catch(() => setLoadFailed(true));
  }, [load]);

  const onChange = async (event: React.ChangeEvent<HTMLSelectElement>) => {
    const next = event.target.value;
    setIsSaving(true);
    setError(null);
    try {
      const response = await fetch("/api/settings/timezone", {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ timezone: next === "" ? null : next }),
      });
      const data = (await response.json()) as { timezone?: unknown };
      if (!response.ok) {
        setError(t("settings.errors.updateTimezone"));
        return;
      }
      setTimeZone(typeof data.timezone === "string" ? data.timezone : null);
    } catch {
      setError(t("settings.errors.updateTimezone"));
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <section className={Section}>
      <h2 className={SectionTitle}>{t("settings.timezone.title")}</h2>
      <p className="mb-4 text-sm text-muted-foreground">
        {t("settings.timezone.description")}
      </p>
      {loadFailed ? (
        <button
          type="button"
          className="text-sm underline"
          onClick={() => void load()}
        >
          {t("common.retry")}
        </button>
      ) : (
        <>
          <label
            htmlFor={selectId}
            className="mb-2 block text-[13px] font-semibold"
          >
            {t("settings.timezone.label")}
          </label>
          <select
            id={selectId}
            value={loaded ? selectedZone : ""}
            disabled={!loaded || isSaving}
            onChange={(event) => void onChange(event)}
            aria-invalid={error ? true : undefined}
            aria-describedby={error ? errorId : undefined}
            className="h-10 w-full max-w-[320px] cursor-pointer rounded-md border bg-background px-3 text-sm text-foreground disabled:cursor-not-allowed disabled:opacity-60"
          >
            <option value="" className="bg-card">
              {t("settings.timezone.autoDetect")}
            </option>
            {options.map((zone) => (
              <option key={zone} value={zone} className="bg-card">
                {zone}
              </option>
            ))}
          </select>
          {timeZone === null && detected && (
            <p className="mt-2 text-sm text-muted-foreground">
              {t("settings.timezone.detected", { zone: detected })}
            </p>
          )}
          {error && (
            <p id={errorId} className="mt-2 text-sm text-[var(--danger)]">
              {error}
            </p>
          )}
        </>
      )}
    </section>
  );
}
