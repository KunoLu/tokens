import { and, eq } from "drizzle-orm";
import { db, groupMembers, groups, teamMembers, teams, users } from "@/lib/db";
import {
  USERNAME_LOOKUP_LIMIT,
  getSingleUsernameMatch,
  usernameEqualsIgnoreCase,
} from "@/lib/db/usernameLookup";

/**
 * Team/Group affiliation shown on the public profile page. Page-only by
 * design (D-4): GET /api/users/[username] stays free of membership, so this
 * loader runs in u/[username]/page.tsx next to loadPublicProfileForPage and
 * is never called from getPublicProfileResponse.
 */
export interface ProfileMembership {
  team: { id: string; name: string; slug: string };
  group: { id: string; name: string } | null;
}

export async function loadProfileMembershipForPage(
  username: string
): Promise<ProfileMembership | null> {
  // Same join shape as the leaderboard's membership lookup: only active
  // teams/groups count as an affiliation.
  const rows = await db
    .select({
      teamId: teams.id,
      teamName: teams.name,
      teamSlug: teams.slug,
      groupId: groups.id,
      groupName: groups.name,
    })
    .from(users)
    .leftJoin(teamMembers, eq(teamMembers.userId, users.id))
    .leftJoin(
      teams,
      and(eq(teams.id, teamMembers.teamId), eq(teams.status, "active"))
    )
    .leftJoin(groupMembers, eq(groupMembers.userId, users.id))
    .leftJoin(
      groups,
      and(eq(groups.id, groupMembers.groupId), eq(groups.status, "active"))
    )
    .where(usernameEqualsIgnoreCase(username))
    .limit(USERNAME_LOOKUP_LIMIT);

  const row = getSingleUsernameMatch(rows, username);
  if (!row || !row.teamId || !row.teamName || !row.teamSlug) return null;
  return {
    team: { id: row.teamId, name: row.teamName, slug: row.teamSlug },
    group:
      row.groupId && row.groupName
        ? { id: row.groupId, name: row.groupName }
        : null,
  };
}
