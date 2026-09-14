import { NextResponse } from "next/server";
import { eq } from "drizzle-orm";
import { db, users } from "@/lib/db";
import { getSession } from "@/lib/auth/session";
import { listMyInvitations } from "@/lib/teams/service";
import { teamErrorResponse } from "@/lib/teams/http";

export async function GET() {
  try {
    const session = await getSession();
    if (!session) {
      return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
    }
    const rows = await db
      .select({ email: users.email })
      .from(users)
      .where(eq(users.id, session.id))
      .limit(1);
    const invitations = await listMyInvitations(session.id, rows[0]?.email ?? null);
    return NextResponse.json({ invitations });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
