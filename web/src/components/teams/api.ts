/**
 * Thin fetch wrapper for the /api/teams* mutations. Returns `null` on
 * success, otherwise a locale-mapped server error for toasts.
 */
import { localizeServerErrorFromCookie } from "@/lib/i18n";

export async function teamApi(
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
      return data.details
        .filter((item): item is string => typeof item === "string")
        .map(localizeServerErrorFromCookie)
        .join("; ");
    }
    return typeof data.error === "string"
      ? localizeServerErrorFromCookie(data.error)
      : localizeServerErrorFromCookie(`HTTP ${res.status}`);
  } catch {
    return localizeServerErrorFromCookie("Network error");
  }
}
