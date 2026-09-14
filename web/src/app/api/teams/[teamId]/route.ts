import { NextResponse } from "next/server";
import { getSession } from "@/lib/auth/session";
import { deleteTeam, getTeam, patchTeam } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function GET(
  _request: Request,
  context: { params: Promise<{ teamId: string }> }
) {
  try {
    const { teamId } = await context.params;
    const session = await getSession();
    const team = await getTeam(teamId, session?.id ?? null);
    return NextResponse.json(team);
  } catch (error) {
    return teamErrorResponse(error);
  }
}

export async function PATCH(
  request: Request,
  context: { params: Promise<{ teamId: string }> }
) {
  try {
    const { teamId } = await context.params;
    const session = await requireCookieSession(request);
    const body = await request.json().catch(() => ({}));
    const team = await patchTeam(teamId, session.id, body);
    return NextResponse.json(team);
  } catch (error) {
    return teamErrorResponse(error);
  }
}

export async function DELETE(
  request: Request,
  context: { params: Promise<{ teamId: string }> }
) {
  try {
    const { teamId } = await context.params;
    const session = await requireCookieSession(request);
    await deleteTeam(teamId, session.id);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
