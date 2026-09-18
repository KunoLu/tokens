export const TEAM_VISIBILITIES = ["public", "private"] as const;
export type TeamVisibility = (typeof TEAM_VISIBILITIES)[number];

export const TEAM_STATUSES = ["active", "disbanded"] as const;
export type TeamStatus = (typeof TEAM_STATUSES)[number];

export const TEAM_MEMBER_ROLES = ["admin", "subadmin", "member"] as const;
export type TeamMemberRole = (typeof TEAM_MEMBER_ROLES)[number];

export const GROUP_STATUSES = ["active", "disbanded"] as const;
export type GroupStatus = (typeof GROUP_STATUSES)[number];

export const TEAM_INVITATION_STATUSES = [
  "pending",
  "accepted",
  "declined",
  "revoked",
  "expired",
  "superseded",
] as const;
export type TeamInvitationStatus = (typeof TEAM_INVITATION_STATUSES)[number];
