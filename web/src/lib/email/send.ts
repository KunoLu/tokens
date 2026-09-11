import { getCloudflareContext } from "@opennextjs/cloudflare";

/**
 * Transactional email through Resend's HTTP API — no SDK, one fetch.
 *
 * Delivery is deliberately decoupled from the request that triggered it: a
 * failed send (or a missing secret) logs and resolves, it never rolls back
 * the user or token rows the caller already committed. A lost verification
 * email is recoverable through POST /api/auth/resend-verification after sign-in.
 * Forgot-password only issues reset_password tokens and cannot verify email.
 */
const RESEND_API_URL = "https://api.resend.com/emails";

async function sendEmail(to: string, subject: string, text: string): Promise<void> {
  const apiKey = process.env.RESEND_API_KEY;
  const from = process.env.EMAIL_FROM;
  if (!apiKey || !from) {
    console.error(`[email] RESEND_API_KEY/EMAIL_FROM not configured; skipped "${subject}" to ${to}`);
    return;
  }

  const response = await fetch(RESEND_API_URL, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ from, to: [to], subject, text }),
  });
  if (!response.ok) {
    console.error(`[email] Resend rejected "${subject}" to ${to}: ${response.status}`);
  }
}

/**
 * Hand the send to the isolate so it can outlive the response; outside
 * Workers the floating promise settles on its own. Never throws.
 */
function sendInBackground(work: Promise<void>): void {
  const guarded = work.catch((error: unknown) => {
    console.error("[email] send failed", error);
  });
  try {
    getCloudflareContext().ctx.waitUntil(guarded);
    return;
  } catch {
    // Not on Workers.
  }
  void guarded;
}

export function sendVerificationEmail(to: string, link: string): void {
  sendInBackground(
    sendEmail(
      to,
      "Verify your Tokens email",
      [
        "Welcome to Tokens.",
        "",
        "Confirm this email address by opening the link below (valid for 24 hours):",
        link,
        "",
        "If you did not create an account, you can ignore this message.",
      ].join("\n")
    )
  );
}

export function sendPasswordResetEmail(to: string, link: string): void {
  sendInBackground(
    sendEmail(
      to,
      "Reset your Tokens password",
      [
        "A password reset was requested for your Tokens account.",
        "",
        "Choose a new password by opening the link below (valid for 1 hour):",
        link,
        "",
        "If you did not request this, you can ignore this message — your password stays unchanged.",
      ].join("\n")
    )
  );
}
