import { expect, test } from "@playwright/test";

// T5: /teams is member-only. Signed-out visitors get a login gate, never the
// management surface. Signed-in journeys live in teams-signed-in.spec.ts
// (synthetic t5e2e-* fixtures); this spec deliberately covers the gate only.

test("未登录访问 /teams: 显示登录门槛而非管理界面", async ({ page }) => {
  await page.goto("/teams");

  await expect(
    page.getByRole("heading", { name: "Teams", level: 1 })
  ).toBeVisible();
  await expect(
    page.locator("#main-content").getByRole("link", { name: "Sign in" })
  ).toHaveAttribute(
    "href",
    `/login?returnTo=${encodeURIComponent("/teams")}`
  );

  // Management chrome must not leak to signed-out visitors.
  await expect(
    page.getByRole("button", { name: "Invite members" })
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Create team" })
  ).toHaveCount(0);
});
