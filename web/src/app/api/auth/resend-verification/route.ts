import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { db, users } from "@/lib/db";
import { hasAllowedOrigin } from "@/lib/auth/requestSession";
import { getSession } from "@/lib/auth/session";
import { authRateLimitAllowed } from "@/lib/auth/rateLimit";
import { issueEmailToken } from "@/lib/auth/emailTokens";
import { sendVerificationEmail } from "@/lib/email/send";

/**
 * Re-issue a verify_email token for the signed-in user.
 *
 * Parent design: fire-and-forget delivery must be recoverable via
 * "resend verification email". Forgot-password cannot stand in — it only
 * issues reset_password tokens and never sets email_verified_at.
 *
 * Session signature is unchanged: we look email_verified_at up from the DB.
 */
export async function POST(request: Request) {
  try {
    if (!hasAllowedOrigin(request)) {
      return NextResponse.json({ error: "Forbidden" }, { status: 403 });
    }
    if (!(await authRateLimitAllowed(request))) {
      return NextResponse.json({ error: "Too many requests" }, { status: 429 });
    }

    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: "Not authenticated" }, { status: 401 });
    }

    const rows = await db
      .select({
        email: users.email,
        emailVerifiedAt: users.emailVerifiedAt,
        bannedAt: users.bannedAt,
      })
      .from(users)
      .where(eq(users.id, session.id))
      .limit(1);
    const user = rows[0];
    if (!user || user.bannedAt) {
      return NextResponse.json({ error: "Account banned" }, { status: 403 });
    }
    if (!user.email) {
      return NextResponse.json({ error: "No email on this account" }, { status: 400 });
    }
    if (user.emailVerifiedAt) {
      return NextResponse.json({ ok: true });
    }

    const link = await issueEmailToken(session.id, "verify_email");
    sendVerificationEmail(user.email, link);
    return NextResponse.json({ ok: true });
  } catch (error) {
    console.error("Resend-verification error:", error);
    return NextResponse.json(
      { error: "Failed to resend verification email" },
      { status: 500 }
    );
  }
}
