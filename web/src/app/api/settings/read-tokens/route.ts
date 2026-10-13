import { NextResponse } from "next/server";
import { getSessionFromRequest } from "@/lib/auth/requestSession";
import { issueReadToken, listReadTokens, normalizeReadTokenName } from "@/lib/auth/readTokens";

function nameFromBody(body: unknown): unknown {
  if (!body || typeof body !== "object" || !("name" in body)) return undefined;
  return body.name;
}

export async function GET(request: Request) {
  try {
    const session = await getSessionFromRequest(request, {
      allowAuthorizationHeader: false,
    });
    if (!session) {
      return NextResponse.json({ error: "Not authenticated" }, { status: 401 });
    }

    return NextResponse.json({ tokens: await listReadTokens(session.id) });
  } catch (error) {
    console.error("Read tokens list error:", error);
    return NextResponse.json({ error: "Failed to fetch read tokens" }, { status: 500 });
  }
}

export async function POST(request: Request) {
  try {
    const session = await getSessionFromRequest(request, {
      allowAuthorizationHeader: false,
    });
    if (!session) {
      return NextResponse.json({ error: "Not authenticated" }, { status: 401 });
    }

    const token = await issueReadToken({
      userId: session.id,
      name: normalizeReadTokenName(nameFromBody(await request.json().catch(() => null))),
    });

    return NextResponse.json({ token }, { status: 201 });
  } catch (error) {
    console.error("Read token create error:", error);
    return NextResponse.json({ error: "Failed to create read token" }, { status: 500 });
  }
}
