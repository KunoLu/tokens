import { getCloudflareContext } from "@opennextjs/cloudflare";

/**
 * Cloudflare Rate Limiting binding (wrangler `ratelimits`, namespace_id
 * `"1001"` — a stable account-unique integer from Worker config, not a
 * provisioned secret). Guards register / login / forgot-password /
 * resend-verification: 10 requests per 60s per key.
 *
 * Missing Worker context or a missing binding (local `next dev`) skips the
 * check. A bound limiter that throws fail-closes so the caller returns 429.
 */
export async function authRateLimitAllowed(request: Request): Promise<boolean> {
  let limiter:
    | { limit(options: { key: string }): Promise<{ success: boolean }> }
    | undefined;
  try {
    limiter = getCloudflareContext().env.AUTH_RATE_LIMITER as typeof limiter;
  } catch {
    // Not on Workers — no binding runtime.
    return true;
  }
  if (!limiter) {
    return true;
  }
  try {
    const key = request.headers.get("CF-Connecting-IP") ?? "0.0.0.0";
    const { success } = await limiter.limit({ key });
    return success;
  } catch (error) {
    console.error("[auth] AUTH_RATE_LIMITER.limit failed", error);
    return false;
  }
}
