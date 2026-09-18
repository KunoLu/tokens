import { expect, test } from "@playwright/test";

// Scenario: web/features/maintenance-cron.feature「未携带密钥被拒绝」。
// The 200 sweep path is intentionally not automated: refreshAllSocialLinks
// fetches GitHub for every user (no github_id filter) and rewrites social
// snapshots, which would make a formal test network-dependent and mutating.
// The feature file tracks that path as @todo.

test("定时维护接口：未携带密钥被拒绝", async ({ request }) => {
  const res = await request.post("/api/cron/refresh-social-links");
  expect(res.status()).toBe(401);
});
