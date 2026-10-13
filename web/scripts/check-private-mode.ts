/**
 * Covers web/features/private-mode.feature and the ranking rule in
 * web/features/leaderboard-and-teamboard.feature.
 *
 * Public rankings omit a private account. Team management still lists them.
 * next/cache must be mocked before the loaders are imported; a static import
 * would bind the real unstable_cache and throw outside the Next server.
 */
import postgres from "postgres";
import { mock } from "bun:test";

mock.module("next/cache", () => ({
  unstable_cache: (fn: (...args: unknown[]) => unknown) => fn,
  revalidatePath: () => {},
  revalidateTag: () => {},
}));

process.env.DATABASE_POOL_MAX ??= "2";
process.env.DATABASE_SSL ??= "disable";

const databaseUrl = process.env.DATABASE_URL;
if (!databaseUrl) {
  throw new Error("DATABASE_URL is required");
}

const { getLeaderboardData } = await import("../src/lib/leaderboard/getLeaderboard");
const { getTeamboard } = await import("../src/lib/teamboard/getTeamboard");
const { listMembers, searchUsers } = await import("../src/lib/teams/service");

const sql = postgres(databaseUrl, { max: 1 });

function expect(name: string, condition: boolean, detail?: string) {
  if (!condition) {
    throw new Error(`${name} failed${detail ? `: ${detail}` : ""}`);
  }
  console.log(`ok - ${name}`);
}

const stamp = `priv_${Date.now().toString(36)}`;
const adminName = `priv_admin_${stamp}`;
const hiddenName = `priv_hidden_${stamp}`;
let adminId = "";
let hiddenId = "";
let teamId = "";

try {
  const [admin] = await sql<{ id: string }[]>`
    INSERT INTO "users" ("username", "email", "is_private")
    VALUES (${adminName}, ${`${adminName}@example.com`}, false)
    RETURNING "id"
  `;
  const [hidden] = await sql<{ id: string }[]>`
    INSERT INTO "users" ("username", "email", "is_private")
    VALUES (${hiddenName}, ${`${hiddenName}@example.com`}, true)
    RETURNING "id"
  `;
  adminId = admin.id;
  hiddenId = hidden.id;

  await sql`
    INSERT INTO "submissions" (
      "user_id", "total_tokens", "total_cost", "input_tokens", "output_tokens",
      "date_start", "date_end", "sources_used", "models_used"
    ) VALUES (
      ${hiddenId}, 1000000, 10, 500000, 500000,
      '2026-10-01', '2026-10-02', ARRAY['claude']::text[], ARRAY['claude-sonnet']::text[]
    )
  `;

  const [team] = await sql<{ id: string }[]>`
    INSERT INTO "teams" ("name", "slug", "created_by", "visibility")
    VALUES (${`Priv ${stamp}`}, ${`priv-${stamp}`}, ${adminId}, 'public')
    RETURNING "id"
  `;
  teamId = team.id;
  await sql`
    INSERT INTO "team_members" ("team_id", "user_id", "role")
    VALUES (${team.id}, ${adminId}, 'admin'), (${team.id}, ${hiddenId}, 'member')
  `;
  const [group] = await sql<{ id: string }[]>`
    INSERT INTO "groups" ("team_id", "name", "created_by")
    VALUES (${team.id}, 'hidden-group', ${adminId})
    RETURNING "id"
  `;
  await sql`
    INSERT INTO "group_members" ("group_id", "user_id")
    VALUES (${group.id}, ${hiddenId})
  `;

  const board = await getLeaderboardData("all", 1, 50, "tokens", hiddenName);
  expect(
    "private account is absent from leaderboard search",
    !board.users.some((user) => user.username === hiddenName),
    board.users.map((user) => user.username).join(","),
  );

  const foundAdmin = await searchUsers(adminId, adminName);
  expect(
    "public account remains searchable",
    foundAdmin.some((user) => user.username === adminName),
  );
  const foundHidden = await searchUsers(adminId, hiddenName);
  expect("username search omits private account", foundHidden.length === 0);

  const teamboard = await getTeamboard(team.id, [], null);
  const names = teamboard.members.map((member) => member.username);
  expect("public teamboard omits private member", !names.includes(hiddenName), names.join(","));
  expect("public teamboard still shows the admin", names.includes(adminName));
  const hiddenGroup = teamboard.groups.find((row) => row.name === "hidden-group");
  expect(
    "group count omits private member",
    hiddenGroup?.memberCount === 0,
    String(hiddenGroup?.memberCount),
  );

  const members = await listMembers(team.id, adminId);
  expect(
    "team management still lists the private member",
    members.some((member) => member.username === hiddenName),
  );
} finally {
  if (teamId) {
    await sql`DELETE FROM "teams" WHERE "id" = ${teamId}`;
  }
  if (adminId || hiddenId) {
    await sql`DELETE FROM "users" WHERE "id" IN (${adminId}, ${hiddenId})`;
  }
  await sql.end();
}
