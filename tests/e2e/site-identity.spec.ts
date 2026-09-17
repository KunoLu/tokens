import { expect, test } from "@playwright/test";

// Traces the scenarios in web/features/site-identity.feature: every
// user-facing brand string and link derives from the deployment origin
// (NEXT_PUBLIC_URL) and CONTACT_EMAIL, never a hardcoded upstream domain.

const BASE_URL = "http://localhost:3000";
const SITE_HOST = new URL(BASE_URL).host;

for (const path of ["/privacy", "/terms", "/docs", "/leaderboard"]) {
  test(`公共页面 HTML 无 tokens.ci 且品牌字样为本站 host: ${path}`, async ({
    page,
  }) => {
    const response = await page.goto(path);
    expect(response?.status()).toBe(200);
    const html = await page.content();
    expect(html).not.toContain("tokens.ci");
    // og:url (inherited from the root layout metadata) resolves against the
    // deployment origin, so the host shows up in every page's head.
    expect(html).toContain(SITE_HOST);
  });
}

test("法律页品牌链接指向本站并显示本站 host", async ({ page }) => {
  for (const path of ["/privacy", "/terms"]) {
    await page.goto(path);
    const brandLink = page.getByRole("link", { name: SITE_HOST }).first();
    await expect(brandLink).toBeVisible();
    await expect(brandLink).toHaveAttribute("href", BASE_URL);
  }
});

test("条款软件段落显示本站 host", async ({ page }) => {
  await page.goto("/terms");
  await expect(
    page.getByText(new RegExp(`hosted service at ${SITE_HOST.replace(/\./g, "\\.")}`)),
  ).toBeVisible();
});

test("OG 图响应不含 tokens.ci", async ({ request }) => {
  const response = await request.get("/api/og?title=Tokens&subtitle=Brand+sweep");
  expect(response.status()).toBe(200);
  const body = await response.body();
  expect(body.includes(Buffer.from("tokens.ci"))).toBe(false);
});

test("embed SVG 品牌字样为本站 host 且不含 tokens.ci", async ({ request }) => {
  // A valid-format but nonexistent username renders the error card, which
  // carries the same brand footer as the data cards and needs no DB fixture.
  const response = await request.get("/api/embed/no-such-user-site-identity/svg");
  expect(response.status()).toBe(200);
  const svg = await response.text();
  expect(svg).not.toContain("tokens.ci");
  expect(svg).toContain(SITE_HOST);
});

test("未配置 CONTACT_EMAIL 时法律页无 mailto 链接", async ({ page }) => {
  // The dev stack under test runs without CONTACT_EMAIL; the legal pages must
  // fall back to the site link instead of inventing an address.
  for (const path of ["/privacy", "/terms"]) {
    await page.goto(path);
    await expect(page.locator('a[href^="mailto:"]')).toHaveCount(0);
  }
});
