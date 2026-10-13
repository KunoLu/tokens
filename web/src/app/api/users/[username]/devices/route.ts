import { getPublicProfileDevicesResponse } from "@/lib/publicProfileDevices";

export const dynamic = "force-dynamic";

interface RouteParams {
  params: Promise<{ username: string }>;
}

export async function GET(request: Request, context: RouteParams) {
  return getPublicProfileDevicesResponse(request, context);
}
