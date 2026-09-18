import { NextResponse } from "next/server";
import { getSession } from "@/lib/auth/session";
import { createTeam, listTeams } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function GET() {
  try {
    const session = await getSession();
    const teams = await listTeams(session?.id ?? null);
    return NextResponse.json({ teams });
  } catch (error) {
    return teamErrorResponse(error);
  }
}

export async function POST(request: Request) {
  try {
    const session = await requireCookieSession(request);
    const body = await request.json().catch(() => ({}));
    const team = await createTeam(session.id, body);
    return NextResponse.json(team, { status: 201 });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
