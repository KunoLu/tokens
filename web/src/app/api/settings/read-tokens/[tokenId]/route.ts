import { NextResponse } from "next/server";
import { getSessionFromRequest } from "@/lib/auth/requestSession";
import { revokeReadToken } from "@/lib/auth/readTokens";

const UUID_REGEX =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

interface RouteParams {
  params: Promise<{ tokenId: string }>;
}

export async function DELETE(request: Request, { params }: RouteParams) {
  try {
    const session = await getSessionFromRequest(request, {
      allowAuthorizationHeader: false,
    });
    if (!session) {
      return NextResponse.json({ error: "Not authenticated" }, { status: 401 });
    }

    const { tokenId } = await params;
    if (!UUID_REGEX.test(tokenId) || !(await revokeReadToken(session.id, tokenId))) {
      return NextResponse.json({ error: "Token not found" }, { status: 404 });
    }

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error("Read token delete error:", error);
    return NextResponse.json({ error: "Failed to delete read token" }, { status: 500 });
  }
}
