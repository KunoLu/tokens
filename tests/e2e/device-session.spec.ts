import { expect, test } from "@playwright/test";

const SESSION = "**/api/auth/session";

test("device session: failed load is not signed-out; retry recovers to sign-in", async ({
  page,
}) => {
  // Navigation also hits GET /api/auth/session. Fail every session request
  // until retry, or the first 500 is consumed and DeviceClient looks signed-out.
  let failSession = true;
  await page.route(SESSION, async (route) => {
    if (failSession) {
      await route.fulfill({
        status: 500,
        contentType: "application/json",
        body: JSON.stringify({ error: "fail" }),
      });
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ user: null }),
    });
  });

  await page.goto("/device");

  await expect(page.getByRole("heading", { name: "Authorize CLI" })).toBeVisible();
  await expect(
    page.locator('[role="alert"]:not(#__next-route-announcer__)'),
  ).toContainText("connection problem, not a sign-out");
  await expect(page.getByRole("button", { name: "Try again" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Sign in" })).toHaveCount(0);

  failSession = false;
  await page.getByRole("button", { name: "Try again" }).click();

  await expect(
    page.getByText("Sign in to authorize the CLI.", { exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Sign in" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Try again" })).toHaveCount(0);
});
