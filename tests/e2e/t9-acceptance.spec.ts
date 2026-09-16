import { expect, test } from "@playwright/test";
import { banFixtureUser, deleteFixtureUsers, fixtureUser, registerUser } from "./users";

test("/shame is 404 and Hall of Shame is gone from the nav", async ({ page }) => {
  const response = await page.goto("/shame");
  expect(response?.status()).toBe(404);
  await expect(page.getByRole("link", { name: "Hall of Shame" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Teamboard" })).toBeVisible();
});

test("Leaderboard column order is # Developer Team Group Tokens Cost", async ({
  page,
}) => {
  await page.goto("/leaderboard");
  // Desktop Chrome (≥ sm): Usage is sm:hidden; Tokens/Cost are sm:table-cell.
  // Assert visible headers only — do not drop the mobile cell from the DOM.
  await expect(page.locator("thead th:visible")).toHaveText([
    "#",
    "Developer",
    "Team",
    "Group",
    "Tokens",
    "Cost",
  ]);
});

test("Docs keeps Install / Everyday / Supported clients and drops removed sections", async ({
  page,
}) => {
  const response = await page.goto("/docs");
  expect(response?.status()).toBe(200);
  await expect(page.getByRole("heading", { name: "Install the CLI" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Everyday use" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Supported clients" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "iOS app" })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Architecture" })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Sponsors" })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "The verified badge" })).toHaveCount(0);
});

test("Privacy, Terms, Settings, embed and badge still respond", async ({ page }) => {
  const privacy = await page.goto("/privacy");
  expect(privacy?.status()).toBe(200);
  await expect(page.getByRole("heading", { name: "Privacy Policy" })).toBeVisible();
  const terms = await page.goto("/terms");
  expect(terms?.status()).toBe(200);
  await expect(page.getByRole("heading", { name: "Terms of Service" })).toBeVisible();
  const settings = await page.goto("/settings");
  expect(settings?.status()).toBe(200);
  await expect(page).toHaveURL(/\/login/);
  await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
  const embed = await page.goto("/api/embed/tokens/svg");
  expect(embed?.status()).toBe(200);
  expect(embed?.headers()["content-type"] ?? "").toContain("image/svg+xml");
  const badge = await page.goto("/api/badge/tokens/svg");
  expect(badge?.status()).toBe(200);
  expect(badge?.headers()["content-type"] ?? "").toContain("image/svg+xml");
  const archive = await page.request.post("/api/archive", { data: {} });
  expect(archive.status()).toBe(401);
  expect(await archive.json()).toEqual({ error: "Not authenticated" });
});


test("banned profile still renders and login is 403", async ({ page }) => {
  const user = fixtureUser(`t9${Date.now().toString(36)}`, "a");
  try {
    await registerUser(page, user);
    await banFixtureUser(user.username, "t9 e2e ban");

    const login = await page.request.post("/api/auth/login", {
      headers: { Origin: "http://localhost:3000" },
      data: { email: user.email, password: user.password },
    });
    expect(login.status()).toBe(403);
    expect(await login.json()).toEqual({ error: "Account banned" });

    await page.goto(`/u/${user.username}`);
    await expect(page.getByText("Account banned", { exact: false })).toBeVisible();
    await expect(page.getByText("Banned", { exact: true })).toBeVisible();
  } finally {
    await deleteFixtureUsers([user.username]);
  }
});
