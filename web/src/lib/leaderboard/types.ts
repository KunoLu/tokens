export type Period =
  | "all"
  | "month"
  | "last-month"
  | "week"
  | "today"
  | "custom";
export type SortBy = "tokens" | "cost";

export interface LeaderboardTeamRef {
  id: string;
  name: string;
  slug: string;
}

export interface LeaderboardGroupRef {
  id: string;
  name: string;
}

export interface LeaderboardUser {
  rank: number;
  userId: string;
  username: string;
  displayName: string | null;
  avatarUrl: string | null;
  team: LeaderboardTeamRef | null;
  group: LeaderboardGroupRef | null;
  totalTokens: number;
  totalCost: number;
}

export interface LeaderboardData {
  users: LeaderboardUser[];
  pagination: {
    page: number;
    limit: number;
    totalUsers: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  stats: {
    totalTokens: number;
    totalCost: number;
    uniqueUsers: number;
  };
  period: Period;
  sortBy: SortBy;
}
