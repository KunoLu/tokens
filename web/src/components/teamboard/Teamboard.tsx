"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "nextjs-toploader/app";
import { usePathname, useSearchParams } from "next/navigation";
import { ChevronDownIcon, SearchIcon } from "lucide-react";
import { useI18n } from "@/lib/i18n";
import { useSettings } from "@/lib/useSettings";
import { cn } from "@/lib/utils";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "@/components/ui/empty";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  DeveloperRow,
  FormatToggle,
  PERIODS,
} from "@/components/leaderboard/Leaderboard";
import { MembershipColumnHeaders } from "@/components/leaderboard/MembershipCells";
import { toLocalDateString } from "@/lib/leaderboard/dateRange";
import type { Period, SortBy } from "@/lib/leaderboard/types";
import type { TeamboardData, TeamboardTeamOption } from "@/lib/teamboard/getTeamboard";

interface TeamboardProps {
  teams: TeamboardTeamOption[];
  /** Null when no team is selected (guide empty state) or the DB is absent. */
  board: TeamboardData | null;
  currentUserId: string | null;
  /** Server-resolved filters; the board repeats them when it exists. */
  period: Period;
  sortBy: SortBy;
}

const NO_GROUPS: string[] = [];

// Period order is the Leaderboard's (`PERIODS`); labels come from the
// dictionary like the rest of this page's copy. `custom` is parsed on the
// URL but is not a ToggleGroup option, so it reuses the All-time label.
const PERIOD_LABEL_KEYS: Record<Period, string> = {
  all: "teamboard.periodAll",
  today: "teamboard.periodToday",
  week: "teamboard.periodWeek",
  month: "teamboard.periodMonth",
  "last-month": "teamboard.periodLastMonth",
  custom: "teamboard.periodAll",
};

/**
 * Filter state lives in the URL (`?team=<id>&group=<id>…&period=&sortBy=
 * &search=&page=`); this shell only navigates, the RSC page re-loads. Team is
 * single-select and switching it clears the group selection; Group is
 * multi-select within the current team. Period, Sort by, search and
 * pagination behave exactly as on the Leaderboard (FR-2).
 */
export function TeamboardClient({ teams, board, currentUserId, period: periodProp, sortBy: sortByProp }: TeamboardProps) {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const { t } = useI18n();
  const { leaderboardTokenFormat: tokenFormat, setLeaderboardTokenFormat } =
    useSettings();

  const selectedTeamId = board?.team.id ?? null;
  const selectedGroupIds = board?.selectedGroupIds ?? NO_GROUPS;

  const urlSearch = searchParams.get("search") ?? "";
  const [search, setSearch] = useState(urlSearch);
  // Typing stays local until submit; Back/Forward changes the URL, so the
  // box follows that snapshot during render (an effect would trip
  // react-hooks/set-state-in-effect).
  const [prevUrlSearch, setPrevUrlSearch] = useState(urlSearch);
  if (urlSearch !== prevUrlSearch) {
    setPrevUrlSearch(urlSearch);
    setSearch(urlSearch);
  }
  // The query these results actually answer, as opposed to what is currently
  // typed in the box.
  const appliedSearch = urlSearch.trim();
  const [pendingPeriod, setPendingPeriod] = useState<Period | null>(null);

  // The server resolves the period, so it is read from props rather than
  // mirrored into state. `pendingPeriod` only holds the optimistic selection
  // between the click and the new data arriving.
  const period = pendingPeriod ?? board?.period ?? periodProp;
  // Sort toggles optimistically like the Leaderboard's: local state first,
  // the navigation lands the same value the server then confirms via props.
  // Same Back/Forward rule as search: follow the snapshot, not an effect.
  // The URL `sortBy` param is part of the snapshot so Back that restores a
  // tokens URL resets the control even when the Cost RSC never arrived and
  // `serverSortBy` therefore never changed.
  const serverSortBy = board?.sortBy ?? sortByProp;
  const urlSortBy = searchParams.get("sortBy");
  const urlSort = urlSortBy === "cost" || urlSortBy === "tokens" ? urlSortBy : null;
  const resolvedSortBy = urlSort ?? serverSortBy;
  const [sortBy, setSortBy] = useState<SortBy>(resolvedSortBy);
  const [prevResolvedSortBy, setPrevResolvedSortBy] = useState(resolvedSortBy);
  if (resolvedSortBy !== prevResolvedSortBy) {
    setPrevResolvedSortBy(resolvedSortBy);
    setSortBy(resolvedSortBy);
  }
  const pending = pendingPeriod != null && pendingPeriod !== (board?.period ?? periodProp);
  if (pendingPeriod != null && pendingPeriod === (board?.period ?? periodProp)) {
    setPendingPeriod(null);
  }
  const searchRef = useRef<HTMLInputElement>(null);

  const pushQuery = useCallback(
    (next: Record<string, string | null>) => {
      const params = new URLSearchParams(searchParams.toString());
      for (const [key, value] of Object.entries(next)) {
        if (value === null || value === "") params.delete(key);
        else params.set(key, value);
      }
      const qs = params.toString();
      router.push(qs ? `${pathname}?${qs}` : pathname);
    },
    [pathname, router, searchParams]
  );

  // The server can only resolve "today" in UTC, but daily rows are bucketed by
  // the submitter's local date — send our own date up, same correction the
  // Leaderboard applies. `replace` keeps it out of the back stack.
  useEffect(() => {
    if (period !== "today") return;
    const now = new Date();
    const localDate = toLocalDateString(now);
    const fromParam = searchParams.get("from");
    if (fromParam === localDate) return;
    if (fromParam === null && localDate === now.toISOString().slice(0, 10)) return;
    const params = new URLSearchParams(searchParams.toString());
    params.set("from", localDate);
    router.replace(`${pathname}?${params.toString()}`);
  }, [period, searchParams, pathname, router]);

  // "/" focuses search — the shortcut this audience reaches for by reflex.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== "/" || event.metaKey || event.ctrlKey) return;
      const tag = (event.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      event.preventDefault();
      searchRef.current?.focus();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Team is single-select; switching it clears the group filter (group ids
  // belong to the team that issued them) and restarts paging.
  const selectTeam = useCallback(
    (teamId: string) => {
      const params = new URLSearchParams(searchParams.toString());
      params.set("team", teamId);
      params.delete("group");
      params.delete("page");
      router.push(`${pathname}?${params.toString()}`);
    },
    [pathname, router, searchParams]
  );

  const toggleGroup = useCallback(
    (groupId: string, checked: boolean) => {
      const next = checked
        ? [...selectedGroupIds, groupId]
        : selectedGroupIds.filter((id) => id !== groupId);
      const params = new URLSearchParams(searchParams.toString());
      params.delete("group");
      for (const id of next) params.append("group", id);
      params.delete("page");
      const qs = params.toString();
      router.push(qs ? `${pathname}?${qs}` : pathname);
    },
    [pathname, router, searchParams, selectedGroupIds]
  );

  const selectedTeamOption = teams.find((team) => team.id === selectedTeamId);
  const members = useMemo(() => board?.members ?? [], [board]);
  const max = useMemo(() => {
    if (members.length === 0) return 0;
    return Math.max(
      ...members.map((member) =>
        sortBy === "cost" ? member.totalCost : member.totalTokens
      )
    );
  }, [members, sortBy]);

  const formatTitles = {
    showExact: t("teamboard.showExact"),
    abbreviate: t("teamboard.abbreviate"),
  };
  const toggleFormat = () =>
    setLeaderboardTokenFormat(tokenFormat === "compact" ? "full" : "compact");

  const pagination = board?.pagination ?? null;

  return (
    <div className={cn(CONTAINER, "pb-24 pt-10 sm:pt-14")}>
      <PageHeader title={t("nav.teamboard")} description={t("teamboard.desc")} />

      <div className="mt-8 flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center sm:gap-2">
        <Select
          value={selectedTeamId}
          onValueChange={(value) => {
            if (typeof value === "string" && value !== selectedTeamId) {
              selectTeam(value);
            }
          }}
        >
          <SelectTrigger
            aria-label={t("teamboard.teamFilter")}
            className="w-full sm:w-64"
          >
            <SelectValue>
              {() =>
                selectedTeamOption?.name ?? board?.team.name ?? t("teamboard.selectTeam")
              }
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            {teams.map((team) => (
              <SelectItem key={team.id} value={team.id}>
                <span className="flex min-w-0 flex-1 items-center gap-2">
                  <span className="truncate">{team.name}</span>
                  {team.visibility === "private" ? (
                    <span className="shrink-0 rounded-sm bg-muted px-1 text-[10px] text-muted-foreground">
                      {t("teams.private")}
                    </span>
                  ) : null}
                  {team.isMine ? (
                    <span className="shrink-0 rounded-sm bg-muted px-1 text-[10px] text-muted-foreground">
                      {t("teamboard.myTeam")}
                    </span>
                  ) : null}
                </span>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Popover>
          <PopoverTrigger
            render={
              <Button
                variant="outline"
                disabled={!board}
                aria-label={t("teamboard.groupFilter")}
                className="w-full justify-between sm:w-56"
              />
            }
          >
            <span className="truncate">
              {selectedGroupIds.length === 0
                ? t("teamboard.allGroups")
                : t("teamboard.nGroups", { n: selectedGroupIds.length })}
            </span>
            <ChevronDownIcon aria-hidden className="size-3.5 shrink-0 opacity-60" />
          </PopoverTrigger>
          <PopoverContent align="start" className="max-h-72 overflow-y-auto p-1.5">
            {board && board.groups.length > 0 ? (
              board.groups.map((group) => {
                const checked = selectedGroupIds.includes(group.id);
                return (
                  <label
                    key={group.id}
                    className="flex cursor-pointer items-center gap-2.5 rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  >
                    <Checkbox
                      checked={checked}
                      onCheckedChange={(next) => toggleGroup(group.id, next === true)}
                    />
                    <span className="min-w-0 flex-1 truncate">{group.name}</span>
                    <span className="tabular shrink-0 text-xs text-muted-foreground">
                      {group.memberCount}
                    </span>
                  </label>
                );
              })
            ) : (
              <p className="px-2 py-1.5 text-xs text-muted-foreground">
                {t("teamboard.noGroups")}
              </p>
            )}
          </PopoverContent>
        </Popover>

        {/* Period five-way, Sort by and search: the Leaderboard's controls,
            kept identical on the Teamboard (FR-2). */}
        <div className="-mx-4 overflow-x-auto px-4 pb-1 sm:mx-0 sm:overflow-visible sm:px-0 sm:pb-0 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
          {/* Base UI's ToggleGroup is array-valued even in single-select mode. */}
          <ToggleGroup
            value={[period]}
            onValueChange={(value) => {
              const next = value[0] as Period | undefined;
              if (!next) return;
              setPendingPeriod(next);
              // `from` only ever means "the viewer's local today" — carry it
              // into a switch to Today and drop it everywhere else.
              pushQuery({
                period: next,
                page: null,
                from: next === "today" ? toLocalDateString(new Date()) : null,
              });
            }}
            variant="outline"
            aria-label={t("teamboard.periodFilter")}
            className="[&>*]:h-10 [&>*]:px-3.5 sm:[&>*]:h-8 sm:[&>*]:px-3"
          >
            {PERIODS.map((p) => (
              <ToggleGroupItem key={p.value} value={p.value}>
                {t(PERIOD_LABEL_KEYS[p.value])}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>

        <div className="flex items-center gap-2">
          <ToggleGroup
            value={[sortBy]}
            onValueChange={(value) => {
              const next = value[0] as SortBy | undefined;
              if (!next) return;
              setSortBy(next);
              pushQuery({ sortBy: next, page: null });
            }}
            variant="outline"
            aria-label={t("teamboard.sortFilter")}
            className="[&>*]:h-10 [&>*]:px-3.5 sm:[&>*]:h-8 sm:[&>*]:px-3"
          >
            <ToggleGroupItem value="tokens">{t("teamboard.sortTokens")}</ToggleGroupItem>
            <ToggleGroupItem value="cost">{t("teamboard.sortCost")}</ToggleGroupItem>
          </ToggleGroup>

          <form
            className="relative flex-1 sm:ml-auto sm:flex-none"
            onSubmit={(event) => {
              event.preventDefault();
              pushQuery({ search: search.trim() || null, page: null });
            }}
          >
            <SearchIcon className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
              ref={searchRef}
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder={t("teamboard.searchPlaceholder")}
              aria-label={t("teamboard.searchAria")}
              className="h-10 w-full pl-8 text-sm sm:h-8 sm:w-56"
            />
          </form>
        </div>
      </div>

      {board ? (
        <div
          className={cn(
            "mt-4 overflow-hidden rounded-lg border transition-opacity",
            pending && "opacity-50"
          )}
          aria-busy={pending}
        >
          <Table>
            <TableHeader>
              <TableRow className="hover:bg-transparent">
                <TableHead className="w-12 pl-4 sm:pl-6">#</TableHead>
                <TableHead>{t("teamboard.colDeveloper")}</TableHead>
                {/* No Team column: the board is already scoped to one team. */}
                <MembershipColumnHeaders
                  includeTeam={false}
                  groupLabel={t("teamboard.colGroup")}
                />
                <TableHead className="pr-4 text-right sm:hidden">
                  {t("teamboard.colUsage")}
                </TableHead>
                <TableHead className="hidden w-44 p-0 text-right sm:table-cell">
                  <FormatToggle
                    label="Tokens"
                    compact={tokenFormat === "compact"}
                    onToggle={toggleFormat}
                    titles={formatTitles}
                  />
                </TableHead>
                <TableHead className="hidden w-32 p-0 pr-4 text-right sm:table-cell">
                  <FormatToggle
                    label="Cost"
                    compact={tokenFormat === "compact"}
                    onToggle={toggleFormat}
                    titles={formatTitles}
                  />
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {members.map((member) => (
                <DeveloperRow
                  key={member.userId}
                  user={member}
                  isSelf={currentUserId === member.userId}
                  max={max}
                  sortBy={sortBy}
                  tokenFormat={tokenFormat}
                  hideTeam
                />
              ))}
            </TableBody>
          </Table>

          {members.length === 0 && (
            <Empty className="border-0">
              <EmptyHeader>
                {/* Search, group filter and a quiet period are different
                    situations; the copy says which one happened. */}
                {appliedSearch ? (
                  <>
                    <EmptyTitle>{t("teamboard.noSearchTitle")}</EmptyTitle>
                    <EmptyDescription>
                      {t("teamboard.noSearchDesc", { q: appliedSearch })}
                    </EmptyDescription>
                  </>
                ) : selectedGroupIds.length > 0 ? (
                  <>
                    <EmptyTitle>{t("teamboard.noMatchTitle")}</EmptyTitle>
                    <EmptyDescription>{t("teamboard.noMatchDesc")}</EmptyDescription>
                  </>
                ) : (
                  <>
                    <EmptyTitle>{t("teamboard.nothingTitle")}</EmptyTitle>
                    <EmptyDescription>{t("teamboard.nothingDesc")}</EmptyDescription>
                  </>
                )}
              </EmptyHeader>
            </Empty>
          )}
        </div>
      ) : (
        <div className="mt-4 overflow-hidden rounded-lg border">
          <Empty className="border-0">
            <EmptyHeader>
              <EmptyTitle>{t("teamboard.emptyTitle")}</EmptyTitle>
              <EmptyDescription>{t("teamboard.emptyDesc")}</EmptyDescription>
            </EmptyHeader>
          </Empty>
        </div>
      )}

      {pagination && pagination.totalPages > 1 && (
        <nav
          className="mt-6 flex items-center justify-between"
          aria-label={t("teamboard.paginationAria")}
        >
          <Button
            variant="outline"
            disabled={!pagination.hasPrev}
            className="h-10 sm:h-8"
            onClick={() => pushQuery({ page: String(pagination.page - 1) })}
          >
            {t("teamboard.prevPage")}
          </Button>
          <span className="tabular text-xs text-muted-foreground">
            {t("teamboard.pageOf", {
              page: pagination.page,
              total: pagination.totalPages,
            })}
          </span>
          <Button
            variant="outline"
            disabled={!pagination.hasNext}
            className="h-10 sm:h-8"
            onClick={() => pushQuery({ page: String(pagination.page + 1) })}
          >
            {t("teamboard.nextPage")}
          </Button>
        </nav>
      )}
    </div>
  );
}
