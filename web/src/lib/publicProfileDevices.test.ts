/// <reference types="bun-types" />
import { expect, mock, test } from "bun:test";
import { fileURLToPath } from "node:url";
import { PgDialect } from "drizzle-orm/pg-core";
import type { SQL } from "drizzle-orm";
import * as schema from "./db/schema";

// Run module mocks in a child Bun process: mock.module replacements otherwise
// leak into publicProfileData.test.ts and other consumers of the same modules.
// Trace: cloud-sessions.feature — 私密 404 关口与所有者私密缓存策略。
async function exercisePrivateDevices(owner: boolean) {
  const events: string[] = [];
  const cloudKey = "cloud_claude_0123456789abcdef0123456789abcdef";
  const user = {
    id: "private-user",
    username: "alice",
    displayName: null,
    avatarUrl: null,
    bannedAt: null,
    get isPrivate() {
      events.push("visibilityHeaders");
      return true;
    },
  };
  const device = {
    id: "session-device", deviceKey: cloudKey, displayName: "my laptop",
    createdAt: "2026-10-01T00:00:00.000Z", lastSubmittedAt: null,
    totalTokens: "100", totalCost: "1", inputTokens: "80", outputTokens: "20",
    activeDays: "2", firstDay: "2026-10-01", lastDay: "2026-10-03",
  };
  const cloudQuery = mock(() => {
    events.push("cloudQuery");
    return [{ provider: "claude", activeDays: "3" }];
  });
  let cloudPredicate: SQL | undefined;
  function chain(rows: unknown[], isCloud: boolean) {
    const builder: Record<string, unknown> = {};
    for (const method of ["from", "limit", "leftJoin", "innerJoin", "groupBy", "orderBy"]) {
      builder[method] = () => builder;
    }
    builder.where = (predicate: SQL) => {
      if (isCloud) cloudPredicate = predicate;
      return builder;
    };
    builder.then = (resolve: (value: unknown) => unknown, reject: (error: unknown) => unknown) =>
      Promise.resolve(rows).then(resolve, reject);
    return builder;
  }
  mock.module("@/lib/db", () => ({
    ...schema,
    db: {
      select: (fields: Record<string, unknown>) => {
        if ("username" in fields) return chain([user], false);
        if ("provider" in fields) return chain(cloudQuery(), true);
        if ("deviceKey" in fields) {
          events.push("deviceQuery");
          return chain([device], false);
        }
        throw new Error("Unexpected device query");
      },
    },
  }));
  // Delayed imports are required here: install DB mocks before privacy loads,
  // then install access/cloud spies before the handler binds its dependencies.
  const privacy = await import("./privacy");
  const getViewerAccess = mock(async () => {
    events.push("getViewerAccess");
    return {
      get allowed() {
        events.push("accessAllowed");
        return owner;
      },
      isOwner: owner,
    };
  });
  mock.module("@/lib/privacy", () => ({ ...privacy, getViewerAccess }));
  const cloud = await import("./devices/cloud");
  // Capture the implementation before replacing its live module export.
  const parse = cloud.parseCloudDeviceKey;
  const parseCloudDeviceKey = mock((key: string) => {
    events.push("cloudLogic");
    return parse(key);
  });
  mock.module("@/lib/devices/cloud", () => ({ ...cloud, parseCloudDeviceKey }));
  const { getPublicProfileDevicesResponse } = await import("./publicProfileDevices");
  const response = await getPublicProfileDevicesResponse(
    new Request("http://tokens.internal/api/users/alice/devices"),
    { params: Promise.resolve({ username: "alice" }) },
  );
  const body = await response.json();
  expect(getViewerAccess).toHaveBeenCalledTimes(1);
  if (!owner) {
    expect(response.status).toBe(404);
    expect(body).toEqual({ error: "User not found" });
    expect(cloudQuery).not.toHaveBeenCalled();
    expect(parseCloudDeviceKey).not.toHaveBeenCalled();
    expect(events).toEqual(["getViewerAccess", "accessAllowed"]);
  } else {
    expect(response.status).toBe(200);
    expect(response.headers.get("Cache-Control")).toBe(privacy.PRIVATE_CACHE_CONTROL);
    // The isPrivate getter observes evaluation of the existing cache policy,
    // without adding a production seam or pretending response creation is earlier.
    expect(events.slice(0, 4)).toEqual([
      "getViewerAccess", "accessAllowed", "visibilityHeaders", "deviceQuery",
    ]);
    expect(events.indexOf("cloudLogic")).toBeGreaterThan(events.indexOf("visibilityHeaders"));
    expect(events.indexOf("cloudQuery")).toBeGreaterThan(events.indexOf("cloudLogic"));
    expect(cloudQuery).toHaveBeenCalledTimes(1);
    expect(body.devices).toEqual([{
      ...device, displayName: "Claude Code cloud", customName: null,
      totalTokens: 100, totalCost: 1, inputTokens: 80, outputTokens: 20,
      activeDays: 2, cloudGroupActiveDays: 3,
    }]);
    const query = new PgDialect().sqlToQuery(cloudPredicate!);
    expect(query.sql).toContain('"submitted_devices"."user_id" =');
    expect(query.params).toEqual(["private-user", cloud.CLOUD_DEVICE_KEY_PATTERN]);
  }
  console.log(JSON.stringify({ status: response.status, events }));
}

if (process.argv[2] === "--private-devices-fixture") {
  await exercisePrivateDevices(process.argv[3] === "owner");
} else {
  for (const [scenario, name] of [
    ["visitor", "private profile denies a non-owner before any cloud query or processing"],
    ["owner", "private owner access and cache policy precede cloud processing and user-scoped queries"],
  ] as const) {
    test(name, () => {
      const result = Bun.spawnSync([
        process.execPath, fileURLToPath(import.meta.url), "--private-devices-fixture", scenario,
      ], { cwd: import.meta.dir });
      expect(result.exitCode, result.stderr.toString()).toBe(0);
      console.log(result.stdout.toString().trim());
    });
  }
}
