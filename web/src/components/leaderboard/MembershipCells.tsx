import { TableCell, TableHead } from "@/components/ui/table";
import { cn } from "@/lib/utils";
import type {
  LeaderboardGroupRef,
  LeaderboardTeamRef,
} from "@/lib/leaderboard/types";

export function MembershipBadges({
  team,
  group,
}: {
  team?: LeaderboardTeamRef | null;
  group?: LeaderboardGroupRef | null;
}) {
  if (!team && !group) return null;
  return (
    <span className="mt-0.5 flex min-w-0 flex-wrap gap-1 sm:hidden">
      {team ? (
        <span className="truncate rounded-sm bg-muted px-1 text-[10px] text-muted-foreground">
          {team.name}
        </span>
      ) : null}
      {group ? (
        <span className="truncate rounded-sm bg-muted px-1 text-[10px] text-muted-foreground">
          {group.name}
        </span>
      ) : null}
    </span>
  );
}

export function MembershipColumnHeaders({
  includeTeam = true,
}: {
  includeTeam?: boolean;
}) {
  return (
    <>
      {includeTeam ? (
        <TableHead className="hidden max-w-[9rem] sm:table-cell">Team</TableHead>
      ) : null}
      <TableHead className="hidden max-w-[9rem] sm:table-cell">Group</TableHead>
    </>
  );
}

export function MembershipCells({
  team,
  group,
  includeTeam = true,
}: {
  team?: LeaderboardTeamRef | null;
  group?: LeaderboardGroupRef | null;
  includeTeam?: boolean;
}) {
  return (
    <>
      {includeTeam ? (
        <TableCell
          className={cn(
            "hidden max-w-[9rem] truncate text-sm text-muted-foreground sm:table-cell"
          )}
        >
          {team?.name ?? ""}
        </TableCell>
      ) : null}
      <TableCell
        className={cn(
          "hidden max-w-[9rem] truncate text-sm text-muted-foreground sm:table-cell"
        )}
      >
        {group?.name ?? ""}
      </TableCell>
    </>
  );
}
