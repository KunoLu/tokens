import { unstable_cache } from "next/cache";
import { and, eq, gte, isNull, lte, or, sql } from "drizzle-orm";
import { dailyBreakdown, db, groupMembers, groups, submissions, teamMembers, teams, users } from "@/lib/db";
import type { LeaderboardGroupRef, LeaderboardTeamRef, LeaderboardUser, Period, SortBy } from "@/lib/leaderboard/types";
import {
  MAX_PAGE,
  MAX_SEARCH_LENGTH,
  aggregatePeriodRows,
  compareLeaderboardUsers,
  getPeriodDateRange,
  pageRanking,
  periodCacheKey,
  rowMatchesSearchDirectives,
  type LeaderboardPeriodRow,
} from "@/lib/leaderboard/getLeaderboard";
import {
  escapeLikePattern,
  hasDirectives,
  parseSearchDirectives,
  type ParsedSearchDirectives,
} from "@/lib/leaderboard/searchDirectives";
import { TeamError } from "@/lib/teams/errors";
import { canViewTeam } from "@/lib/teams/visibility";

export interface TeamboardTeamOption {
  id: string;
  name: string;
  slug: string;
  /** Raw teams.visibility value; the client only compares against "private". */
  visibility: string;
  isMine: boolean;
}

export interface TeamboardGroupOption {
  id: string;
  name: string;
  memberCount: number;
}

/** FR-2: the Teamboard keeps the Leaderboard's Period / Sort by / search / page. */
export interface TeamboardQuery {
  period?: Period;
  sortBy?: SortBy;
  page?: number;
  search?: string;
  customFrom?: string;
  customTo?: string;
}

export interface TeamboardData {
  team: LeaderboardTeamRef;
  groups: TeamboardGroupOption[];
  /**
   * The current page slice. Ranked after the group filter is applied, so
   * ranks restart at 1; assigned before the text filter, so a search shows
   * each member's real position (same rule as the Leaderboard).
   */
  members: LeaderboardUser[];
  /** The URL group ids that actually exist on this team. */
  selectedGroupIds: string[];
  pagination: {
    page: number;
    limit: number;
    totalUsers: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  period: Period;
  sortBy: SortBy;
}

interface TeamboardBundle {
  team: {
    id: string;
    name: string;
    slug: string;
    visibility: string;
    status: string;
  };
  groups: TeamboardGroupOption[];
  members: Array<Omit<LeaderboardUser, "rank" | "team">>;
}

interface TeamboardPeriodRow {
  userId: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
  tokens: number;
  cost: number;
  sourceBreakdown: Record<string, { models: Record<string, unknown> }> | null;
  groupId: string | null;
  groupName: string | null;
}

// team/board ids arrive from a public URL; comparing a uuid column against
// garbage raises Postgres 22P02 instead of returning no rows, so validate
// before querying and treat malformed ids as "no such team".
const UUID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

// Same page size as the Leaderboard's default, so paging one board feels like
// paging the other.
const TEAMBOARD_PAGE_SIZE = 50;

/**
 * The whole board for one team, aggregated once and shared by every viewer.
 * Viewer identity only gates access (checked per request in `getTeamboard`),
 * never the contents, so one cache entry per team is correct for everyone.
 * Tagged `leaderboard` because every mutation that can change these numbers —
 * submissions, membership, group and team edits — already revalidates it.
 *
 * All-time totals only: bounded periods read `dailyBreakdown` instead, same
 * as the Leaderboard.
 */
async function fetchTeamboardBundle(teamId: string): Promise<TeamboardBundle | null> {
  const teamRows = await db
    .select({
      id: teams.id,
      name: teams.name,
      slug: teams.slug,
      visibility: teams.visibility,
      status: teams.status,
    })
    .from(teams)
    .where(eq(teams.id, teamId))
    .limit(1);
  const team = teamRows[0];
  if (!team) return null;

  const [groupRows, memberRows] = await Promise.all([
    db
      .select({
        id: groups.id,
        name: groups.name,
        memberCount: sql<number>`COUNT(${groupMembers.id})`.as("member_count"),
      })
      .from(groups)
      .leftJoin(groupMembers, eq(groupMembers.groupId, groups.id))
      .where(and(eq(groups.teamId, teamId), eq(groups.status, "active")))
      .groupBy(groups.id, groups.name)
      .orderBy(groups.name),
    // Members with no submissions stay on the board with zeroed totals — a
    // team page that hid everyone who had a quiet month would read as broken.
    // group_members.user_id is unique, so at most one (active) group joins.
    db
      .select({
        userId: users.id,
        username: users.username,
        displayName: users.displayName,
        avatarUrl: users.avatarUrl,
        totalTokens: sql<number>`COALESCE(SUM(${submissions.totalTokens}), 0)`.as("total_tokens"),
        totalCost: sql<number>`COALESCE(SUM(CAST(${submissions.totalCost} AS DECIMAL(18,4))), 0)`.as("total_cost"),
        groupId: sql<string | null>`${groups.id}`.as("group_id"),
        groupName: sql<string | null>`${groups.name}`.as("group_name"),
      })
      .from(teamMembers)
      .innerJoin(users, eq(users.id, teamMembers.userId))
      .leftJoin(submissions, eq(submissions.userId, users.id))
      .leftJoin(groupMembers, eq(groupMembers.userId, users.id))
      .leftJoin(
        groups,
        and(eq(groups.id, groupMembers.groupId), eq(groups.status, "active"))
      )
      .where(and(eq(teamMembers.teamId, teamId), isNull(users.bannedAt)))
      .groupBy(users.id, users.username, users.displayName, users.avatarUrl, groups.id, groups.name)
      .orderBy(
        sql`COALESCE(SUM(${submissions.totalTokens}), 0) DESC`,
        sql`COALESCE(SUM(CAST(${submissions.totalCost} AS DECIMAL(18,4))), 0) DESC`,
        sql`LOWER(${users.username}) ASC`
      ),
  ]);

  return {
    team,
    groups: groupRows.map((row) => ({
      id: row.id,
      name: row.name,
      memberCount: Number(row.memberCount) || 0,
    })),
    members: memberRows.map((row) => ({
      userId: row.userId,
      username: row.username,
      displayName: row.displayName,
      avatarUrl: row.avatarUrl,
      group:
        row.groupId && row.groupName
          ? ({ id: row.groupId, name: row.groupName } satisfies LeaderboardGroupRef)
          : null,
      totalTokens: Number(row.totalTokens) || 0,
      totalCost: Number(row.totalCost) || 0,
    })),
  };
}

function getCachedTeamboardBundle(teamId: string): Promise<TeamboardBundle | null> {
  return unstable_cache(
    () => fetchTeamboardBundle(teamId),
    ["teamboard-bundle", teamId],
    { tags: ["leaderboard"], revalidate: 60 }
  )();
}

async function isTeamMember(teamId: string, userId: string): Promise<boolean> {
  const rows = await db
    .select({ id: teamMembers.id })
    .from(teamMembers)
    .where(and(eq(teamMembers.teamId, teamId), eq(teamMembers.userId, userId)))
    .limit(1);
  return rows.length > 0;
}

/**
 * Every daily row of the team's members inside the period, folded per user by
 * the caller — the team-scoped twin of the Leaderboard's period rows. Members
 * with no submissions in the period drop out, matching the Leaderboard's
 * bounded periods. `sourceBreakdown` only comes along when a `client:`/`model:`
 * directive will read it (it is the heaviest column by far).
 */
async function fetchTeamboardPeriodRows(
  teamId: string,
  period: Exclude<Period, "all">,
  customFrom: string | undefined,
  customTo: string | undefined,
  withBreakdown: boolean
): Promise<TeamboardPeriodRow[]> {
  const dateRange = getPeriodDateRange(period, new Date(), customFrom, customTo);
  if (!dateRange) return [];

  const rows = await db
    .select({
      userId: users.id,
      username: users.username,
      displayName: users.displayName,
      avatarUrl: users.avatarUrl,
      tokens: dailyBreakdown.tokens,
      cost: dailyBreakdown.cost,
      ...(withBreakdown ? { sourceBreakdown: dailyBreakdown.sourceBreakdown } : {}),
      groupId: sql<string | null>`${groups.id}`.as("group_id"),
      groupName: sql<string | null>`${groups.name}`.as("group_name"),
    })
    .from(teamMembers)
    .innerJoin(users, eq(users.id, teamMembers.userId))
    .innerJoin(submissions, eq(submissions.userId, users.id))
    .innerJoin(dailyBreakdown, eq(dailyBreakdown.submissionId, submissions.id))
    .leftJoin(groupMembers, eq(groupMembers.userId, users.id))
    .leftJoin(
      groups,
      and(eq(groups.id, groupMembers.groupId), eq(groups.status, "active"))
    )
    .where(
      and(
        eq(teamMembers.teamId, teamId),
        gte(dailyBreakdown.date, dateRange.start),
        lte(dailyBreakdown.date, dateRange.end),
        isNull(users.bannedAt)
      )
    );

  return rows.map((row) => ({
    userId: row.userId,
    username: row.username,
    displayName: row.displayName,
    avatarUrl: row.avatarUrl,
    tokens: Number(row.tokens) || 0,
    cost: Number(row.cost) || 0,
    sourceBreakdown:
      ("sourceBreakdown" in row ? row.sourceBreakdown : null) ?? null,
    groupId: row.groupId ?? null,
    groupName: row.groupName ?? null,
  }));
}

/**
 * All-time totals for the members whose submissions match a `client:`/`model:`
 * directive. Mirrors the Leaderboard's all-time search path: the EXISTS
 * conditions filter submission rows, so the sums cover matching submissions
 * only. Only called when a directive is present — the plain all-time board
 * reads the per-team bundle instead.
 */
async function fetchAllTimeDirectiveMembers(
  teamId: string,
  parsed: ParsedSearchDirectives
): Promise<Array<Omit<LeaderboardUser, "rank" | "team">>> {
  const clientConditions = parsed.clients.map(
    (client) =>
      sql`EXISTS (SELECT 1 FROM unnest(${submissions.sourcesUsed}) AS s WHERE LOWER(s) LIKE ${`%${escapeLikePattern(client)}%`})`
  );
  const modelConditions = parsed.models.map(
    (model) =>
      sql`EXISTS (SELECT 1 FROM unnest(${submissions.modelsUsed}) AS m WHERE LOWER(m) LIKE ${`%${escapeLikePattern(model)}%`})`
  );
  const directiveConditions = [
    clientConditions.length > 0 ? or(...clientConditions) : undefined,
    modelConditions.length > 0 ? or(...modelConditions) : undefined,
  ].filter((condition): condition is NonNullable<typeof condition> => condition !== undefined);

  const rows = await db
    .select({
      userId: users.id,
      username: users.username,
      displayName: users.displayName,
      avatarUrl: users.avatarUrl,
      totalTokens: sql<number>`COALESCE(SUM(${submissions.totalTokens}), 0)`.as("total_tokens"),
      totalCost: sql<number>`COALESCE(SUM(CAST(${submissions.totalCost} AS DECIMAL(18,4))), 0)`.as("total_cost"),
      groupId: sql<string | null>`${groups.id}`.as("group_id"),
      groupName: sql<string | null>`${groups.name}`.as("group_name"),
    })
    .from(teamMembers)
    .innerJoin(users, eq(users.id, teamMembers.userId))
    .innerJoin(submissions, eq(submissions.userId, users.id))
    .leftJoin(groupMembers, eq(groupMembers.userId, users.id))
    .leftJoin(
      groups,
      and(eq(groups.id, groupMembers.groupId), eq(groups.status, "active"))
    )
    .where(
      and(
        eq(teamMembers.teamId, teamId),
        isNull(users.bannedAt),
        ...directiveConditions
      )
    )
    .groupBy(users.id, users.username, users.displayName, users.avatarUrl, groups.id, groups.name);

  return rows.map((row) => ({
    userId: row.userId,
    username: row.username,
    displayName: row.displayName,
    avatarUrl: row.avatarUrl,
    group:
      row.groupId && row.groupName
        ? ({ id: row.groupId, name: row.groupName } satisfies LeaderboardGroupRef)
        : null,
    totalTokens: Number(row.totalTokens) || 0,
    totalCost: Number(row.totalCost) || 0,
  }));
}

/**
 * The board for one (team, groups, period, sortBy, page, search) combination.
 * Everything past the membership rows — group filter, ordering, ranking,
 * search, slicing — reuses the Leaderboard's pure helpers so both boards
 * answer the same query the same way.
 */
async function fetchTeamboard(
  bundle: TeamboardBundle,
  selectedGroupIds: string[],
  period: Period,
  sortBy: SortBy,
  page: number,
  search: string,
  customFrom?: string,
  customTo?: string
): Promise<TeamboardData> {
  const teamRef: LeaderboardTeamRef = {
    id: bundle.team.id,
    name: bundle.team.name,
    slug: bundle.team.slug,
  };
  const parsed = parseSearchDirectives(search);

  let members: Array<Omit<LeaderboardUser, "rank">>;
  if (period === "all") {
    const base = hasDirectives(parsed)
      ? await fetchAllTimeDirectiveMembers(bundle.team.id, parsed)
      : bundle.members;
    members = base.map((member) => ({ ...member, team: teamRef }));
  } else {
    const rows = await fetchTeamboardPeriodRows(
      bundle.team.id,
      period,
      customFrom,
      customTo,
      hasDirectives(parsed)
    );
    const matching = hasDirectives(parsed)
      ? rows.filter((row) => rowMatchesSearchDirectives(row.sourceBreakdown, parsed))
      : rows;
    members = aggregatePeriodRows(
      matching.map(
        (row): LeaderboardPeriodRow => ({
          userId: row.userId,
          username: row.username,
          displayName: row.displayName,
          avatarUrl: row.avatarUrl,
          tokens: row.tokens,
          cost: row.cost,
          sourceBreakdown: row.sourceBreakdown,
          teamId: teamRef.id,
          teamName: teamRef.name,
          teamSlug: teamRef.slug,
          groupId: row.groupId,
          groupName: row.groupName,
        })
      ),
      sortBy
    );
  }

  // Group filter first — ranks restart at 1 within the selected groups — then
  // the Leaderboard's ordering, rank, text filter and page slice.
  const selected = new Set(selectedGroupIds);
  const groupFiltered =
    selected.size > 0
      ? members.filter((member) => member.group !== null && selected.has(member.group.id))
      : members;
  const sorted = [...groupFiltered].sort((left, right) =>
    compareLeaderboardUsers(left, right, sortBy)
  );
  const data = pageRanking(sorted, page, TEAMBOARD_PAGE_SIZE, period, sortBy, parsed.text);

  return {
    team: teamRef,
    groups: bundle.groups,
    members: data.users,
    selectedGroupIds,
    pagination: data.pagination,
    period,
    sortBy,
  };
}

/**
 * Board data for one team. Throws TeamError(404) for unknown teams and for
 * teams the viewer may not see (D-2: 404, not 403, so the URL probes nothing).
 * Missing DATABASE_URL propagates — the page degrades and the API 500s; the
 * loader does not invent empty data.
 *
 * The viewer check runs per request; the board itself is cached per
 * (teamId, groupIds, period, sortBy, page, search) because its contents are
 * viewer-independent (PRD §11). `page` and `search` are clamped for the same
 * reason the Leaderboard clamps them: they arrive from a public URL and join
 * the cache key, so an uncapped value buys unbounded cache entries.
 */
export async function getTeamboard(
  teamId: string,
  groupIds: string[],
  viewerId: string | null,
  query: TeamboardQuery = {}
): Promise<TeamboardData> {
  if (!UUID_PATTERN.test(teamId)) {
    throw new TeamError("Team not found", 404);
  }
  const bundle = await getCachedTeamboardBundle(teamId);
  if (!bundle) {
    throw new TeamError("Team not found", 404);
  }
  const viewerIsMember = viewerId ? await isTeamMember(teamId, viewerId) : false;
  if (!canViewTeam(bundle.team, viewerIsMember)) {
    throw new TeamError("Team not found", 404);
  }

  const period = query.period ?? "all";
  const sortBy = query.sortBy ?? "tokens";
  const requestedPage = query.page ?? 1;
  const page = Number.isFinite(requestedPage)
    ? Math.min(MAX_PAGE, Math.max(1, Math.floor(requestedPage)))
    : 1;
  const search = (query.search ?? "").slice(0, MAX_SEARCH_LENGTH);

  const validGroupIds = new Set(bundle.groups.map((group) => group.id));
  const selectedGroupIds = [...new Set(groupIds)].filter((id) => validGroupIds.has(id));
  const groupKey = [...selectedGroupIds].sort().join(",");

  const customFrom = query.customFrom;
  const customTo = query.customTo;

  return unstable_cache(
    async () => {
      // Re-read the (already cached) bundle instead of closing over this
      // request's object — `unstable_cache` callbacks must be self-contained.
      const cached = await getCachedTeamboardBundle(teamId);
      if (!cached) {
        throw new TeamError("Team not found", 404);
      }
      return fetchTeamboard(
        cached,
        selectedGroupIds,
        period,
        sortBy,
        page,
        search,
        customFrom,
        customTo
      );
    },
    [
      "teamboard",
      teamId,
      groupKey,
      period === "all"
        ? "all"
        : periodCacheKey(period, customFrom, customTo),
      sortBy,
      String(page),
      search,
    ],
    { tags: ["leaderboard"], revalidate: 60 }
  )();
}

/**
 * Filter options for the board: public active teams ∪ the viewer's own team
 * (any visibility). The public half is shared and cached; the membership half
 * is one indexed lookup per request. The viewer's team sorts first so the
 * default selection is always the top option.
 */
export async function getTeamboardTeams(
  viewerId: string | null
): Promise<TeamboardTeamOption[]> {
  const publicTeams = await getCachedPublicActiveTeams();
  if (!viewerId) {
    return publicTeams.map((team) => ({ ...team, isMine: false }));
  }

  // INV-1: a user belongs to at most one team.
  const mineRows = await db
    .select({
      id: teams.id,
      name: teams.name,
      slug: teams.slug,
      visibility: teams.visibility,
    })
    .from(teamMembers)
    .innerJoin(teams, eq(teams.id, teamMembers.teamId))
    .where(eq(teamMembers.userId, viewerId))
    .limit(1);
  const mine = mineRows[0];

  const byId = new Map(
    publicTeams.map((team) => [team.id, { ...team, isMine: false } as TeamboardTeamOption])
  );
  if (mine) {
    byId.set(mine.id, { ...mine, isMine: true });
  }
  return [...byId.values()].sort(
    (a, b) => Number(b.isMine) - Number(a.isMine) || a.name.localeCompare(b.name)
  );
}

interface PublicActiveTeamRow {
  id: string;
  name: string;
  slug: string;
  visibility: string;
}

function getCachedPublicActiveTeams(): Promise<PublicActiveTeamRow[]> {
  return unstable_cache(
    async () =>
      db
        .select({
          id: teams.id,
          name: teams.name,
          slug: teams.slug,
          visibility: teams.visibility,
        })
        .from(teams)
        .where(and(eq(teams.visibility, "public"), eq(teams.status, "active")))
        .orderBy(teams.name),
    ["teamboard-public-active-teams"],
    { tags: ["leaderboard"], revalidate: 60 }
  )();
}
