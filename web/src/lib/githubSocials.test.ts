/// <reference types="bun-types" />
import { beforeEach, expect, mock, test } from "bun:test";
import * as schema from "./db/schema";

// The update chain records what persistSocialLinks would write. Any call at
// all is a write: the regression being pinned here is a failed GitHub fetch
// overwriting a stored snapshot with an empty list.
const writes: Array<{ socialLinks: unknown }> = [];
mock.module("@/lib/db", () => ({
  ...schema,
  db: {
    update: () => ({
      set: (values: { socialLinks: unknown }) => ({
        where: () => {
          writes.push(values);
          return Promise.resolve();
        },
      }),
    }),
  },
}));

// Dynamic import on purpose: the `@/lib/db` mock above must register before
// the module under test loads, and a static import would hoist past it.
const { syncGitHubSocialLinks, syncGitHubSocialLinksStrict } = await import(
  "./githubSocials"
);

type FetchStub = (url: string) => { status: number; body?: unknown };

function stubFetch(stub: FetchStub) {
  globalThis.fetch = mock((input: unknown) => {
    const url = String(input);
    const { status, body } = stub(url);
    return Promise.resolve(
      new Response(body === undefined ? null : JSON.stringify(body), { status }),
    );
  }) as unknown as typeof fetch;
}

function stubFetchRejects() {
  globalThis.fetch = mock(() =>
    Promise.reject(new Error("timed out")),
  ) as unknown as typeof fetch;
}

const PROFILE_URL = "https://api.github.com/users/octocat";
const SOCIAL_URL = `${PROFILE_URL}/social_accounts`;

beforeEach(() => {
  writes.length = 0;
});

test("a GitHub rate limit on both calls writes nothing", async () => {
  stubFetch(() => ({ status: 403 }));

  const links = await syncGitHubSocialLinks("octocat");

  expect(links).toEqual([]);
  expect(writes).toHaveLength(0);
});

test("a half-failed pair is partial: the stored snapshot stays untouched", async () => {
  stubFetch((url) =>
    url === SOCIAL_URL
      ? { status: 500 }
      : { status: 200, body: { blog: "https://example.com/about" } },
  );

  const links = await syncGitHubSocialLinks("octocat");

  expect(links).toEqual([{ provider: "website", url: "https://example.com/about" }]);
  expect(writes).toHaveLength(0);
});

test("a transport failure writes nothing", async () => {
  stubFetchRejects();

  const links = await syncGitHubSocialLinks("octocat");

  expect(links).toEqual([]);
  expect(writes).toHaveLength(0);
});

test("404 means no GitHub profile: a complete, empty snapshot that persists", async () => {
  stubFetch(() => ({ status: 404 }));

  const links = await syncGitHubSocialLinks("octocat");

  expect(links).toEqual([]);
  expect(writes).toHaveLength(1);
  expect(writes[0].socialLinks).toEqual([]);
});

test("both calls ok persist the combined links", async () => {
  stubFetch((url) =>
    url === SOCIAL_URL
      ? { status: 200, body: [{ provider: "twitter", url: "https://x.com/octocat" }] }
      : { status: 200, body: { blog: "example.com/about" } },
  );

  const links = await syncGitHubSocialLinks("octocat");

  expect(links).toEqual([
    { provider: "website", url: "https://example.com/about" },
    { provider: "twitter", url: "https://x.com/octocat" },
  ]);
  expect(writes).toHaveLength(1);
  expect(writes[0].socialLinks).toEqual(links);
});

test("the strict cron variant throws instead of writing a partial snapshot", async () => {
  stubFetch((url) =>
    url === SOCIAL_URL ? { status: 403 } : { status: 200, body: { blog: "" } },
  );

  await expect(syncGitHubSocialLinksStrict("octocat")).rejects.toThrow(
    "incomplete",
  );
  expect(writes).toHaveLength(0);
});
