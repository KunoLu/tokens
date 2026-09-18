/**
 * In-process fixed-window limiter for register / login / forgot-password /
 * resend-verification: 10 requests per 60 seconds per client IP. Replaced the
 * Cloudflare Rate Limiting binding when the Workers layer was removed.
 *
 * The client key is X-Forwarded-For ONLY, and only because the reverse proxy
 * overwrites it — a client must never be able to pick its own bucket. Do NOT
 * honor `CF-Connecting-IP` here: on the self-host path there is no Cloudflare
 * in front, so a direct client could forge it and rotate through fresh
 * buckets.
 *
 * ponytail: counters are per-process; a second instance would get its own
 * bucket map. If the deployment ever scales beyond one process, this needs a
 * shared store.
 */
const WINDOW_MS = 60_000;
const LIMIT = 10;

const buckets = new Map<string, { count: number; resetAt: number }>();

function clientKey(request: Request): string {
  const forwarded = request.headers.get("x-forwarded-for")?.split(",")[0]?.trim();
  return forwarded || "0.0.0.0";
}

const MAX_BUCKETS = 10_000;

export async function authRateLimitAllowed(request: Request): Promise<boolean> {
  const now = Date.now();
  const key = clientKey(request);
  let bucket = buckets.get(key);
  if (!bucket || bucket.resetAt <= now) {
    if (buckets.size >= MAX_BUCKETS) {
      // At capacity: evict expired entries, and if the map is still full fail
      // closed rather than grow without bound — a flood of distinct IPs must
      // not grow memory or turn every request into a full-map scan.
      for (const [k, b] of buckets) {
        if (b.resetAt <= now) buckets.delete(k);
      }
      if (buckets.size >= MAX_BUCKETS) return false;
    }
    bucket = { count: 0, resetAt: now + WINDOW_MS };
    buckets.set(key, bucket);
  }
  bucket.count += 1;

  return bucket.count <= LIMIT;
}
