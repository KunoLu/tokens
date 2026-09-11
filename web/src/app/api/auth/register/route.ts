import { NextResponse } from "next/server";
import { sql } from "drizzle-orm";
import { db, users } from "@/lib/db";
import { hasAllowedOrigin } from "@/lib/auth/requestSession";
import { authRateLimitAllowed } from "@/lib/auth/rateLimit";
import { hashPassword, validatePassword } from "@/lib/auth/password";
import { createSession, setSessionCookie } from "@/lib/auth/session";
import { issueEmailToken } from "@/lib/auth/emailTokens";
import { sendVerificationEmail } from "@/lib/email/send";
import { isValidGitHubUsername } from "@/lib/validation/username";

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

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
    const username = typeof body.username === "string" ? body.username.trim() : "";
    const password = typeof body.password === "string" ? body.password : "";

    if (!EMAIL_PATTERN.test(email) || email.length > 255) {
      return NextResponse.json({ error: "Invalid email address" }, { status: 400 });
    }
    if (!isValidGitHubUsername(username)) {
      return NextResponse.json(
        { error: "Username must be 1-39 characters of letters, digits or hyphens" },
        { status: 400 }
      );
    }
    const passwordCheck = validatePassword(password);
    if (passwordCheck.error) {
      return NextResponse.json(
        { error: passwordCheck.error, details: passwordCheck.details },
        { status: 400 }
      );
    }

    const conflicts = await db
      .select({
        emailTaken: sql<boolean>`bool_or(lower(${users.email}) = ${email})`,
        usernameTaken: sql<boolean>`bool_or(lower(${users.username}) = ${username.toLowerCase()})`,
      })
      .from(users)
      .where(
        sql`lower(${users.email}) = ${email} OR lower(${users.username}) = ${username.toLowerCase()}`
      );
    const conflict = conflicts[0];
    if (conflict?.emailTaken) {
      return NextResponse.json({ error: "Email is already registered" }, { status: 409 });
    }
    if (conflict?.usernameTaken) {
      return NextResponse.json({ error: "Username is already taken" }, { status: 409 });
    }

    const passwordHash = await hashPassword(password);
    let userId: string;
    try {
      const inserted = await db
        .insert(users)
        .values({ email, username, displayName: username, passwordHash })
        .returning({ id: users.id });
      userId = inserted[0].id;
    } catch {
      // Unique-index race between the conflict check and the insert.
      return NextResponse.json({ error: "Email or username is already taken" }, { status: 409 });
    }

    // Unverified users hold a session from the start; email_verified_at only
    // records when the emailed link was followed.
    const sessionToken = await createSession(userId, {
      userAgent: request.headers.get("user-agent") ?? undefined,
    });
    await setSessionCookie(sessionToken);

    const link = await issueEmailToken(userId, "verify_email");
    sendVerificationEmail(email, link);

    return NextResponse.json({ ok: true });
  } catch (error) {
    console.error("Register error:", error);
    return NextResponse.json({ error: "Failed to register" }, { status: 500 });
  }
}
