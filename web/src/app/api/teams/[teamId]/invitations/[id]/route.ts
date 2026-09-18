import { NextResponse } from "next/server";
import { revokeInvitation } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function DELETE(
  request: Request,
  context: { params: Promise<{ teamId: string; id: string }> }
) {
  try {
    const { teamId, id } = await context.params;
    const session = await requireCookieSession(request);
    await revokeInvitation(teamId, id, session.id);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
