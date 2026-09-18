import { expect, test, type Browser, type Page } from "@playwright/test";
import {
  deleteFixtureUsers,
  fixtureUser,
  registerUser,
  type FixtureUser,
} from "./users";

// T8 /u/[username] Team/Group block + owner-only leave. Traces
// web/features/profile-team-membership.feature:
// 有归属时才展示团队区块 (团队+分组 / 仅团队 / 无归属不渲染),
// 只有本人可以看到并执行退出操作 (本人可见; 他人与未登录访客不可见),
// 退出分组不影响团队归属, 普通成员退出团队会同时退出分组 (INV-8),
// admin 退出团队被 409 阻断并提示先移交或解散.
// Fixtures are synthetic t5e2e-* accounts on the local dev stack only;
// afterAll deletes them and cascades remove team/membership/session rows.

const runId = Date.now().toString(36);
const admin = fixtureUser(runId, "a");
const member = fixtureUser(runId, "b");
const loner = fixtureUser(runId, "c");
const teamName = `t5e2e-t8-team-${runId}`;
const groupName = `t5e2e-t8-g-${runId}`;

const ORIGIN = { Origin: "http://localhost:3000" };
const MEMBERSHIP_REGION = "Membership";
const LEAVE_TEAM = "Leave team";
const LEAVE_GROUP = "Leave group";
const ADMIN_GUARD_COPY = "Transfer admin or disband the team before leaving";

/** Each account gets its own browser context so the sessions never clash. */
async function registerInNewContext(
  browser: Browser,
  user: FixtureUser
): Promise<Page> {
  const context = await browser.newContext();
  const page = await context.newPage();
  await registerUser(page, user);
  return page;
}

/**
 * The leave buttons render only after the client session check resolves, so
 * "no buttons" assertions must wait for that fetch. The wait is registered
 * before goto or it can miss an already-finished response.
 */
async function gotoProfileWithSessionCheck(
  page: Page,
  username: string
): Promise<void> {
  const sessionCheck = page.waitForResponse((res) =>
    res.url().includes("/api/auth/session")
  );
  await page.goto(`/u/${username}`);
  await sessionCheck;
}

test.afterAll(async () => {
  await deleteFixtureUsers([admin.username, member.username, loner.username]);
});

// Cold `next dev` compiles each route on first hit; register ×3 plus team,
// group, invite and several page loads can exceed the default 60s budget.
test.setTimeout(180_000);

test("个人主页: Team/Group 区块展示与 owner-only 退出", async ({
  page,
  browser,
}) => {
  let teamId = "";
  let groupId = "";
  let memberPage!: Page;

  await test.step("setup: admin 建团建分组并自入组; member 受邀入团; loner 无归属", async () => {
    memberPage = await registerInNewContext(browser, member);
    const lonerPage = await registerInNewContext(browser, loner);
    await lonerPage.context().close();
    await registerUser(page, admin);

    const teamRes = await page.request.post("/api/teams", {
      headers: ORIGIN,
      data: { name: teamName, visibility: "public" },
    });
    expect(
      teamRes.status(),
      `create team failed: ${teamRes.status()} ${await teamRes.text()}`
    ).toBe(201);
    teamId = ((await teamRes.json()) as { id: string }).id;

    const groupRes = await page.request.post(`/api/teams/${teamId}/groups`, {
      headers: ORIGIN,
      data: { name: groupName },
    });
    expect(
      groupRes.status(),
      `create group failed: ${groupRes.status()} ${await groupRes.text()}`
    ).toBe(201);
    groupId = ((await groupRes.json()) as { id: string }).id;

    const sessionRes = await page.request.get("/api/auth/session");
    const adminId = ((await sessionRes.json()) as { user: { id: string } })
      .user.id;
    const addSelfRes = await page.request.put(
      `/api/teams/${teamId}/groups/${groupId}/members/${adminId}`,
      { headers: ORIGIN }
    );
    expect(
      addSelfRes.status(),
      `add self to group failed: ${await addSelfRes.text()}`
    ).toBe(200);
    const inviteRes = await page.request.post(
      `/api/teams/${teamId}/members`,
      { headers: ORIGIN, data: { items: [{ username: member.username }] } }
    );
    expect(
      inviteRes.ok(),
      `invite failed: ${inviteRes.status()} ${await inviteRes.text()}`
    ).toBe(true);
    const { created } = (await inviteRes.json()) as {
      created: Array<{ id: string }>;
    };
    expect(
      created[0]?.id,
      "invite should create one invitation row"
    ).toBeTruthy();

    const invitationsRes = await memberPage.request.get("/api/me/invitations");
    expect(
      invitationsRes.status(),
      `GET /api/me/invitations: ${await invitationsRes.text()}`
    ).toBe(200);
    const { invitations } = (await invitationsRes.json()) as {
      invitations: Array<{ id: string }>;
    };
    expect(invitations.length, "member should have one invitation").toBe(1);
    expect(invitations[0].id).toBe(created[0].id);

    const acceptRes = await memberPage.request.post(
      `/api/invitations/${invitations[0].id}/accept`,
      { headers: ORIGIN }
    );
    expect(
      acceptRes.ok(),
      `accept failed: ${acceptRes.status()} ${await acceptRes.text()}`
    ).toBe(true);
  });

  await test.step("访客: admin 主页展示团队与分组名称，无任何退出操作", async () => {
    const visitorPage = await (await browser.newContext()).newPage();
    await gotoProfileWithSessionCheck(visitorPage, admin.username);
    const region = visitorPage.getByRole("region", {
      name: MEMBERSHIP_REGION,
    });
    await expect(region).toBeVisible();
    await expect(region).toContainText(teamName);
    await expect(region).toContainText(groupName);
    await expect(
      visitorPage.getByRole("button", { name: LEAVE_TEAM })
    ).toHaveCount(0);
    await expect(
      visitorPage.getByRole("button", { name: LEAVE_GROUP })
    ).toHaveCount(0);
    await visitorPage.context().close();
  });

  await test.step("访客: member 主页只有团队没有分组; loner 主页无归属区块", async () => {
    const visitorPage = await (await browser.newContext()).newPage();

    await gotoProfileWithSessionCheck(visitorPage, member.username);
    const memberRegion = visitorPage.getByRole("region", {
      name: MEMBERSHIP_REGION,
    });
    await expect(memberRegion).toBeVisible();
    await expect(memberRegion).toContainText(teamName);
    await expect(memberRegion).not.toContainText("Group:");

    await visitorPage.goto(`/u/${loner.username}`);
    await expect(
      visitorPage.getByRole("heading", { level: 1, name: loner.username })
    ).toBeVisible();
    await expect(
      visitorPage.getByRole("region", { name: MEMBERSHIP_REGION })
    ).toHaveCount(0);
    await visitorPage.context().close();
  });

  await test.step("他人视角: member 登录后看 admin 主页，无退出操作", async () => {
    await gotoProfileWithSessionCheck(memberPage, admin.username);
    const region = memberPage.getByRole("region", {
      name: MEMBERSHIP_REGION,
    });
    await expect(region).toBeVisible();
    await expect(region).toContainText(teamName);
    await expect(
      memberPage.getByRole("button", { name: LEAVE_TEAM })
    ).toHaveCount(0);
    await expect(
      memberPage.getByRole("button", { name: LEAVE_GROUP })
    ).toHaveCount(0);
  });

  await test.step("本人视角: admin 看到 退出 Team 与 退出 Group", async () => {
    await gotoProfileWithSessionCheck(page, admin.username);
    await expect(
      page.getByRole("button", { name: LEAVE_TEAM })
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: LEAVE_GROUP })
    ).toBeVisible();
  });

  await test.step("admin 看 member 主页: 仍是登录态但无退出操作", async () => {
    // Same session, different profile: the owner check must follow the
    // profile username, not stick to the first page it matched.
    await gotoProfileWithSessionCheck(page, member.username);
    const region = page.getByRole("region", { name: MEMBERSHIP_REGION });
    await expect(region).toBeVisible();
    await expect(region).toContainText(teamName);
    await expect(
      page.getByRole("button", { name: LEAVE_TEAM })
    ).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: LEAVE_GROUP })
    ).toHaveCount(0);
  });

  await test.step("普通成员退出团队: 团队与分组同时清空 (INV-8)", async () => {
    // Put the member in the group first so the leave has both to clear.
    const memberSession = await memberPage.request.get("/api/auth/session");
    const memberId = ((await memberSession.json()) as { user: { id: string } })
      .user.id;
    const addRes = await page.request.put(
      `/api/teams/${teamId}/groups/${groupId}/members/${memberId}`,
      { headers: ORIGIN }
    );
    expect(addRes.status(), `add member failed: ${await addRes.text()}`).toBe(
      200
    );

    await gotoProfileWithSessionCheck(memberPage, member.username);
    await memberPage.getByRole("button", { name: LEAVE_TEAM }).click();
    await expect(
      memberPage.getByRole("region", { name: MEMBERSHIP_REGION })
    ).toHaveCount(0);
    await memberPage.context().close();
  });

  await test.step("退出分组: 分组消失但团队保留", async () => {
    await gotoProfileWithSessionCheck(page, admin.username);
    await page.getByRole("button", { name: LEAVE_GROUP }).click();
    const region = page.getByRole("region", { name: MEMBERSHIP_REGION });
    await expect(region).toContainText(teamName);
    await expect(region).not.toContainText(groupName);
    await expect(
      page.getByRole("button", { name: LEAVE_GROUP })
    ).toHaveCount(0);
  });

  await test.step("admin 退出团队被阻断: 409 提示移交或解散，仍是成员", async () => {
    await page.getByRole("button", { name: LEAVE_TEAM }).click();
    await expect(page.getByText(ADMIN_GUARD_COPY)).toBeVisible();
    const region = page.getByRole("region", { name: MEMBERSHIP_REGION });
    await expect(region).toContainText(teamName);
  });
});
