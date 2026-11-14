import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { getSessionFromRequest } from "@/lib/auth/requestSession";
import { db, users } from "@/lib/db";
import { PRIVATE_CACHE_CONTROL } from "@/lib/privacy";
import { isValidIanaTimeZone } from "@/lib/timezone";

function timezoneFromBody(body: unknown): unknown {
  if (!body || typeof body !== "object" || !("timezone" in body)) return undefined;
  return body.timezone;
}

/**
 * GET   /api/settings/timezone  → { timezone: string | null }
 * PATCH /api/settings/timezone  { timezone: string | null } → { timezone: string | null }
 *
 * null means auto-detect from the browser. Browser session only, same as the
 * other settings mutations: a submit token must not be able to change how an
 * account's usage is presented.
 */
export async function GET(request: Request) {
  try {
    const session = await getSessionFromRequest(request, {
      allowAuthorizationHeader: false,
    });
    if (!session) {
      return NextResponse.json({ error: "Not authenticated" }, { status: 401 });
    }

    const [row] = await db
      .select({ timezone: users.timezone })
      .from(users)
      .where(eq(users.id, session.id))
      .limit(1);

    return NextResponse.json(
      { timezone: row?.timezone ?? null },
      { headers: { "Cache-Control": PRIVATE_CACHE_CONTROL } }
    );
  } catch (error) {
    console.error("Timezone read error:", error);
    return NextResponse.json({ error: "Failed to load time zone setting" }, { status: 500 });
  }
}

export async function PATCH(request: Request) {
  try {
    const session = await getSessionFromRequest(request, {
      allowAuthorizationHeader: false,
    });
    if (!session) {
      return NextResponse.json({ error: "Not authenticated" }, { status: 401 });
    }

    const timezone = timezoneFromBody(await request.json().catch(() => null));
    if (timezone !== null && (typeof timezone !== "string" || !isValidIanaTimeZone(timezone))) {
      return NextResponse.json(
        { error: "`timezone` must be a valid IANA time zone name or null" },
        { status: 400 }
      );
    }

    await db
      .update(users)
      .set({ timezone, updatedAt: new Date() })
      .where(eq(users.id, session.id));

    return NextResponse.json({ timezone });
  } catch (error) {
    console.error("Timezone update error:", error);
    return NextResponse.json({ error: "Failed to update time zone setting" }, { status: 500 });
  }
}
