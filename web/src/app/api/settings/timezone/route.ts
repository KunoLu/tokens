import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { getSessionFromRequest } from "@/lib/auth/requestSession";
import { db, users } from "@/lib/db";
import { PRIVATE_CACHE_CONTROL } from "@/lib/privacy";
import { normalizeIanaTimeZone } from "@/lib/timezone";

function timezoneFromBody(body: unknown): unknown {
  if (!body || typeof body !== "object" || !("timezone" in body)) return undefined;
  return body.timezone;
}

/**
 * The timezone column arrives with migration 0028; a database that has not
 * run it yet makes every read/write fail with Postgres' 42703. That state is
 * reported as an explicit 503 (run db:migrate) instead of a bare 500.
 */
function isMissingTimezoneColumnError(error: unknown): boolean {
  if (!error || typeof error !== "object") return false;
  const { code, message } = error as { code?: unknown; message?: unknown };
  if (code === "42703") return true;
  return (
    typeof message === "string" &&
    /column .*timezone.* does not exist/i.test(message)
  );
}

const MIGRATION_REQUIRED = {
  error:
    "Time zone setting is unavailable until migration 0028 is applied; run `bun run db:migrate` first",
} as const;

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
    if (isMissingTimezoneColumnError(error)) {
      return NextResponse.json(MIGRATION_REQUIRED, { status: 503 });
    }
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
    // What gets stored is the runtime's resolvedOptions spelling, not the
    // submitted one ("asia/shanghai" lands as "Asia/Shanghai").
    const normalized =
      typeof timezone === "string" ? normalizeIanaTimeZone(timezone) : null;
    if (timezone !== null && normalized === null) {
      return NextResponse.json(
        { error: "`timezone` must be a valid IANA time zone name or null" },
        { status: 400 }
      );
    }

    await db
      .update(users)
      .set({ timezone: normalized, updatedAt: new Date() })
      .where(eq(users.id, session.id));

    return NextResponse.json({ timezone: normalized });
  } catch (error) {
    if (isMissingTimezoneColumnError(error)) {
      return NextResponse.json(MIGRATION_REQUIRED, { status: 503 });
    }
    console.error("Timezone update error:", error);
    return NextResponse.json({ error: "Failed to update time zone setting" }, { status: 500 });
  }
}
