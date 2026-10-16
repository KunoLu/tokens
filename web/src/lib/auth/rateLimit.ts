/**
 * Fixed-window limiter for register / login / forgot-password /
 * resend-verification: 10 requests per 60 seconds per client IP.
 *
 * Default store is the in-process map. Docker/self-host does not set Upstash
 * env, so that path is unchanged. Both UPSTASH_REDIS_REST_URL and
 * UPSTASH_REDIS_REST_TOKEN must be set before the same window is enforced in
 * Upstash instead. Keys are prefixed `tokens:auth:${VERCEL_ENV||local}` so
 * Preview and Production do not share a bucket on one Redis. A Redis error
 * or timeout fails closed and does not fall through to the process map —
 * that would split the budget across isolates.
 *
 * Client IP, default: first X-Forwarded-For hop, else 0.0.0.0. The self-host
 * reverse proxy overwrites XFF; a client must not pick its own bucket. Do NOT
 * honor CF-Connecting-IP on that path.
 *
 * VERCEL=1 (set by the platform) or TRUST_PROXY=1|true|yes: prefer
 * x-vercel-forwarded-for, then x-real-ip. Vercel writes those; an outer proxy
 * can rewrite x-forwarded-for, so XFF is only the fallback. Still never
 * CF-Connecting-IP.
 */
import { Ratelimit } from "@upstash/ratelimit";
import { Redis } from "@upstash/redis";

const WINDOW_MS = 60_000;
const LIMIT = 10;
const MAX_BUCKETS = 10_000;
const UPSTASH_TIMEOUT_MS = 2_000;

const buckets = new Map<string, { count: number; resetAt: number }>();

function firstHop(header: string | null): string {
  return header?.split(",")[0]?.trim() || "";
}

function trustPlatformIp(): boolean {
  const trust = process.env.TRUST_PROXY?.trim().toLowerCase();
  return (
    process.env.VERCEL === "1" ||
    trust === "1" ||
    trust === "true" ||
    trust === "yes"
  );
}

export function clientIp(request: Request): string {
  if (trustPlatformIp()) {
    const platform =
      firstHop(request.headers.get("x-vercel-forwarded-for")) ||
      request.headers.get("x-real-ip")?.trim() ||
      "";
    if (platform) return platform;
  }
  return firstHop(request.headers.get("x-forwarded-for")) || "0.0.0.0";
}

function upstashConfigured(): boolean {
  return Boolean(
    process.env.UPSTASH_REDIS_REST_URL?.trim() &&
      process.env.UPSTASH_REDIS_REST_TOKEN?.trim(),
  );
}

let shared: Ratelimit | undefined;
let sharedPrefix: string | undefined;

function upstashLimiter(): Ratelimit {
  const url = process.env.UPSTASH_REDIS_REST_URL?.trim();
  const token = process.env.UPSTASH_REDIS_REST_TOKEN?.trim();
  if (!url || !token) {
    throw new Error("Upstash rate limiter is not configured");
  }
  const prefix = `tokens:auth:${process.env.VERCEL_ENV || "local"}`;
  // Env is fixed in production. Tests and a module loaded before env is set
  // must not keep a client whose keys belong to another environment.
  if (!shared || sharedPrefix !== prefix) {
    shared = new Ratelimit({
      redis: new Redis({ url, token }),
      limiter: Ratelimit.fixedWindow(LIMIT, "60 s"),
      prefix,
      timeout: UPSTASH_TIMEOUT_MS,
      analytics: false,
    });
    sharedPrefix = prefix;
  }
  return shared;
}

function memoryAllowed(key: string): boolean {
  const now = Date.now();
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

async function sharedAllowed(key: string): Promise<boolean> {
  try {
    const result = await upstashLimiter().limit(key);
    // The library fail-opens on its own timeout. Auth limiting must not.
    if (result.reason === "timeout") {
      console.error("[rateLimit] Upstash timed out; denying");
      return false;
    }
    void result.pending.catch((error: unknown) => {
      console.error("[rateLimit] Upstash pending failed", error);
    });
    return result.success;
  } catch (error) {
    console.error("[rateLimit] Upstash limiter failed", error);
    return false;
  }
}

export async function authRateLimitAllowed(request: Request): Promise<boolean> {
  const key = clientIp(request);
  if (upstashConfigured()) return sharedAllowed(key);
  return memoryAllowed(key);
}
