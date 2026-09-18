import { NextResponse } from "next/server";
import { hasAllowedOrigin } from "@/lib/auth/requestSession";
import { hashPassword, validatePassword } from "@/lib/auth/password";
import { consumeResetTokenAndSetPassword } from "@/lib/auth/emailTokens";

export async function POST(request: Request) {
  try {
    if (!hasAllowedOrigin(request)) {
      return NextResponse.json({ error: "Forbidden" }, { status: 403 });
    }

    const body = await request.json().catch(() => ({}));
    const token = typeof body.token === "string" ? body.token : "";
    const password = typeof body.password === "string" ? body.password : "";

    const passwordCheck = validatePassword(password);
    if (passwordCheck.error) {
      return NextResponse.json(
        { error: passwordCheck.error, details: passwordCheck.details },
        { status: 400 }
      );
    }

    // Hash before the transaction: PBKDF2 is slow CPU work and must not run
    // while the consume + user + session writes hold the transaction open.
    const passwordHash = await hashPassword(password);
    const userId = await consumeResetTokenAndSetPassword(token, passwordHash);
    if (!userId) {
      return NextResponse.json(
        { error: "Invalid or expired reset link" },
        { status: 400 }
      );
    }

    return NextResponse.json({ ok: true });
  } catch (error) {
    console.error("Reset-password error:", error);
    return NextResponse.json({ error: "Failed to reset password" }, { status: 500 });
  }
}
