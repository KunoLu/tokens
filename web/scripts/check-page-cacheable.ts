/**
 * Worker-path evidence for PAGE_CACHEABLE. `next dev` never hits
 * `caches.default`; this imports the same matcher `worker.ts` uses.
 */
import {
  PAGE_CACHEABLE,
  workerSharesSignedOutHtml,
} from "../src/lib/cache/pageCacheable";

function expect(name: string, condition: boolean) {
  if (!condition) throw new Error(name);
  console.log(`ok - ${name}`);
}

expect("PAGE_CACHEABLE matches /", PAGE_CACHEABLE.test("/"));
expect("PAGE_CACHEABLE matches /leaderboard", PAGE_CACHEABLE.test("/leaderboard"));
expect(
  "PAGE_CACHEABLE does not match /teamboard",
  PAGE_CACHEABLE.test("/teamboard") === false
);
expect(
  "PAGE_CACHEABLE does not match /teamboard/",
  PAGE_CACHEABLE.test("/teamboard/") === false
);
expect(
  "signed-out /leaderboard is shared-cached",
  workerSharesSignedOutHtml("/leaderboard", false) === true
);
expect(
  "signed-in /leaderboard is not shared-cached",
  workerSharesSignedOutHtml("/leaderboard", true) === false
);
expect(
  "signed-out /teamboard is not shared-cached",
  workerSharesSignedOutHtml("/teamboard", false) === false
);
expect(
  "signed-in /teamboard is not shared-cached",
  workerSharesSignedOutHtml("/teamboard", true) === false
);
expect(
  "signed-out /teamboard?team=x pathname is not shared-cached",
  workerSharesSignedOutHtml(new URL("https://tokens.ci/teamboard?team=x").pathname, false) ===
    false
);

console.log("ok - worker page-cacheable characterization complete");
