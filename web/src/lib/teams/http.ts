import { NextResponse } from "next/server";
import { getSessionFromRequest } from "@/lib/auth/requestSession";
import { TeamError } from "./errors";

export function teamErrorResponse(error: unknown): NextResponse {
  if (error instanceof TeamError) {
    const body = error.details
      ? { error: error.message, details: error.details }
      : { error: error.message };
    return NextResponse.json(body, { status: error.status });
  }
  console.error("Team API error:", error);
  return NextResponse.json({ error: "Request failed" }, { status: 500 });
}

export async function requireCookieSession(request: Request) {
  const session = await getSessionFromRequest(request, {
    allowAuthorizationHeader: false,
  });
  if (!session) {
    throw new TeamError("Unauthorized", 401);
  }
  return session;
}
