import type { Metadata } from "next";
import { cookies } from "next/headers";
import { notFound } from "next/navigation";
import { getSession } from "@/lib/auth/session";
import { TeamError } from "@/lib/teams/errors";
import {
  getTeamboard,
  getTeamboardTeams,
  type TeamboardData,
  type TeamboardTeamOption,
} from "@/lib/teamboard/getTeamboard";
import {
  SORT_BY_COOKIE_NAME,
  resolveSortByParam,
} from "@/lib/leaderboard/constants";
import { isValidDateString, parseCustomDateRange } from "@/lib/leaderboard/dateRange";
import type { Period, SortBy } from "@/lib/leaderboard/types";
import { SITE_URL } from "@/lib/site";
import { TeamboardClient } from "@/components/teamboard/Teamboard";

// Reads searchParams on every request; `revalidate` + searchParams flaps on
// Workers, so this page is dynamic like any other filter-driven route.
export const dynamic = "force-dynamic";

export const metadata: Metadata = {
  title: "Teamboard - Tokens",
  description: "Token usage ranked within a team, filtered by group.",
  openGraph: {
    title: "Teamboard — Tokens",
    description: "Token usage ranked within a team, filtered by group.",
    url: `${SITE_URL}/teamboard`,
    siteName: "Tokens",
    images: [
      {
        url: "/api/og?title=Teamboard&subtitle=Token+usage+ranked+within+a+team.",
        width: 1200,
        height: 630,
      },
    ],
  },
  twitter: { card: "summary_large_image" },
};

function isMissingDatabaseUrl(error: unknown): boolean {
  return (
    error instanceof Error &&
    error.message === "DATABASE_URL environment variable is not set"
  );
}

const VALID_PERIODS: Period[] = ["all", "month", "last-month", "week", "today", "custom"];

interface PageProps {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>;
}

export default async function TeamboardPage({ searchParams }: PageProps) {
  const [cookieStore, params] = await Promise.all([cookies(), searchParams]);
  const teamParam =
    typeof params.team === "string" && params.team ? params.team : null;
  const groupParam = params.group;
  const groupIds = Array.isArray(groupParam)
    ? groupParam
    : typeof groupParam === "string" && groupParam
      ? [groupParam]
      : [];

  // FR-2: the Teamboard keeps the Leaderboard's Period / Sort by / search /
  // pagination. Resolution mirrors the leaderboard page — param, then the
  // shared sort cookie, then the default — except the landing period is "all"
  // (the team's overall standing, per docs/demo/teamboard-demo.html).
  const sortByParam = typeof params.sortBy === "string" ? params.sortBy : null;
  const sortByCookie = cookieStore.get(SORT_BY_COOKIE_NAME)?.value;
  const sortBy: SortBy =
    resolveSortByParam(sortByParam) ?? resolveSortByParam(sortByCookie) ?? "tokens";

  const periodParam = typeof params.period === "string" ? params.period : null;
  let period: Period =
    periodParam && VALID_PERIODS.includes(periodParam as Period)
      ? (periodParam as Period)
      : "all";

  const fromParam = typeof params.from === "string" ? params.from : null;
  const toParam = typeof params.to === "string" ? params.to : null;
  const customDateRange =
    period === "custom" ? parseCustomDateRange(fromParam, toParam) : null;
  if (period === "custom" && !customDateRange) {
    period = "all";
  }
  // Same local-today correction the leaderboard applies: daily rows are
  // bucketed by the submitter's local date, and the client re-sends its own.
  const localToday =
    period === "today" && isValidDateString(fromParam) ? fromParam : undefined;
  const customFrom = customDateRange?.from ?? localToday;
  const customTo = customDateRange?.to;

  const pageParam =
    typeof params.page === "string" ? Math.max(1, Number(params.page) || 1) : 1;
  const searchParam =
    typeof params.search === "string" ? params.search.trim() : "";

  // No database (local dev without DATABASE_URL): degrade to the empty board
  // instead of throwing, same as the leaderboard page.
  const session = await getSession().catch((error: unknown) => {
    if (isMissingDatabaseUrl(error)) return null;
    throw error;
  });
  const viewerId = session?.id ?? null;

  const teams: TeamboardTeamOption[] = await getTeamboardTeams(viewerId).catch(
    (error: unknown) => {
      if (isMissingDatabaseUrl(error)) return [];
      throw error;
    }
  );

  // Signed-in viewers land on their own team; everyone else picks one.
  const selectedTeamId = teamParam ?? teams.find((team) => team.isMine)?.id ?? null;

  let board: TeamboardData | null = null;
  if (selectedTeamId) {
    try {
      board = await getTeamboard(selectedTeamId, groupIds, viewerId, {
        period,
        sortBy,
        page: pageParam,
        search: searchParam,
        customFrom,
        customTo,
      });
    } catch (error) {
      // Unknown or private-to-the-viewer team: 404, not 403 (D-2).
      if (error instanceof TeamError && error.status === 404) notFound();
      if (isMissingDatabaseUrl(error)) {
        board = null;
      } else {
        throw error;
      }
    }
  }

  return (
    <main id="main-content">
      <TeamboardClient
        teams={teams}
        board={board}
        currentUserId={viewerId}
        period={period}
        sortBy={sortBy}
      />
    </main>
  );
}
