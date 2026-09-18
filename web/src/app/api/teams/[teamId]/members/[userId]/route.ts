import { NextResponse } from "next/server";
import { patchMemberRole, removeMember } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function PATCH(
  request: Request,
  context: { params: Promise<{ teamId: string; userId: string }> }
) {
  try {
    const { teamId, userId } = await context.params;
    const session = await requireCookieSession(request);
    const body = await request.json().catch(() => ({}));
    await patchMemberRole(teamId, session.id, userId, body.role);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}

export async function DELETE(
  request: Request,
  context: { params: Promise<{ teamId: string; userId: string }> }
) {
  try {
    const { teamId, userId } = await context.params;
    const session = await requireCookieSession(request);
    await removeMember(teamId, session.id, userId);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
