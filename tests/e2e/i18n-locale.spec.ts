import { expect, test, type Page } from "@playwright/test";
import { deleteFixtureUsers, fixtureUser, registerUser } from "./users";

// Traces the LocaleToggle scenarios in web/features/i18n.feature (T10/T11).
// Scenario names are kept in the test titles for traceability.

const BASE_URL = "http://localhost:3000";

function mainNav(page: Page, name = "Main navigation") {
  return page.getByRole("navigation", { name });
}

async function switchLocale(page: Page, buttonName: string, optionName: string) {
  await page.getByRole("button", { name: buttonName }).click();
  // Clicking an option writes tt_locale and reloads the page.
  await page.getByRole("menuitem", { name: optionName }).click();
}

test("按钮位置与选项: 语言按钮在主题切换左侧，选项恰好为 English / 中文", async ({
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
  await expect(items).toHaveCount(2);
  await expect(items.nth(0)).toHaveText("English");
  await expect(items.nth(1)).toHaveText("中文");
});

test("切换到中文: 选择中文后 lang=zh-CN、cookie tt_locale=zh、导航变中文", async ({
  page,
  context,
}) => {
  await page.goto("/leaderboard");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");

  await switchLocale(page, "Language", "中文");

  await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");
  const cookies = await context.cookies();
  expect(cookies.find((cookie) => cookie.name === "tt_locale")?.value).toBe(
    "zh"
  );

  const nav = mainNav(page, "主导航");
  await expect(nav.getByRole("link", { name: "排行榜" })).toBeVisible();
  await expect(nav.getByRole("link", { name: "团队榜" })).toBeVisible();
  await expect(nav.getByRole("link", { name: "文档" })).toBeVisible();
});

test("切换到 English: 中文偏好下选择 English 后 lang=en、导航变英文", async ({
  page,
  context,
}) => {
  await context.addCookies([
    { name: "tt_locale", value: "zh", url: BASE_URL },
  ]);
  await page.goto("/leaderboard");
  await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");

  await switchLocale(page, "语言", "English");

  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  const nav = mainNav(page);
  await expect(nav.getByRole("link", { name: "Leaderboard" })).toBeVisible();
  await expect(nav.getByRole("link", { name: "Docs" })).toBeVisible();
  await expect(nav.getByRole("link", { name: "Teamboard" })).toBeVisible();
});

test("关闭浏览器后重新打开: 中文偏好随 cookie 跨浏览器会话保持", async ({
  page,
  context,
  browser,
}) => {
  await page.goto("/leaderboard");
  await switchLocale(page, "Language", "中文");
  await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");

  // A fresh context seeded with the persisted cookie stands in for closing
  // and reopening the browser (tt_locale has Max-Age=1y, so it survives).
  const state = await context.storageState();
  const reopened = await browser.newContext({ storageState: state });
  try {
    const reopenedPage = await reopened.newPage();
    await reopenedPage.goto("/leaderboard");
    await expect(reopenedPage.locator("html")).toHaveAttribute(
      "lang",
      "zh-CN"
    );
    await expect(
      mainNav(reopenedPage, "主导航").getByRole("link", { name: "排行榜" })
    ).toBeVisible();
  } finally {
    await reopened.close();
  }
});

test("切换后全部页面文案跟随: 中文偏好渲染公共页面标题和主要文案", async ({
  page,
  context,
}) => {
  await context.addCookies([
    { name: "tt_locale", value: "zh", url: BASE_URL },
  ]);

  const pages = [
    {
      path: "/leaderboard",
      heading: "排行榜",
      copy: "AI 编程 token 用量，由 Tokens CLI 上报。",
    },
    {
      path: "/teamboard",
      heading: "团队榜",
      copy: "查看单个团队内的 token 用量排名，可按分组筛选。",
    },
    {
      path: "/teams",
      heading: "团队",
      copy: "登录后可创建团队，或管理你所在的团队。",
    },
    {
      path: "/docs",
      heading: "文档",
      copy: "安装 CLI",
    },
    {
      path: "/local",
      heading: "本地查看器",
      copy: "在本地查看 token 用量数据，无需提交",
    },
    {
      path: "/login",
      heading: "登录",
      copy: "使用你注册时的邮箱和密码。",
    },
    {
      path: "/register",
      heading: "创建账号",
      copy: "用邮箱和密码注册。",
    },
    {
      path: "/forgot-password",
      heading: "重置密码",
      copy: "输入账号邮箱，我们会发送重置链接。",
    },
    {
      path: "/reset-password",
      heading: "选择新密码",
    },
    {
      path: "/verify-email",
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
    if (path === "/reset-password") {
      await expect(
        page.locator('[role="alert"]:not(#__next-route-announcer__)'),
      ).toContainText("此重置链接缺少 token");
    }
  }
});

test("隐私页面: 中文内容明确标注英文版本具有最终效力", async ({
  page,
  context,
}) => {
  await context.addCookies([
    { name: "tt_locale", value: "zh", url: BASE_URL },
  ]);

  await page.goto("/privacy");

  await expect(page.getByRole("heading", { name: "隐私政策" })).toBeVisible();
  await expect(
    page.getByText("本页内容以英文版本为准。", { exact: true }),
  ).toBeVisible();
});

test("条款页面: 中文内容明确标注英文版本具有最终效力", async ({
  page,
  context,
}) => {
  await context.addCookies([
    { name: "tt_locale", value: "zh", url: BASE_URL },
  ]);

  await page.goto("/terms");

  await expect(page.getByRole("heading", { name: "服务条款" })).toBeVisible();
  await expect(
    page.getByText("本页内容以英文版本为准。", { exact: true }),
  ).toBeVisible();
});

const settingsRunId = Date.now().toString(36);
const settingsUser = fixtureUser(settingsRunId, "a");

test.afterAll(async () => {
  await deleteFixtureUsers([settingsUser.username]);
});

test("已登录设置页: 中文偏好渲染设置标题和说明", async ({
  page,
  context,
}) => {
  test.setTimeout(180_000);
  await registerUser(page, settingsUser);
  await context.addCookies([
    { name: "tt_locale", value: "zh", url: BASE_URL },
  ]);
  await page.goto("/settings");
  await expect(page.locator("html")).toHaveAttribute("lang", "zh-CN");
  await expect(page.getByRole("heading", { name: "设置" })).toBeVisible({
    timeout: 60_000,
  });
  await expect(
    page.getByText("管理你的资料、API token、设备和已提交数据。", {
      exact: true,
    })
  ).toBeVisible();
});

test("中文界面登录失败: 错误密码提示邮箱或密码不正确", async ({
  page,
  context,
}) => {
  await context.addCookies([
    { name: "tt_locale", value: "zh", url: BASE_URL },
  ]);
  await page.goto("/login");
  await page.getByLabel("邮箱").fill("nobody@example.test");
  await page.getByLabel("密码").fill("WrongPass1!");
  await page.getByRole("button", { name: "登录" }).click();
  await expect(page.getByText("邮箱或密码不正确")).toBeVisible();
});

