import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { db, users } from "@/lib/db";
import { hasAllowedOrigin } from "@/lib/auth/requestSession";
import {
  consumeVerifyTokenAndMarkVerified,
  findEmailToken,
} from "@/lib/auth/emailTokens";
import { linkPendingInvitationsForEmail } from "@/lib/teams/service";

async function backfillPendingInvites(
  userId: string,
  email: string | null | undefined
) {
  if (!email) return;
  try {
    await linkPendingInvitationsForEmail(userId, email);
  } catch (error: unknown) {
    console.error("Invite backfill failed", error);
  }
}

export async function POST(request: Request) {
  try {
    if (!hasAllowedOrigin(request)) {
      return NextResponse.json({ error: "Forbidden" }, { status: 403 });
    }

    const body = await request.json().catch(() => ({}));
    const token = typeof body.token === "string" ? body.token : "";

    const userId = await consumeVerifyTokenAndMarkVerified(token);
    if (userId) {
      const rows = await db
        .select({ email: users.email })
        .from(users)
        .where(eq(users.id, userId))
        .limit(1);
      await backfillPendingInvites(userId, rows[0]?.email);
      return NextResponse.json({ ok: true });
    }

    // Replay after a lost 200: token already consumed, user already verified.
    // Retry backfill here so a transient first-pass failure is not permanent.
    const existing = await findEmailToken(token, "verify_email");
    if (existing?.consumedAt) {
      const rows = await db
        .select({ email: users.email, emailVerifiedAt: users.emailVerifiedAt })
        .from(users)
        .where(eq(users.id, existing.userId))
        .limit(1);
      if (rows[0]?.emailVerifiedAt) {
        await backfillPendingInvites(existing.userId, rows[0].email);
        return NextResponse.json({ ok: true });
      }
    }

    return NextResponse.json(
      { error: "Invalid or expired verification link" },
      { status: 400 }
    );
  } catch (error) {
    console.error("Verify-email error:", error);
    return NextResponse.json({ error: "Failed to verify email" }, { status: 500 });
  }
}
