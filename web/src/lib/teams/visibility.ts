import type { TeamStatus, TeamVisibility } from "./types";

export function canViewTeam(
  team: {
    status: string;
    visibility: string;
  },
  isMember: boolean
): boolean {
  if (isMember) return true;
  if (team.status === "active" && team.visibility === "public") return true;
  return false;
}

export function isTeamVisibility(value: string): value is TeamVisibility {
  return value === "public" || value === "private";
}

export function isTeamStatus(value: string): value is TeamStatus {
  return value === "active" || value === "disbanded";
}
