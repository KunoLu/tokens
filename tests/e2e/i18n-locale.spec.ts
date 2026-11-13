import { expect, test, type Page } from "@playwright/test";
import { deleteFixtureUsers, fixtureUser, registerUser } from "./users";

// Traces the scenarios in web/features/i18n.feature (locale prefix cutover,
// PR A of the v27.2.1 upstream sync). Scenario names are kept in the test
// titles for traceability.
//
// The scheme under test: the language lives in the URL prefix
// (/zh-CN/..., /ja/...; unprefixed is English). tt_locale only remembers the
// choice for the root redirect; it never decides what an unprefixed page
// renders. A pre-cutover `tt_locale=zh` cookie still means Chinese.

const BASE_URL = "http://localhost:3000";

function mainNav(page: Page, name = "Main navigation") {
  return page.getByRole("navigation", { name });
}

async function switchLocale(page: Page, buttonName: string, optionName: string) {
  await page.getByRole("button", { name: buttonName }).click();
  // Clicking an option writes tt_locale and navigates to the prefixed address.
  await page.getByRole("menuitem", { name: optionName }).click();
}

test("按钮位置与选项: 语言按钮在主题切换左侧，选项恰好为 English / 简体中文 / 日本語", async ({
  page,
}) => {
  await page.goto("/leaderboard");

  const languageButton = page.getByRole("button", { name: "Language" });
  const themeButton = page.getByRole("button", {
    name: /Switch to (dark|light) theme/,
  });
  await expect(languageButton).toBeVisible();
  await expect(themeButton).toBeVisible();

  const languageBox = await languageButton.boundingBox();
  const themeBox = await themeButton.boundingBox();
  expect(languageBox).not.toBeNull();
  expect(themeBox).not.toBeNull();
  expect(languageBox!.x + languageBox!.width).toBeLessThanOrEqual(themeBox!.x);

  await languageButton.click();
  const items = page.getByRole("menuitem");
  await expect(items).toHaveCount(3);
  await expect(items.nth(0)).toHaveText("English");
  await expect(items.nth(1)).toHaveText("简体中文");
  await expect(items.nth(2)).toHaveText("日本語");
});

test("切换到简体中文: 选择后地址变为 /zh-CN/leaderboard、cookie 为 zh-CN、导航变中文", async ({
  page,
  context,
}) => {
  await page.goto("/leaderboard");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");

  await switchLocale(page, "Language", "简体中文");

  await expect(page).toHaveURL(/\/zh-CN\/leaderboard$/);
  await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");
  const cookies = await context.cookies();
  expect(cookies.find((cookie) => cookie.name === "tt_locale")?.value).toBe(
    "zh-CN"
  );

  const nav = mainNav(page, "主导航");
  await expect(nav.getByRole("link", { name: "排行榜" })).toBeVisible();
  await expect(nav.getByRole("link", { name: "团队榜" })).toBeVisible();
  await expect(nav.getByRole("link", { name: "文档" })).toBeVisible();
});

test("切换到 English: 中文前缀下选择 English 后回到无前缀地址、导航变英文", async ({
  page,
}) => {
  await page.goto("/zh-CN/leaderboard");
  await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");

  await switchLocale(page, "语言", "English");

  await expect(page).toHaveURL(/\/leaderboard$/);
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  const nav = mainNav(page);
  await expect(nav.getByRole("link", { name: "Leaderboard" })).toBeVisible();
  await expect(nav.getByRole("link", { name: "Docs" })).toBeVisible();
  await expect(nav.getByRole("link", { name: "Teamboard" })).toBeVisible();
});

test("从 / 回到记忆语言: 切换为简体中文后打开 / 重定向到 /zh-CN/leaderboard", async ({
  page,
}) => {
  await page.goto("/leaderboard");
  await switchLocale(page, "Language", "简体中文");
  await expect(page).toHaveURL(/\/zh-CN\/leaderboard$/);

  await page.goto("/");
  await expect(page).toHaveURL(/\/zh-CN\/leaderboard$/);
  await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");
});

test("旧版 tt_locale=zh cookie: 打开 / 重定向到 /zh-CN/leaderboard", async ({
  page,
  context,
}) => {
  await context.addCookies([
    { name: "tt_locale", value: "zh", url: BASE_URL },
  ]);
  const response = await page.goto("/");
  // The middleware answers with a 307 to the zh-CN board before any render.
  expect(response?.request().redirectedFrom()?.url()).toBe(`${BASE_URL}/`);
  await expect(page).toHaveURL(/\/zh-CN\/leaderboard$/);
  await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");
});

test("旧无前缀地址正常打开: /leaderboard 不 404、按 English 渲染", async ({
  page,
  context,
}) => {
  // Pre-cutover Chinese links carried no prefix; they must still answer.
  // Unprefixed is English by design — the prefix, not the cookie, decides.
  await context.addCookies([
    { name: "tt_locale", value: "zh", url: BASE_URL },
  ]);
  const response = await page.goto("/leaderboard");
  expect(response?.status()).toBe(200);
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
});

test("带前缀页面文案跟随: /zh-CN 渲染公共页面标题和主要文案", async ({ page }) => {
  const pages = [
    {
      path: "/zh-CN/leaderboard",
      heading: "排行榜",
      copy: "AI 编程 token 用量，由 Tokens CLI 上报。",
    },
    {
      path: "/zh-CN/teamboard",
      heading: "团队榜",
      copy: "查看单个团队内的 token 用量排名，可按分组筛选。",
    },
    {
      path: "/zh-CN/teams",
      heading: "团队",
      copy: "登录后可创建团队，或管理你所在的团队。",
    },
    {
      path: "/zh-CN/docs",
      heading: "文档",
      copy: "安装 CLI",
    },
    {
      path: "/zh-CN/local",
      heading: "本地查看器",
      copy: "在本地查看 token 用量数据，无需提交",
    },
    {
      path: "/zh-CN/login",
      heading: "登录",
      copy: "使用你注册时的邮箱和密码。",
    },
    {
      path: "/zh-CN/register",
      heading: "创建账号",
      copy: "用邮箱和密码注册。",
    },
    {
      path: "/zh-CN/forgot-password",
      heading: "重置密码",
      copy: "输入账号邮箱，我们会发送重置链接。",
    },
    {
      path: "/zh-CN/reset-password",
      heading: "选择新密码",
    },
    {
      path: "/zh-CN/verify-email",
      heading: "验证邮箱",
      copy: "此验证链接缺少 token。",
    },
  ];

  for (const { path, heading, copy } of pages) {
    await page.goto(path);
    await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");
    await expect(page.getByRole("heading", { name: heading })).toBeVisible();
    if (copy) {
      await expect(page.getByText(copy, { exact: true })).toBeVisible();
    }
    if (path === "/zh-CN/reset-password") {
      await expect(
        page.getByText("此重置链接缺少 token", { exact: false })
      ).toBeVisible();
    }
  }
});

test("设置页未登录重定向保留前缀: /zh-CN/settings → /login?returnTo=/zh-CN/settings", async ({
  page,
}) => {
  await page.goto("/zh-CN/settings");
  await expect(page).toHaveURL(
    `/login?returnTo=${encodeURIComponent("/zh-CN/settings")}`
  );
});

test("隐私与条款页面: 中文与日文翻译均标注英文版本具有最终效力", async ({ page }) => {
  await page.goto("/zh-CN/privacy");
  await expect(page.getByText("以英文版", { exact: false })).toBeVisible();

  await page.goto("/zh-CN/terms");
  await expect(page.getByText("以英文版", { exact: false })).toBeVisible();

  await page.goto("/ja/terms");
  await expect(page.getByText("英語版", { exact: false })).toBeVisible();
});

test("中文界面登录失败: 在 /zh-CN/login 用错误密码提交显示中文错误", async ({
  page,
  baseURL,
}) => {
  const user = fixtureUser("i18nloginfail", "a");
  await registerUser(page, user);

  await page.goto("/zh-CN/login");
  await page.getByLabel("邮箱").fill(user.email);
  await page.getByLabel("密码").fill("WrongPass!234");
  await page.getByRole("button", { name: "登录" }).click();

  await expect(page.getByText("邮箱或密码不正确")).toBeVisible();

  await deleteFixtureUsers([user.username]);
});
