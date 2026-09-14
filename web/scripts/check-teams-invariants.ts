/**
 * T4 safety net. Schema characterization plus domain permission/lifecycle
 * cases from web/features/team-management.feature.
 */
import postgres from "postgres";
import { TeamError } from "../src/lib/teams/errors";
import { canViewTeam } from "../src/lib/teams/visibility";
import { issueEmailToken } from "../src/lib/auth/emailTokens";
import { POST as postVerifyEmail } from "../src/app/api/auth/verify-email/route";
import {
  acceptInvitation,
  addGroupMember,
  createGroup,
  createTeam,
  declineInvitation,
  deleteTeam,
  disbandTeam,
  expireInvitations,
  getTeam,
  inviteMembers,
  leaveTeam,
  linkPendingInvitationsForEmail,
  patchMemberRole,
  patchTeam,
  removeGroupMember,
  searchUsers,
} from "../src/lib/teams/service";

const databaseUrl = process.env.DATABASE_URL;
if (!databaseUrl) {
  throw new Error("DATABASE_URL is required");
}

const sql = postgres(databaseUrl, { max: 1 });

function expect(name: string, condition: boolean, detail?: string) {
  if (!condition) {
    throw new Error(`${name} failed${detail ? `: ${detail}` : ""}`);
  }
  console.log(`ok - ${name}`);
}

function postgresErrorCode(err: unknown): string | undefined {
  if (err && typeof err === "object" && "code" in err && typeof err.code === "string") {
    return err.code;
  }
  return undefined;
}

async function expectSqlstate(
  name: string,
  code: string,
  run: () => Promise<unknown>
) {
  await sql`SAVEPOINT expect_sqlstate`;
  try {
    await run();
  } catch (err) {
    await sql`ROLLBACK TO SAVEPOINT expect_sqlstate`;
    if (postgresErrorCode(err) !== code) throw err;
    console.log(`ok - ${name}`);
    return;
  }
  await sql`ROLLBACK TO SAVEPOINT expect_sqlstate`;
  throw new Error(`${name} failed: expected SQLSTATE ${code}`);
}

try {
  await sql`BEGIN`;
  try {
    const stamp = Date.now().toString(36);
    const [userA] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username")
      VALUES (${`t4a_${stamp}`})
      RETURNING "id"
    `;
    const [userB] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username")
      VALUES (${`t4b_${stamp}`})
      RETURNING "id"
    `;
    const [team] = await sql<{ id: string }[]>`
      INSERT INTO "teams" ("name", "slug", "created_by")
      VALUES ('T4 Net', ${`t4-net-${stamp}`}, ${userA.id})
      RETURNING "id"
    `;
    await sql`
      INSERT INTO "team_members" ("team_id", "user_id", "role")
      VALUES (${team.id}, ${userA.id}, 'admin')
    `;

    await expectSqlstate(
      "INV-1 rejects a second team membership for the same user",
      "23505",
      () => sql`
        INSERT INTO "team_members" ("team_id", "user_id", "role")
        VALUES (${team.id}, ${userA.id}, 'member')
      `
    );

    const [group] = await sql<{ id: string }[]>`
      INSERT INTO "groups" ("team_id", "name", "created_by")
      VALUES (${team.id}, 'Alpha', ${userA.id})
      RETURNING "id"
    `;
    const [groupB] = await sql<{ id: string }[]>`
      INSERT INTO "groups" ("team_id", "name", "created_by")
      VALUES (${team.id}, 'Beta', ${userA.id})
      RETURNING "id"
    `;
    await sql`
      INSERT INTO "group_members" ("group_id", "user_id")
      VALUES (${group.id}, ${userA.id})
    `;
    await expectSqlstate(
      "INV-3 rejects a second group membership for the same user",
      "23505",
      () => sql`
        INSERT INTO "group_members" ("group_id", "user_id")
        VALUES (${groupB.id}, ${userA.id})
      `
    );

    await sql`
      INSERT INTO "team_invitations" (
        "team_id", "invited_email", "invited_by", "token_hash", "expires_at"
      )
      VALUES (
        ${team.id},
        ${`t4-${stamp}@example.com`},
        ${userA.id},
        ${"c".repeat(64)},
        now() + interval '7 days'
      )
    `;
    await expectSqlstate(
      "invitation CHECK rejects missing user and email",
      "23514",
      () => sql`
        INSERT INTO "team_invitations" (
          "team_id", "invited_by", "token_hash", "expires_at"
        )
        VALUES (
          ${team.id},
          ${userB.id},
          ${"d".repeat(64)},
          now() + interval '7 days'
        )
      `
    );

    expect("schema characterization complete", true);
  } finally {
    await sql`ROLLBACK`;
  }

  const stamp = `dom_${Date.now().toString(36)}`;
  const [admin] = await sql<{ id: string }[]>`
    INSERT INTO "users" ("username", "email")
    VALUES (${`t4adm_${stamp}`}, ${`t4adm_${stamp}@example.com`})
    RETURNING "id"
  `;
  const [member] = await sql<{ id: string }[]>`
    INSERT INTO "users" ("username", "email")
    VALUES (${`t4mem_${stamp}`}, ${`t4mem_${stamp}@example.com`})
    RETURNING "id"
  `;
  const [outsider] = await sql<{ id: string }[]>`
    INSERT INTO "users" ("username")
    VALUES (${`t4out_${stamp}`})
    RETURNING "id"
  `;

  async function expectTeamStatus(
    name: string,
    status: number,
    run: () => Promise<unknown>
  ) {
    try {
      await run();
    } catch (err) {
      if (err instanceof TeamError && err.status === status) {
        console.log(`ok - ${name}`);
        return;
      }
      throw err;
    }
    throw new Error(`${name} failed: expected TeamError ${status}`);
  }

  try {
    const team = await createTeam(admin.id, {
      name: `T4 Dom ${stamp}`,
      visibility: "private",
    });
    expect("createTeam makes an active private team", team.status === "active");

    await expectTeamStatus(
      "INV-1 createTeam rejected when already on a team",
      409,
      () => createTeam(admin.id, { name: "Second" })
    );

    await sql`
      INSERT INTO "team_members" ("team_id", "user_id", "role")
      VALUES (${team.id}, ${member.id}, 'member')
    `;

    await expectTeamStatus(
      "member cannot patch team",
      403,
      () => patchTeam(team.id, member.id, { name: "Hacked" })
    );
    await expectTeamStatus(
      "member cannot invite",
      403,
      () => inviteMembers(team.id, member.id, [{ username: `t4out_${stamp}` }])
    );
    await expectTeamStatus(
      "admin cannot leave",
      409,
      () => leaveTeam(team.id, admin.id)
    );
    await expectTeamStatus(
      "private team is 404 to outsider",
      404,
      () => getTeam(team.id, outsider.id)
    );
    await expectTeamStatus(
      "private team is 404 when signed out",
      404,
      () => getTeam(team.id, null)
    );
    await expectTeamStatus(
      "outsider cannot delete a private team",
      404,
      () => deleteTeam(team.id, outsider.id)
    );

    expect(
      "INV-10 member can view private",
      canViewTeam({ status: "active", visibility: "private" }, true)
    );
    expect(
      "INV-10 outsider cannot view private",
      !canViewTeam({ status: "active", visibility: "private" }, false)
    );
    expect(
      "INV-10 public active is visible",
      canViewTeam({ status: "active", visibility: "public" }, false)
    );
    expect(
      "INV-10 public disbanded is hidden",
      !canViewTeam({ status: "disbanded", visibility: "public" }, false)
    );
    expect(
      "INV-10 creator cannot view disbanded without membership",
      !canViewTeam({ status: "disbanded", visibility: "private" }, false)
    );

    await expectTeamStatus(
      "outsider searchUsers is 403",
      403,
      () => searchUsers(outsider.id, "t4")
    );
    const nameHits = await searchUsers(admin.id, `t4mem_${stamp}`);
    expect("searchUsers matches username", nameHits[0]?.username === `t4mem_${stamp}`);
    expect(
      "searchUsers omits email on username query",
      nameHits.every((row) => !("email" in row))
    );
    const emailHits = await searchUsers(admin.id, `t4mem_${stamp}@example.com`);
    expect(
      "email search matches user without echoing email",
      emailHits[0]?.username === `t4mem_${stamp}` &&
        emailHits.every((row) => !("email" in row))
    );

    const [s1] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username") VALUES (${`t4s1_${stamp}`}) RETURNING "id"
    `;
    const [s2] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username") VALUES (${`t4s2_${stamp}`}) RETURNING "id"
    `;
    const [s3] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username") VALUES (${`t4s3_${stamp}`}) RETURNING "id"
    `;
    await sql`
      INSERT INTO "team_members" ("team_id", "user_id", "role")
      VALUES
        (${team.id}, ${s1.id}, 'member'),
        (${team.id}, ${s2.id}, 'member'),
        (${team.id}, ${s3.id}, 'member')
    `;
    await patchMemberRole(team.id, admin.id, s1.id, "subadmin");
    await expectTeamStatus(
      "subadmin cannot assign subadmin",
      403,
      () => patchMemberRole(team.id, s1.id, s3.id, "subadmin")
    );
    await expectTeamStatus(
      "subadmin cannot disband",
      403,
      () => disbandTeam(team.id, s1.id)
    );
    await patchMemberRole(team.id, admin.id, s2.id, "subadmin");
    await expectTeamStatus(
      "third subadmin is rejected",
      409,
      () => patchMemberRole(team.id, admin.id, s3.id, "subadmin")
    );

    const [capAdmin] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username") VALUES (${`t4capadm_${stamp}`}) RETURNING "id"
    `;
    const [cap1] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username") VALUES (${`t4cap1_${stamp}`}) RETURNING "id"
    `;
    const [cap2] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username") VALUES (${`t4cap2_${stamp}`}) RETURNING "id"
    `;
    const [cap3] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username") VALUES (${`t4cap3_${stamp}`}) RETURNING "id"
    `;
    const capTeam = await createTeam(capAdmin.id, {
      name: `T4 Cap ${stamp}`,
      visibility: "private",
    });
    await sql`
      INSERT INTO "team_members" ("team_id", "user_id", "role")
      VALUES
        (${capTeam.id}, ${cap1.id}, 'member'),
        (${capTeam.id}, ${cap2.id}, 'member'),
        (${capTeam.id}, ${cap3.id}, 'member')
    `;
    await patchMemberRole(capTeam.id, capAdmin.id, cap1.id, "subadmin");
    await Promise.allSettled([
      patchMemberRole(capTeam.id, capAdmin.id, cap2.id, "subadmin"),
      patchMemberRole(capTeam.id, capAdmin.id, cap3.id, "subadmin"),
    ]);
    const capRoles = await sql<{ count: number }[]>`
      SELECT count(*)::int AS count FROM "team_members"
      WHERE "team_id" = ${capTeam.id} AND "role" = 'subadmin'
    `;
    expect("concurrent subadmin promotions stay at cap", capRoles[0].count <= 2);
    await patchMemberRole(team.id, admin.id, s3.id, "admin");

    const roles = await sql<{ user_id: string; role: string }[]>`
      SELECT "user_id", "role" FROM "team_members" WHERE "team_id" = ${team.id}
    `;
    const roleByUser = new Map(roles.map((row) => [row.user_id, row.role]));
    expect("admin transfer demotes the previous admin", roleByUser.get(admin.id) === "member");
    expect("admin transfer promotes the target", roleByUser.get(s3.id) === "admin");
    await patchMemberRole(team.id, s3.id, admin.id, "admin");

    const emailInvite = await inviteMembers(team.id, admin.id, [
      { email: `t4own_${stamp}@example.com` },
    ]);
    const emailInviteId = emailInvite.created[0]?.id;
    expect("email-only invite created", Boolean(emailInviteId));
    await expectTeamStatus(
      "wrong user cannot accept email-only invite",
      403,
      () => acceptInvitation(emailInviteId, outsider.id)
    );
    const [owner] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username", "email")
      VALUES (${`t4own_${stamp}`}, ${`t4own_${stamp}@example.com`})
      RETURNING "id"
    `;
    await expectTeamStatus(
      "unverified email owner cannot accept",
      403,
      () => acceptInvitation(emailInviteId, owner.id)
    );
    await sql`
      UPDATE "users" SET "email_verified_at" = now() WHERE "id" = ${owner.id}
    `;
    await declineInvitation(emailInviteId, owner.id);
    expect("verified email owner can decline email-only invite", true);

    const accEmail = `t4acc_${stamp}@example.com`;
    const accInvite = await inviteMembers(team.id, admin.id, [{ email: accEmail }]);
    const accInviteId = accInvite.created[0]?.id;
    expect("accept-path email-only invite created", Boolean(accInviteId));
    const [accUser] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username", "email")
      VALUES (${`t4acc_${stamp}`}, ${accEmail})
      RETURNING "id"
    `;
    await expectTeamStatus(
      "unverified accept-path owner cannot accept",
      403,
      () => acceptInvitation(accInviteId, accUser.id)
    );
    await sql`
      UPDATE "users" SET "email_verified_at" = now() WHERE "id" = ${accUser.id}
    `;
    await acceptInvitation(accInviteId, accUser.id);
    const accMembership = await sql<{ role: string }[]>`
      SELECT "role" FROM "team_members"
      WHERE "team_id" = ${team.id} AND "user_id" = ${accUser.id}
    `;
    expect(
      "verified email owner can accept email-only invite",
      accMembership[0]?.role === "member"
    );
    const teamB = await createTeam(outsider.id, {
      name: `T4 B ${stamp}`,
      visibility: "private",
    });
    const groupB = await createGroup(teamB.id, outsider.id, { name: "Other" });
    await expectTeamStatus(
      "cannot delete group members on another team's group",
      404,
      () => removeGroupMember(team.id, groupB.id, admin.id, member.id)
    );

    const firstInvite = await inviteMembers(team.id, admin.id, [
      { username: `t4out_${stamp}` },
    ]);
    expect("invite creates a pending row", firstInvite.created.length === 1);
    const secondInvite = await inviteMembers(team.id, admin.id, [
      { username: `t4out_${stamp}` },
    ]);
    expect(
      "duplicate pending invite is skipped",
      secondInvite.created.some((row) => row.skipped)
    );
    await expectTeamStatus(
      "unknown username invite is 400",
      400,
      () => inviteMembers(team.id, admin.id, [{ username: `nobody-here-${stamp}` }])
    );

    const [nameOnly] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username", "email")
      VALUES (${`t4nonly_${stamp}`}, ${`t4nonly_${stamp}@example.com`})
      RETURNING "id"
    `;
    const nameInvite = await inviteMembers(team.id, admin.id, [
      { username: `t4nonly_${stamp}` },
    ]);
    const nameInviteId = nameInvite.created[0]?.id;
    expect("username-only invite created", Boolean(nameInviteId));
    const storedNameInvite = await sql<{ invited_email: string | null }[]>`
      SELECT "invited_email" FROM "team_invitations" WHERE "id" = ${nameInviteId}
    `;
    expect(
      "username-only invite does not persist invited_email",
      storedNameInvite[0]?.invited_email == null
    );
    await acceptInvitation(nameInviteId, nameOnly.id);
    const nameMem = await sql<{ role: string }[]>`
      SELECT "role" FROM "team_members"
      WHERE "team_id" = ${team.id} AND "user_id" = ${nameOnly.id}
    `;
    expect(
      "username-only invite accepts without emailVerifiedAt",
      nameMem[0]?.role === "member"
    );

    const bfEmail = `t4bf_${stamp}@example.com`;
    const bfInvite = await inviteMembers(team.id, admin.id, [{ email: bfEmail }]);
    const [bfUser] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username", "email")
      VALUES (${`t4bf_${stamp}`}, ${bfEmail})
      RETURNING "id"
    `;
    const verifyLink = await issueEmailToken(bfUser.id, "verify_email");
    const verifyToken = new URL(verifyLink).searchParams.get("token") ?? "";
    const verifyResponse = await postVerifyEmail(
      new Request("http://localhost:3000/api/auth/verify-email", {
        method: "POST",
        headers: {
          Origin: "http://localhost:3000",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ token: verifyToken }),
      })
    );
    expect("verify-email route returns ok", verifyResponse.ok);
    const verified = await sql<{ email_verified_at: Date | null }[]>`
      SELECT "email_verified_at" FROM "users" WHERE "id" = ${bfUser.id}
    `;
    expect("verify-email marks email_verified_at", Boolean(verified[0]?.email_verified_at));
    const filled = await sql<{ invited_user_id: string | null; status: string }[]>`
      SELECT "invited_user_id", "status" FROM "team_invitations"
      WHERE "id" = ${bfInvite.created[0].id}
    `;
    expect(
      "verify-email backfill fills invited_user_id",
      filled[0].invited_user_id === bfUser.id && filled[0].status === "pending"
    );

    const winEmail = `t4win_${stamp}@example.com`;
    const [winUser] = await sql<{ id: string }[]>`
      INSERT INTO "users" ("username", "email")
      VALUES (${`t4win_${stamp}`}, ${winEmail})
      RETURNING "id"
    `;
    const [emailRow] = await sql<{ id: string }[]>`
      INSERT INTO "team_invitations" (
        "team_id", "invited_email", "invited_by", "token_hash", "expires_at", "created_at"
      )
      VALUES (
        ${team.id},
        ${winEmail},
        ${admin.id},
        ${`e${stamp}`.padEnd(64, "0")},
        now() + interval '7 days',
        now() - interval '2 days'
      )
      RETURNING "id"
    `;
    const [userRow] = await sql<{ id: string }[]>`
      INSERT INTO "team_invitations" (
        "team_id", "invited_user_id", "invited_email", "invited_by", "token_hash", "expires_at", "created_at"
      )
      VALUES (
        ${team.id},
        ${winUser.id},
        ${winEmail},
        ${admin.id},
        ${`u${stamp}`.padEnd(64, "1")},
        now() + interval '7 days',
        now() - interval '1 day'
      )
      RETURNING "id"
    `;
    await linkPendingInvitationsForEmail(winUser.id, winEmail);
    const winStates = await sql<{ id: string; status: string; invited_user_id: string | null }[]>`
      SELECT "id", "status", "invited_user_id" FROM "team_invitations"
      WHERE "id" IN (${emailRow.id}, ${userRow.id})
    `;
    const byInvite = new Map(winStates.map((row) => [row.id, row]));
    expect(
      "earlier email invite wins backfill",
      byInvite.get(emailRow.id)?.status === "pending" &&
        byInvite.get(emailRow.id)?.invited_user_id === winUser.id
    );
    expect(
      "later user invite is superseded",
      byInvite.get(userRow.id)?.status === "superseded"
    );

    const expiredInviteId = firstInvite.created[0]?.id;
    await sql`
      UPDATE "team_invitations"
      SET "expires_at" = now() - interval '1 day'
      WHERE "team_id" = ${team.id} AND "status" = 'pending'
    `;
    const expired = await expireInvitations();
    expect("expireInvitations marks overdue pending rows", expired >= 1);
    try {
      await acceptInvitation(expiredInviteId, outsider.id);
      throw new Error("expired invite accept should fail");
    } catch (err) {
      expect(
        "expired invite cannot be accepted",
        err instanceof TeamError &&
          err.status === 409 &&
          err.message === "Invitation has expired"
      );
    }

    const group = await createGroup(team.id, admin.id, { name: "G1" });
    await addGroupMember(team.id, group.id, admin.id, member.id);
    await expectTeamStatus(
      "INV-2 outsider cannot join group",
      409,
      () => addGroupMember(team.id, group.id, admin.id, outsider.id)
    );
    const group2 = await createGroup(team.id, admin.id, { name: "G2" });
    await addGroupMember(team.id, group2.id, admin.id, member.id);
    const gmem = await sql<{ name: string }[]>`
      SELECT g."name" FROM "group_members" gm
      JOIN "groups" g ON g."id" = gm."group_id"
      WHERE gm."user_id" = ${member.id}
    `;
    expect(
      "INV-3 transfer keeps a single group",
      gmem.length === 1 && gmem[0].name === "G2"
    );
    await leaveTeam(team.id, member.id);
    const leftover = await sql<{ count: number }[]>`
      SELECT count(*)::int AS count FROM "group_members" WHERE "user_id" = ${member.id}
    `;
    expect("leaveTeam cascades out of group", leftover[0].count === 0);

    await expectTeamStatus(
      "active team cannot be deleted",
      409,
      () => deleteTeam(team.id, admin.id)
    );
    await disbandTeam(team.id, admin.id);
    const membersLeft = await sql<{ count: number }[]>`
      SELECT count(*)::int AS count FROM "team_members" WHERE "team_id" = ${team.id}
    `;
    expect("disband clears all members", membersLeft[0].count === 0);
    await deleteTeam(team.id, admin.id);
    expect("created_by can delete a disbanded empty team", true);
  } finally {
    await sql`DELETE FROM "users" WHERE "username" LIKE ${`t4%${stamp}`}`;
  }
} finally {
  await sql.end();
}
