import { NextResponse } from "next/server";
import { sql } from "drizzle-orm";
import { db, users } from "@/lib/db";
import { hasAllowedOrigin } from "@/lib/auth/requestSession";
import { authRateLimitAllowed } from "@/lib/auth/rateLimit";
import { hashPassword, verifyPassword } from "@/lib/auth/password";
import { createSession, setSessionCookie } from "@/lib/auth/session";

const INVALID_CREDENTIALS = "Invalid email or password";

// Unknown emails and legacy accounts without a password hash must cost the
// same PBKDF2 as a real verify, otherwise response timing reveals which
// emails are registered. Derived once per isolate at module load, never
// stored anywhere.
const DUMMY_HASH_PROMISE = hashPassword("timing-equalization-placeholder");

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
    const password = typeof body.password === "string" ? body.password : "";
    if (!email || !password) {
      return NextResponse.json({ error: INVALID_CREDENTIALS }, { status: 401 });
    }

    const rows = await db
      .select()
      .from(users)
      .where(sql`lower(${users.email}) = ${email}`)
      .limit(1);
    const user = rows[0];

    const passwordHash = user?.passwordHash ?? (await DUMMY_HASH_PROMISE);
    const passwordOk = await verifyPassword(password, passwordHash);
    // One 401 covers "no such email", "legacy OAuth account without a
    // password" and "wrong password" so the endpoint cannot enumerate
    // registered emails.
    if (!user?.passwordHash || !passwordOk) {
      return NextResponse.json({ error: INVALID_CREDENTIALS }, { status: 401 });
    }

    // Distinct from a wrong password: a banned account should not sit in a
    // "wrong password" retry loop.
    if (user.bannedAt) {
      return NextResponse.json({ error: "Account banned" }, { status: 403 });
    }

    const sessionToken = await createSession(user.id, {
      userAgent: request.headers.get("user-agent") ?? undefined,
    });
    await setSessionCookie(sessionToken);

    return NextResponse.json({ ok: true });
  } catch (error) {
    console.error("Login error:", error);
    return NextResponse.json({ error: "Failed to log in" }, { status: 500 });
  }
}
