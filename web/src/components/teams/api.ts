/**
 * Thin fetch wrapper for the /api/teams* mutations. Returns `null` on
 * success, otherwise a locale-mapped server error for toasts.
 *
 * The translator comes from the caller (every caller is a component inside
 * `LocaleProvider`): the URL — not a cookie — is what decided the language,
 * so a client module cannot read it back on its own.
 */
import { localizeServerError, localizeServerErrorList } from "@/lib/i18n/serverError";
import type { Translator } from "@/lib/i18n/translate";

export async function teamApi(
  t: Translator,
  path: string,
  init?: { method?: "POST" | "PATCH" | "PUT" | "DELETE"; body?: unknown }
): Promise<string | null> {
  try {
    const res = await fetch(path, {
      method: init?.method ?? "POST",
      headers: init?.body !== undefined ? { "content-type": "application/json" } : undefined,
      body: init?.body !== undefined ? JSON.stringify(init.body) : undefined,
    });
    if (res.ok) return null;
    const data: { error?: unknown; details?: unknown } = await res
      .json()
      .catch(() => ({}));
    if (Array.isArray(data.details) && data.details.length > 0) {
      return localizeServerErrorList(
        t,
        data.details.filter((item): item is string => typeof item === "string")
      ).join("; ");
    }
    return typeof data.error === "string"
      ? localizeServerError(t, data.error)
      : localizeServerError(t, `HTTP ${res.status}`);
  } catch {
    return localizeServerError(t, "Network error");
  }
}
