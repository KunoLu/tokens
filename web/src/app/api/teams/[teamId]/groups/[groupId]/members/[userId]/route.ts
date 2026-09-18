import { NextResponse } from "next/server";
import { addGroupMember, removeGroupMember } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function PUT(
  request: Request,
  context: { params: Promise<{ teamId: string; groupId: string; userId: string }> }
) {
  try {
    const { teamId, groupId, userId } = await context.params;
    const session = await requireCookieSession(request);
    await addGroupMember(teamId, groupId, session.id, userId);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}

export async function DELETE(
  request: Request,
  context: { params: Promise<{ teamId: string; groupId: string; userId: string }> }
) {
  try {
    const { teamId, groupId, userId } = await context.params;
    const session = await requireCookieSession(request);
    await removeGroupMember(teamId, groupId, session.id, userId);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
