import { expect, test } from "@playwright/test";
import { deleteFixtureUsers, fixtureUser, registerUser } from "./users";

// T7 /teamboard. Traces web/features/leaderboard-and-teamboard.feature:
// 尚未选择团队 (signed-out guide empty state), 登录用户默认选中自己的团队,
// 列中不包含 Team, 私有团队不出现在筛选器中, 直接请求他人的私有团队返回 404,
// Teamboard 保留 Leaderboard 的 Period/Sort by/搜索/分页, 团队榜 API 遵循 §9.3 参数命名.
// Fixtures are synthetic t5e2e-* accounts on the local dev stack only;
// afterAll deletes them and cascades remove team/membership/session rows.

test("未登录访问 /teamboard: 引导空态 + 保留筛选器，无成员列表", async ({ page }) => {
  await page.goto("/teamboard");

  await expect(
    page.getByRole("heading", { name: "Teamboard", level: 1 })
  ).toBeVisible();
  await expect(page.getByText("Select a team first")).toBeVisible();

  // The filter stays so the visitor can pick a public team…
  await expect(
    page.getByRole("combobox", { name: "Team filter" })
  ).toBeVisible();
  // …but no member data renders before a team is chosen.
  await expect(page.locator("#main-content tbody tr")).toHaveCount(0);
});

test.describe("私有团队可见性 (D-2)", () => {
  const runId = Date.now().toString(36);
  const admin = fixtureUser(runId, "a");
  const teamName = `t5e2e-t7-team-${runId}`;

  test.afterAll(async () => {
    await deleteFixtureUsers([admin.username]);
  });

  // Cold `next dev` compiles each route on first hit; register + /api/teams +
  // two page loads can exceed the default 60s budget.
  test.setTimeout(180_000);

  test("成员默认选中自己的私有团队；访客直接请求 404 且筛选器不列出", async ({
    page,
  }) => {
    let teamId = "";

    await test.step("注册并创建私有团队", async () => {
      await registerUser(page, admin);
      const res = await page.request.post("/api/teams", {
        headers: { Origin: "http://localhost:3000" },
        data: { name: teamName, visibility: "private" },
      });
      expect(
        res.status(),
        `create team failed: ${res.status()} ${await res.text()}`
      ).toBe(201);
      const created = (await res.json()) as { id: string };
      teamId = created.id;
      expect(teamId).toBeTruthy();
    });

    await test.step("成员打开 /teamboard: 默认选中自己的团队，无 Team 列", async () => {
      await page.goto("/teamboard");
      // Default selection renders the member table without any URL params.
      await expect(
        page.locator("#main-content tbody tr", { hasText: admin.username })
      ).toHaveCount(1);
      // Columns are # / Developer / Group / Tokens / Cost — no Team column.
      await expect(
        page.getByRole("columnheader", { name: "Team" })
      ).toHaveCount(0);
      await expect(
        page.getByRole("columnheader", { name: "Group" })
      ).toBeVisible();
    });
    await test.step("FR-2: Period 五选一 / Sort by / 搜索 chrome 可见", async () => {
      // The member table is already rendered (own team is the default
      // selection), so the FR-2 controls sit above it. Pagination is the
      // Leaderboard's: hidden until there is a second page, so a one-member
      // team does not grow Previous/Next.
      const periodGroup = page.getByRole("group", { name: "Period" });
      await expect(periodGroup).toBeVisible();
      for (const label of ["All time", "Today", "Week", "Month", "Last month"]) {
        await expect(
          periodGroup.getByRole("button", { name: label, exact: true })
        ).toBeVisible();
      }

      const sortGroup = page.getByRole("group", { name: "Sort by" });
      await expect(sortGroup.getByRole("button", { name: "Tokens" })).toBeVisible();
      await expect(sortGroup.getByRole("button", { name: "Cost" })).toBeVisible();

      await expect(
        page.getByRole("textbox", { name: "Search developers" })
      ).toBeVisible();

      await periodGroup.getByRole("button", { name: "Week" }).click();
      await expect(page).toHaveURL(/[?&]period=week/);
      await sortGroup.getByRole("button", { name: "Cost" }).click();
      await expect(page).toHaveURL(/[?&]sortBy=cost/);

      // Back must not leave stale controls: textbox and Sort by follow the
      // URL the server answered (ReviewerT7b P2). History so far:
      // /teamboard → ?period=week → +sortBy=cost → +search=<admin>.
      const searchBox = page.getByRole("textbox", { name: "Search developers" });
      await searchBox.fill(admin.username);
      await searchBox.press("Enter");
      await expect(page).toHaveURL(/[?&]search=/);

      await page.goBack();
      await expect(page).toHaveURL(/[?&]sortBy=cost/);
      await expect(searchBox).toHaveValue("");

      await page.goBack();
      await expect(page).not.toHaveURL(/[?&]sortBy=/);
      await expect(
        sortGroup.getByRole("button", { name: "Tokens" })
      ).toHaveAttribute("aria-pressed", "true");

      // Settled URL, not click-then-immediate-Back (that race is flaky):
      // load sortBy=cost fully, then open the tokens URL. Displayed Sort
      // by follows the URL even if a Cost RSC never becomes current.
      const here = new URL(page.url());
      const tokensPath = `${here.pathname}${here.search}`;
      here.searchParams.set("sortBy", "cost");
      await page.goto(`${here.pathname}${here.search}`);
      await expect(page).toHaveURL(/[?&]sortBy=cost/);
      await expect(
        sortGroup.getByRole("button", { name: "Cost" })
      ).toHaveAttribute("aria-pressed", "true");

      await page.goto(tokensPath);
      await expect(page).not.toHaveURL(/[?&]sortBy=/);
      await expect(
        sortGroup.getByRole("button", { name: "Tokens" })
      ).toHaveAttribute("aria-pressed", "true");
    });

    await test.step("API §9.3: teamId 返回榜单而不是 {teams}", async () => {
      const boardRes = await page.request.get(
        `/api/teamboard?teamId=${teamId}&period=all&sortBy=cost&page=1&search=${admin.username}`
      );
      expect(boardRes.status()).toBe(200);
      const board = (await boardRes.json()) as {
        team: { id: string };
        teams?: unknown;
        members: Array<{ username: string }>;
        pagination: { page: number };
        period: string;
        sortBy: string;
      };
      expect(board.team.id).toBe(teamId);
      expect(board.teams).toBeUndefined();
      expect(board.period).toBe("all");
      expect(board.sortBy).toBe("cost");
      expect(board.pagination.page).toBe(1);
      expect(board.members.map((m) => m.username)).toEqual([admin.username]);

      const listRes = await page.request.get("/api/teamboard");
      expect(listRes.status()).toBe(200);
      const list = (await listRes.json()) as { teams?: unknown[] };
      expect(Array.isArray(list.teams)).toBe(true);
    });

    await test.step("API §9.3: groupIds 只返回所选分组的成员", async () => {
      // Two groups, the admin in the first: filtering by group A must answer
      // with the admin, filtering by the empty group B with nobody — a loader
      // that drops or misnames groupIds fails one of the two.
      const origin = { Origin: "http://localhost:3000" };
      const createGroup = async (name: string) => {
        const res = await page.request.post(`/api/teams/${teamId}/groups`, {
          headers: origin,
          data: { name },
        });
        expect(res.status(), `create group failed: ${await res.text()}`).toBe(201);
        const group = (await res.json()) as { id: string };
        return group.id;
      };
      const groupA = await createGroup(`t5e2e-t7-ga-${runId}`);
      const groupB = await createGroup(`t5e2e-t7-gb-${runId}`);

      const allRes = await page.request.get(`/api/teamboard?teamId=${teamId}`);
      const all = (await allRes.json()) as {
        members: Array<{ userId: string; username: string }>;
      };
      const adminId = all.members.find(
        (member) => member.username === admin.username
      )?.userId;
      expect(adminId).toBeTruthy();

      const addRes = await page.request.put(
        `/api/teams/${teamId}/groups/${groupA}/members/${adminId}`,
        { headers: origin }
      );
      expect(addRes.status(), `add member failed: ${await addRes.text()}`).toBe(200);

      const filteredRes = await page.request.get(
        `/api/teamboard?teamId=${teamId}&groupIds=${groupA}`
      );
      expect(filteredRes.status()).toBe(200);
      const filtered = (await filteredRes.json()) as {
        members: Array<{ username: string }>;
      };
      expect(filtered.members.map((m) => m.username)).toEqual([admin.username]);

      const emptyRes = await page.request.get(
        `/api/teamboard?teamId=${teamId}&groupIds=${groupB}`
      );
      expect(emptyRes.status()).toBe(200);
      const empty = (await emptyRes.json()) as { members: unknown[] };
      expect(empty.members).toEqual([]);
    });

    await test.step("退出登录后：页面与 API 对该团队均 404，筛选器不列出", async () => {
      await page.context().clearCookies();

      const pageResponse = await page.goto(`/teamboard?team=${teamId}`);
      expect(pageResponse?.status()).toBe(404);

      // §9.3 name and the page-URL alias both 404 for a private-to-the-viewer
      // team — never 403, and never a fake 200 empty list.
      const apiByTeamId = await page.request.get(`/api/teamboard?teamId=${teamId}`);
      expect(apiByTeamId.status()).toBe(404);
      const apiResponse = await page.request.get(`/api/teamboard?team=${teamId}`);
      expect(apiResponse.status()).toBe(404);

      await page.goto("/teamboard");
      await page.getByRole("combobox", { name: "Team filter" }).click();
      await expect(
        page.getByRole("option").filter({ hasText: teamName })
      ).toHaveCount(0);
    });
  });
});
