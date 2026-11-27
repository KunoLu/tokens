/// <reference types="bun-types" />
import { afterAll, afterEach, expect, mock, test } from "bun:test";
import { GlobalRegistrator } from "@happy-dom/global-registrator";
import { createTranslator } from "@/lib/i18n/translate";
import { resolveMessages } from "@/lib/i18n/messages";
// DOM globals must exist before react-dom or the component module load, so
// the imports below are a deliberate dynamic import: static imports would
// hoist above this registration.
if (!GlobalRegistrator.isRegistered) {
  GlobalRegistrator.register();
}
(globalThis as Record<string, unknown>).IS_REACT_ACT_ENVIRONMENT = true;

const t = createTranslator("en", resolveMessages("en"));
mock.module("@/components/layout/LocaleProvider", () => ({
  useLocale: () => "en",
  useT: () => t,
}));

const { ProfileModels } = await import("./ProfileModels");
const { createRoot } = await import("react-dom/client");
const { act } = await import("react");
const Tooltip = await import("@radix-ui/react-tooltip");

const roots: Array<{ root: { unmount: () => void }; container: HTMLElement }> = [];

afterEach(() => {
  while (roots.length > 0) {
    const { root, container } = roots.pop()!;
    act(() => {
      root.unmount();
    });
    container.remove();
  }
});

afterAll(() => {
  (globalThis as Record<string, unknown>).IS_REACT_ACT_ENVIRONMENT = false;
  if (GlobalRegistrator.isRegistered) {
    GlobalRegistrator.unregister();
  }
});

test("60 of 100 model tokens render as 60%, with no unknown speed row", async () => {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => {
    root.render(
      <Tooltip.Provider>
        <ProfileModels
          models={["opus", "sonnet"]}
          modelUsage={[
            {
              model: "opus",
              tokens: 100,
              cost: 2,
              percentage: 40,
              speedUsage: [{ speed: "fast", source: "reported", tokens: 60, messages: 2 }],
            },
            { model: "sonnet", tokens: 50, cost: 1, percentage: 20 },
          ]}
        />
      </Tooltip.Provider>,
    );
  });
  roots.push({ root, container });

  const rows = [...container.querySelectorAll("tbody tr")];
  expect(rows).toHaveLength(2);
  const known = rows[0].textContent ?? "";
  const bare = rows[1].textContent ?? "";
  // Cost share is 40.0%. The speed line must stay 60% of model tokens.
  expect(known).toContain("60 tokens · 60%");
  expect(known).toContain("100");
  expect(known).not.toContain("100%");
  expect(known).not.toContain("message");
  expect(bare).not.toMatch(/Standard|Fast|Ultrafast/);
  expect(container.textContent).not.toMatch(/Unknown|未知|不明/);
});
