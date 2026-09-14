import { NextResponse } from "next/server";
import { getSession } from "@/lib/auth/session";
import { getTeamboard, getTeamboardTeams } from "@/lib/teamboard/getTeamboard";
import { teamErrorResponse } from "@/lib/teams/http";
import type { Period, SortBy } from "@/lib/leaderboard/types";
import { isValidDateString, parseCustomDateRange } from "@/lib/leaderboard/dateRange";

const VALID_PERIODS: Period[] = ["all", "month", "last-month", "week", "today", "custom"];
const VALID_SORT_BY: SortBy[] = ["tokens", "cost"];

/**
 * GET /api/teamboard (PRD §9.3)
 * - `?teamId=<id>&groupIds=<id>…&period=&sortBy=&page=&search=` → one team's
 *   ranked members (404 for unknown or private-to-the-viewer teams, never
 *   403 — D-2). The HTML page's own URL keeps its `team`/`group` names; both
 *   are accepted here as aliases.
 * - no `teamId` → the caller's filter list (public active teams ∪ own team).
 *
 * `period`/`sortBy`/`page`/`search` mirror `/api/leaderboard`, including
 * `from`/`to` for the custom range and the viewer-local "today" correction.
 * Responses are viewer-dependent, so the route stays dynamic; the loader owns
 * caching. A missing DATABASE_URL surfaces as a 500 `{ error }` like every
 * other loader-backed route — never a fake empty 200.
 */
export async function GET(request: Request) {
  try {
    const { searchParams } = new URL(request.url);
    const session = await getSession();
    const viewerId = session?.id ?? null;

    const teamId = searchParams.get("teamId") ?? searchParams.get("team");
    if (!teamId) {
      const teams = await getTeamboardTeams(viewerId);
      return NextResponse.json({ teams });
    }

    const groupIds = [
      ...searchParams.getAll("groupIds"),
      ...searchParams.getAll("group"),
    ];

    const periodParam = searchParams.get("period") || "all";
    let period: Period = VALID_PERIODS.includes(periodParam as Period)
      ? (periodParam as Period)
      : "all";

    const sortByParam = searchParams.get("sortBy") || "tokens";
    const sortBy: SortBy = VALID_SORT_BY.includes(sortByParam as SortBy)
      ? (sortByParam as SortBy)
      : "tokens";

    const pageParam = Number(searchParams.get("page"));
    const page = Number.isFinite(pageParam) && pageParam > 0 ? Math.floor(pageParam) : 1;

    const search = (searchParams.get("search") || "").trim();

    const fromParam = searchParams.get("from");
    const toParam = searchParams.get("to");
    const customDateRange =
      period === "custom" ? parseCustomDateRange(fromParam, toParam) : null;
    if (period === "custom" && !customDateRange) {
      period = "all";
    }
    const customFrom = customDateRange?.from ??
      (period === "today" && isValidDateString(fromParam) ? fromParam : undefined);
    const customTo = customDateRange?.to;

    const data = await getTeamboard(teamId, groupIds, viewerId, {
      period,
      sortBy,
      page,
      search,
      customFrom,
      customTo,
    });
    return NextResponse.json(data);
  } catch (error) {
    return teamErrorResponse(error);
  }
}
