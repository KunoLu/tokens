import { revalidatePath, revalidateTag } from "next/cache";
import { and, eq, gt, isNull, type SQL } from "drizzle-orm";
import { db, sessions, users } from "@/lib/db";
import { getBearerToken } from "@/lib/auth/bearerToken";
import { READ_TOKEN_PREFIX, verifyReadTokenForUser } from "@/lib/auth/readTokens";
import { hashToken } from "@/lib/auth/utils";
import {
  USERNAME_LOOKUP_LIMIT,
  getSingleUsernameMatch,
  normalizeUsernameCacheKey,
  revalidateUsernamePaths,
  usernameEqualsIgnoreCase,
} from "@/lib/db/usernameLookup";

const SESSION_COOKIE_NAME = "tt_session";

/** Private payloads must not enter a URL-keyed shared cache. */
export const PRIVATE_CACHE_CONTROL = "private, no-store, max-age=0";

/**
 * Rows a public ranking may count. Banned and private accounts are both
 * invisible there. A banned profile still renders its notice; a private one
 * does not, unless the caller is the owner or holds a read token.
 */
export function publicUserCondition(): SQL {
  return and(isNull(users.bannedAt), eq(users.isPrivate, false)) as SQL;
}

export function getReadTokenFromRequest(request: Request): string | null {
  const headerToken = getBearerToken(request.headers.get("Authorization"));
  if (headerToken?.startsWith(READ_TOKEN_PREFIX)) return headerToken;

  try {
    const queryToken = new URL(request.url).searchParams.get("token")?.trim();
    if (queryToken?.startsWith(READ_TOKEN_PREFIX)) return queryToken;
  } catch {
    // Unparseable URL: there is no query token to read.
  }

  return null;
}

function getCookie(request: Request, name: string): string | null {
  const header = request.headers.get("cookie");
  if (!header) return null;

  for (const part of header.split(";")) {
    const separator = part.indexOf("=");
    if (separator === -1) continue;
    if (part.slice(0, separator).trim() !== name) continue;
    const value = part.slice(separator + 1).trim();
    try {
      return decodeURIComponent(value);
    } catch {
      return value;
    }
  }

  return null;
}

async function getSessionUserIdFromRequest(request: Request): Promise<string | null> {
  const sessionToken = getCookie(request, SESSION_COOKIE_NAME);
  if (!sessionToken) return null;

  const [row] = await db
    .select({ userId: sessions.userId })
    .from(sessions)
    .innerJoin(users, eq(sessions.userId, users.id))
    .where(
      and(
        eq(sessions.tokenHash, hashToken(sessionToken)),
        gt(sessions.expiresAt, new Date()),
        isNull(users.bannedAt)
      )
    )
    .limit(1);

  return row?.userId ?? null;
}

export interface ViewerAccess {
  allowed: boolean;
  isOwner: boolean;
}

export async function getViewerAccess(
  request: Request,
  user: { id: string; isPrivate: boolean }
): Promise<ViewerAccess> {
  const isOwner = (await getSessionUserIdFromRequest(request)) === user.id;
  if (!user.isPrivate || isOwner) {
    return { allowed: true, isOwner };
  }

  const readToken = getReadTokenFromRequest(request);
  const allowed = readToken ? await verifyReadTokenForUser(readToken, user.id) : false;
  return { allowed, isOwner: false };
}

export function revalidateUserVisibility(username: string): void {
  const usernameCacheKey = normalizeUsernameCacheKey(username);
  try {
    revalidateTag("leaderboard", "max");
    revalidateTag(`user:${usernameCacheKey}`, "max");
    revalidateTag("user-rank", "max");
    revalidateTag(`user-rank:${usernameCacheKey}`, "max");
    revalidateTag(`embed-user:${usernameCacheKey}`, "max");
    revalidateTag(`embed-user:${usernameCacheKey}:tokens`, "max");
    revalidateTag(`embed-user:${usernameCacheKey}:cost`, "max");
    revalidateTag(`embed-contrib:${usernameCacheKey}`, "max");
    revalidateTag(`embed-today:${usernameCacheKey}`, "max");
    revalidatePath("/leaderboard");
    revalidatePath("/teamboard");
    revalidateUsernamePaths(username);
  } catch {
    // Cache invalidation is best-effort outside a request store.
  }
}

export async function setUserPrivacy(userId: string, isPrivate: boolean): Promise<void> {
  await db
    .update(users)
    .set({ isPrivate, updatedAt: new Date() })
    .where(eq(users.id, userId));
}

export async function getUserPrivacy(userId: string): Promise<boolean> {
  const [row] = await db
    .select({ isPrivate: users.isPrivate })
    .from(users)
    .where(eq(users.id, userId))
    .limit(1);
  return row?.isPrivate ?? false;
}

/** Whether a badge or embed request may see this account. */
export async function embedAudience(
  request: Request,
  username: string,
): Promise<"hidden" | "public" | "private"> {
  const rows = await db
    .select({
      id: users.id,
      isPrivate: users.isPrivate,
      bannedAt: users.bannedAt,
    })
    .from(users)
    .where(usernameEqualsIgnoreCase(username))
    .limit(USERNAME_LOOKUP_LIMIT);
  const user = getSingleUsernameMatch(rows, username);
  if (!user || user.bannedAt) return "hidden";
  if (!user.isPrivate) return "public";
  const access = await getViewerAccess(request, user);
  return access.allowed ? "private" : "hidden";
}
