import { NextResponse } from "next/server";
import { deleteGroup, patchGroup } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function PATCH(
  request: Request,
  context: { params: Promise<{ teamId: string; groupId: string }> }
) {
  try {
    const { teamId, groupId } = await context.params;
    const session = await requireCookieSession(request);
    const body = await request.json().catch(() => ({}));
    const group = await patchGroup(teamId, groupId, session.id, body);
    return NextResponse.json(group);
  } catch (error) {
    return teamErrorResponse(error);
  }
}

export async function DELETE(
  request: Request,
  context: { params: Promise<{ teamId: string; groupId: string }> }
) {
  try {
    const { teamId, groupId } = await context.params;
    const session = await requireCookieSession(request);
    await deleteGroup(teamId, groupId, session.id);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
