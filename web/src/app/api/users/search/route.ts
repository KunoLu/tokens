import { NextResponse } from "next/server";
import { searchUsers } from "@/lib/teams/service";
import { requireCookieSession, teamErrorResponse } from "@/lib/teams/http";

export async function GET(request: Request) {
  try {
    const session = await requireCookieSession(request);
    const q = new URL(request.url).searchParams.get("q") ?? "";
    const users = await searchUsers(session.id, q);
    return NextResponse.json({ users });
  } catch (error) {
    return teamErrorResponse(error);
  }
}
