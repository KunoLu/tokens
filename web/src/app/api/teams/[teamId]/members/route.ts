import { NextResponse } from "next/server";
import { getSession } from "@/lib/auth/session";
import { inviteMembers, listMembers } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function GET(
  _request: Request,
  context: { params: Promise<{ teamId: string }> }
) {
  try {
    const { teamId } = await context.params;
    const session = await getSession();
    const members = await listMembers(teamId, session?.id ?? null);
    return NextResponse.json({ members });
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
    const items = Array.isArray(body.items) ? body.items : Array.isArray(body) ? body : [body];
    const result = await inviteMembers(teamId, session.id, items);
    return NextResponse.json(result);
  } catch (error) {
    return teamErrorResponse(error);
  }
}
