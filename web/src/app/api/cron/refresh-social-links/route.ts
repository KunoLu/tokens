import { timingSafeEqual } from "crypto";
import { NextResponse } from "next/server";
import { getCloudflareContext } from "@opennextjs/cloudflare";
import { countUsers, refreshAllSocialLinks } from "@/lib/cron/refreshSocialLinks";
import { deleteExpiredEmailTokens } from "@/lib/auth/emailTokens";

export const dynamic = "force-dynamic";

function safeEqual(a: string, b: string): boolean {
  const bufferA = Buffer.from(a);
  const bufferB = Buffer.from(b);
  return bufferA.length === bufferB.length && timingSafeEqual(bufferA, bufferB);
}

/**
 * Keep the sync running after the 202 has been sent.
 *
 * On Workers an isolate stops executing once its response is returned, so the
 * promise has to be handed to `waitUntil` or the run is truncated partway
 * through the user list. Outside Workers there is no such constraint and the
 * floating promise settles on its own.
 */
function runInBackground(work: Promise<unknown>): void {
  try {
    getCloudflareContext().ctx.waitUntil(work);
    return;
  } catch {
    // Not on Workers.
  }

  void work;
}

/**
 * Daily refresh of every user's GitHub social-links snapshot (the profile
 * page's social-links row), followed by a sweep of expired email verification
 * tokens. Triggered by the Worker's cron trigger, or over HTTP for manual
 * runs; guarded by CRON_SECRET. Responds immediately and syncs in the
 * background so proxy timeouts can't cut the run short.
 */
export async function POST(request: Request) {
  const secret = process.env.CRON_SECRET;
  if (!secret) {
    return NextResponse.json(
      { error: "CRON_SECRET is not configured" },
      { status: 503 },
    );
  }

  const authorization = request.headers.get("authorization") ?? "";
  if (!safeEqual(authorization, `Bearer ${secret}`)) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  const total = await countUsers();

  runInBackground(
    refreshAllSocialLinks()
      .then(async ({ users }) => {
        const expiredTokens = await deleteExpiredEmailTokens();
        console.log(
          `[cron] refresh-social-links: synced ${users} users, deleted ${expiredTokens} expired email tokens`,
        );
      })
      .catch((error: unknown) => {
        console.error("[cron] refresh-social-links failed", error);
      }),
  );

  return NextResponse.json({ accepted: true, users: total }, { status: 202 });
}
