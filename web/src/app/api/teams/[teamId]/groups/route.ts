import { NextResponse } from "next/server";
import { getSession } from "@/lib/auth/session";
import { createGroup, listGroups } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function GET(
  _request: Request,
  context: { params: Promise<{ teamId: string }> }
) {
  try {
    const { teamId } = await context.params;
    const session = await getSession();
    const groups = await listGroups(teamId, session?.id ?? null);
    return NextResponse.json({ groups });
  } catch (error) {
    return teamErrorResponse(error);
  }
}

export async function POST(
  request: Request,
  context: { params: Promise<{ teamId: string }> }
) {
  try {
    const { teamId } = await context.params;
    const session = await requireCookieSession(request);
    const body = await request.json().catch(() => ({}));
    const group = await createGroup(teamId, session.id, body);
    return NextResponse.json(group, { status: 201 });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
