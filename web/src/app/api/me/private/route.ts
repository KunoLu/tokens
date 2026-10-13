import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { getBearerToken } from "@/lib/auth/bearerToken";
import { authenticatePersonalToken } from "@/lib/auth/personalTokens";
import {
  issueReadTokenInTransaction,
  normalizeReadTokenName,
} from "@/lib/auth/readTokens";
import { db, users } from "@/lib/db";
import { PRIVATE_CACHE_CONTROL, revalidateUserVisibility } from "@/lib/privacy";

function tokenNameFromBody(body: unknown): unknown {
  if (!body || typeof body !== "object" || !("tokenName" in body)) return undefined;
  return body.tokenName;
}

/**
 * POST /api/me/private — `tokens login --private`.
 *
 * Switches the account behind the Bearer submit token to private and issues a
 * read token, returned once. Repeating it stays private and issues another
 * token. It never turns private mode off.
 */
export async function POST(request: Request) {
  try {
    const token = getBearerToken(request.headers.get("Authorization"));
    if (!token) {
      return NextResponse.json(
        { error: "Missing or invalid Authorization header" },
        { status: 401 }
      );
    }

    const auth = await authenticatePersonalToken(token);
    if (auth.status === "banned") {
      return NextResponse.json(
        { error: "This account has been banned for violating the platform rules" },
        { status: 403 }
      );
    }
    if (auth.status !== "valid") {
      return NextResponse.json(
        {
          error: auth.status === "expired" ? "API token has expired" : "Invalid API token",
        },
        { status: 401 }
      );
    }

    const readToken = await db.transaction(async (tx) => {
      await tx
        .update(users)
        .set({ isPrivate: true, updatedAt: new Date() })
        .where(eq(users.id, auth.userId));

      return issueReadTokenInTransaction(tx, {
        userId: auth.userId,
        name: normalizeReadTokenName(tokenNameFromBody(await request.json().catch(() => null))),
      });
    });

    revalidateUserVisibility(auth.username);

    return NextResponse.json(
      {
        private: true,
        readToken: {
          id: readToken.id,
          name: readToken.name,
          token: readToken.token,
          createdAt: readToken.createdAt,
        },
      },
      { headers: { "Cache-Control": PRIVATE_CACHE_CONTROL } }
    );
  } catch (error) {
    console.error("Enable private mode error:", error);
    return NextResponse.json({ error: "Failed to enable private mode" }, { status: 500 });
  }
}
