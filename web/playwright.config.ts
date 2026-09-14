import { defineConfig, devices } from "@playwright/test";

// E2E specs live at the repo root (tests/e2e) so they can span packages;
// this config only wires the web/ dev server and reporters to them.
export default defineConfig({
  testDir: "../tests/e2e",
  timeout: 60_000,
  expect: { timeout: 15_000 },
  reporter: [
    ["list"],
    [
      "html",
      {
        // Temp report dir; trellis-check promotes native reports itself.
        outputFolder: "../tests/e2e/reports/.playwright-html-current",
        open: "never",
      },
    ],
  ],
  use: {
    baseURL: "http://localhost:3000",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    // LocaleToggle is a client cookie + reload flow; plain `next dev` is the
    // surface under test, not the OpenNext preview worker. cwd defaults to
    // this config file's directory (web/).
    command: "bun run dev",
    url: "http://localhost:3000",
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
