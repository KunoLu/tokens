import { timingSafeEqual } from "crypto";
import { NextResponse } from "next/server";
import { refreshAllSocialLinks } from "@/lib/cron/refreshSocialLinks";
import { deleteExpiredEmailTokens } from "@/lib/auth/emailTokens";
import { expireInvitations } from "@/lib/teams/service";

export const dynamic = "force-dynamic";

function safeEqual(a: string, b: string): boolean {
  const bufferA = Buffer.from(a);
  const bufferB = Buffer.from(b);
  return bufferA.length === bufferB.length && timingSafeEqual(bufferA, bufferB);
}

/**
 * Daily maintenance for the self-hosted deployment — the Node replacement for
 * the Worker cron trigger. Refreshes social-links snapshots, sweeps expired
 * email-verification tokens, and expires overdue team invitations.
 *
 * Every job runs to completion inside the request and the response is the
 * scheduler's retry signal: 200 only when all three succeeded, 500 when any
 * failed. A failing job does not skip the others (they are independent), and
 * all three are idempotent sweeps, so retrying after a partial failure is
 * safe. Trigger it from the same host (system cron -> loopback) so no public
 * proxy timeout can cut a run short.
 *
 * Scenarios: web/features/maintenance-cron.feature.
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

  const [socialLinks, emailTokens, invitations] = await Promise.all([
    refreshAllSocialLinks()
      .then(({ users }) => ({ ok: true as const, refreshedUsers: users }))
      .catch((error: unknown) => {
        console.error("[cron] refreshAllSocialLinks failed", error);
        return { ok: false as const };
      }),
    deleteExpiredEmailTokens()
      .then((deleted) => ({ ok: true as const, expiredEmailTokens: deleted }))
      .catch((error: unknown) => {
        console.error("[cron] deleteExpiredEmailTokens failed", error);
        return { ok: false as const };
      }),
    expireInvitations()
      .then((expired) => ({ ok: true as const, expiredInvitations: expired }))
      .catch((error: unknown) => {
        console.error("[cron] expireInvitations failed", error);
        return { ok: false as const };
      }),
  ]);

  if (!socialLinks.ok || !emailTokens.ok || !invitations.ok) {
    return NextResponse.json(
      { error: "One or more cron jobs failed" },
      { status: 500 },
    );
  }

  return NextResponse.json({
    refreshedUsers: socialLinks.refreshedUsers,
    expiredEmailTokens: emailTokens.expiredEmailTokens,
    expiredInvitations: invitations.expiredInvitations,
  });
}
