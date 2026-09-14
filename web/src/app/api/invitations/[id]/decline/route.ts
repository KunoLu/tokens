import { NextResponse } from "next/server";
import { declineInvitation } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function POST(
  request: Request,
  context: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await context.params;
    const session = await requireCookieSession(request);
    await declineInvitation(id, session.id);
    return NextResponse.json({ ok: true });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
