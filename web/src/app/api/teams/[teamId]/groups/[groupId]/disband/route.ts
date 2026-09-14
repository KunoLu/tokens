import { NextResponse } from "next/server";
import { disbandGroup } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function POST(
  request: Request,
  context: { params: Promise<{ teamId: string; groupId: string }> }
) {
  try {
    const { teamId, groupId } = await context.params;
    const session = await requireCookieSession(request);
    await disbandGroup(teamId, groupId, session.id);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
