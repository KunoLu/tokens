import { NextResponse } from "next/server";
import { sql } from "drizzle-orm";
import { db, users } from "@/lib/db";
import { hasAllowedOrigin } from "@/lib/auth/requestSession";
import { authRateLimitAllowed } from "@/lib/auth/rateLimit";
import { issueEmailToken } from "@/lib/auth/emailTokens";
import { sendPasswordResetEmail } from "@/lib/email/send";

/**
 * Always 200: the response must not reveal whether an email is registered.
 * The reset token is only written (and the email only sent) when the address
 * belongs to a live, unbanned account.
 */
export async function POST(request: Request) {
  try {
    if (!hasAllowedOrigin(request)) {
      return NextResponse.json({ error: "Forbidden" }, { status: 403 });
    }
    if (!(await authRateLimitAllowed(request))) {
      return NextResponse.json({ error: "Too many requests" }, { status: 429 });
    }

    const body = await request.json().catch(() => ({}));
    const email = typeof body.email === "string" ? body.email.trim().toLowerCase() : "";

    if (email) {
      const rows = await db
        .select({ id: users.id, bannedAt: users.bannedAt })
        .from(users)
        .where(sql`lower(${users.email}) = ${email}`)
        .limit(1);
      const user = rows[0];
      if (user && !user.bannedAt) {
        const link = await issueEmailToken(user.id, "reset_password");
        sendPasswordResetEmail(email, link);
      }
    }

    return NextResponse.json({ ok: true });
  } catch (error) {
    console.error("Forgot-password error:", error);
    return NextResponse.json({ error: "Failed to process request" }, { status: 500 });
  }
}
