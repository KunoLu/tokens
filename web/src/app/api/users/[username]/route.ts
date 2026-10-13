import { getPublicProfileResponse } from "@/lib/publicProfileData";

export const dynamic = "force-dynamic";

interface RouteParams {
  params: Promise<{ username: string }>;
}

export async function GET(request: Request, context: RouteParams) {
  return getPublicProfileResponse(request, context);
}
