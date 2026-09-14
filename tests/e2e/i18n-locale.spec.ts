import { expect, test, type Page } from "@playwright/test";

// Traces the LocaleToggle scenarios in web/features/i18n.feature (T10).
// Scenario names are kept in the test titles for traceability.

const BASE_URL = "http://localhost:3000";

function mainNav(page: Page) {
  return page.getByRole("navigation", { name: "Main navigation" });
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

  const nav = mainNav(page);
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
      mainNav(reopenedPage).getByRole("link", { name: "排行榜" })
    ).toBeVisible();
  } finally {
    await reopened.close();
  }
});
