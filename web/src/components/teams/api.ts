/**
 * Thin fetch wrapper for the /api/teams* mutations. Returns `null` on
 * success, otherwise the server's error message — callers toast it as-is
 * because TeamError messages are already user-facing English.
 */
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
      return data.details.join("; ");
    }
    return typeof data.error === "string" ? data.error : `HTTP ${res.status}`;
  } catch {
    return "Network error";
  }
}
