"use client";

import { useCallback, useEffect, useId, useMemo, useState } from "react";
import { useT } from "@/components/layout/LocaleProvider";
import { detectBrowserTimeZone } from "@/lib/timezone";

const Section = "mb-6 rounded-xl border bg-card p-6";
const SectionTitle = "mb-4 text-lg font-semibold text-foreground";

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
    if (typeof Intl.supportedValuesOf === "function") {
      return Intl.supportedValuesOf("timeZone");
    }
    return detected ? [detected] : [];
  }, [detected]);

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
            value={loaded ? (timeZone ?? "") : ""}
            disabled={!loaded || isSaving}
            onChange={(event) => void onChange(event)}
            aria-invalid={error ? true : undefined}
            aria-describedby={error ? errorId : undefined}
            className="h-10 w-full max-w-[320px] cursor-pointer rounded-md border bg-background px-3 text-sm text-foreground disabled:cursor-not-allowed disabled:opacity-60"
          >
            <option value="" className="bg-card">
              {t("settings.timezone.autoDetect")}
            </option>
            {zones.map((zone) => (
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
