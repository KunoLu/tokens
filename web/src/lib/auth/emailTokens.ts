import { and, eq, gt, isNull, lt } from "drizzle-orm";
import { db, emailVerificationTokens, sessions, users } from "@/lib/db";
import { generateApiToken, hashToken } from "./utils";

export type EmailTokenPurpose = "verify_email" | "reset_password";

const TOKEN_TTL_MS: Record<EmailTokenPurpose, number> = {
  verify_email: 24 * 60 * 60 * 1000,
  reset_password: 60 * 60 * 1000,
};

function emailLink(path: string, token: string): string {
  const baseUrl = process.env.NEXT_PUBLIC_URL || "http://localhost:3000";
  return `${baseUrl}${path}?token=${token}`;
}

/** Works for both the pool client and a transaction handle. */
type EmailTokenDb = Pick<typeof db, "select" | "insert" | "update" | "delete">;

/**
 * Single-use consume: marks the matching live token consumed and returns the
 * owning user id exactly once. Invalid, expired, wrong-purpose, or
 * already-consumed tokens all return null. Exported flows call this inside
 * their own transaction so the consume commits or rolls back together with
 * the follow-up writes.
 */
async function consumeEmailTokenIn(
  tx: EmailTokenDb,
  token: string,
  purpose: EmailTokenPurpose,
  now: Date
): Promise<string | null> {
  const rows = await tx
    .update(emailVerificationTokens)
    .set({ consumedAt: now })
    .where(
      and(
        eq(emailVerificationTokens.tokenHash, hashToken(token)),
        eq(emailVerificationTokens.purpose, purpose),
        isNull(emailVerificationTokens.consumedAt),
        gt(emailVerificationTokens.expiresAt, now)
      )
    )
    .returning({ userId: emailVerificationTokens.userId });
  return rows[0]?.userId ?? null;
}

/**
 * Issue a token and return the emailed link. Only the SHA-256 hash is stored.
 * Issuing consumes any still-active tokens for the same user+purpose, so at
 * most one link works at a time. The user row is locked first: without it,
 * two concurrent issues could both survive with an unconsumed token.
 */
export async function issueEmailToken(
  userId: string,
  purpose: EmailTokenPurpose
): Promise<string> {
  const now = new Date();
  // generateApiToken's tt_ prefix is only a display convention; stripping it
  // keeps the emailed URL free of a marker that means "API token" elsewhere.
  const token = generateApiToken().slice(3);

  await db.transaction(async (tx) => {
    const locked = await tx
      .select({ id: users.id })
      .from(users)
      .where(eq(users.id, userId))
      .for("update")
      .limit(1);
    if (!locked[0]) {
      throw new Error(`Cannot issue ${purpose} token for missing user ${userId}`);
    }

    await tx
      .update(emailVerificationTokens)
      .set({ consumedAt: now })
      .where(
        and(
          eq(emailVerificationTokens.userId, userId),
          eq(emailVerificationTokens.purpose, purpose),
          isNull(emailVerificationTokens.consumedAt)
        )
      );

    await tx.insert(emailVerificationTokens).values({
      userId,
      tokenHash: hashToken(token),
      purpose,
      expiresAt: new Date(now.getTime() + TOKEN_TTL_MS[purpose]),
    });
  });

  return emailLink(purpose === "verify_email" ? "/verify-email" : "/reset-password", token);
}

/**
 * Reset flow, atomic: consume the reset token, set the new password hash, and
 * delete every web session in one transaction — a crash cannot leave the
 * token spent while the old password still works. A reset is how a
 * compromised account is recovered, so every existing web session dies with
 * the old password. API tokens are user-managed secrets and survive.
 *
 * The caller hashes the password BEFORE calling: PBKDF2 is slow CPU work and
 * must not run inside the transaction.
 */
export async function consumeResetTokenAndSetPassword(
  token: string,
  passwordHash: string
): Promise<string | null> {
  const now = new Date();
  return db.transaction(async (tx) => {
    const userId = await consumeEmailTokenIn(tx, token, "reset_password", now);
    if (!userId) {
      return null;
    }
    await tx
      .update(users)
      .set({ passwordHash, updatedAt: now })
      .where(eq(users.id, userId));
    await tx.delete(sessions).where(eq(sessions.userId, userId));
    return userId;
  });
}

/**
 * Verify flow, atomic: consume the verify token and mark the email verified
 * in one transaction. Replay after a lost 200 is handled by the caller via
 * findEmailToken — this function is strictly single-use.
 */
export async function consumeVerifyTokenAndMarkVerified(
  token: string
): Promise<string | null> {
  const now = new Date();
  return db.transaction(async (tx) => {
    const userId = await consumeEmailTokenIn(tx, token, "verify_email", now);
    if (!userId) {
      return null;
    }
    await tx
      .update(users)
      .set({ emailVerifiedAt: now, updatedAt: now })
      .where(and(eq(users.id, userId), isNull(users.emailVerifiedAt)));
    return userId;
  });
}

/**
 * Look up a token regardless of consumed/expiry. Used so a retry after a
 * successful verify (lost response) can still return 200 when the user is
 * already verified — the consume flows above are single-use.
 */
export async function findEmailToken(
  token: string,
  purpose: EmailTokenPurpose
): Promise<{ userId: string; consumedAt: Date | null } | null> {
  const rows = await db
    .select({
      userId: emailVerificationTokens.userId,
      consumedAt: emailVerificationTokens.consumedAt,
    })
    .from(emailVerificationTokens)
    .where(
      and(
        eq(emailVerificationTokens.tokenHash, hashToken(token)),
        eq(emailVerificationTokens.purpose, purpose)
      )
    )
    .limit(1);
  return rows[0] ?? null;
}

/** Daily cron sweep; expired rows are worthless once expired. */
export async function deleteExpiredEmailTokens(): Promise<number> {
  const rows = await db
    .delete(emailVerificationTokens)
    .where(lt(emailVerificationTokens.expiresAt, new Date()))
    .returning({ id: emailVerificationTokens.id });
  return rows.length;
}
