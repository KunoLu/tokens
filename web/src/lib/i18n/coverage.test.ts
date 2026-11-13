import { describe, expect, test } from "bun:test";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { resolveMessages } from "./messages";

/**
 * `TranslationKey` is a plain string on purpose (see translate.ts), so a typo'd
 * key compiles fine and renders the key itself to a user. This test is the net
 * for that: every literal key handed to `t()` (or carried in a `*Keys` table)
 * must resolve to a string in the English catalogue.
 *
 * And the fork namespaces (auth / teams / teamboard, plus the fork-owned docs
 * and legal) ship complete zh-CN and ja translations, so for those the
 * fall back to English is a bug, not a grace period: every leaf in en must
 * exist in all three locales there.
 */

const SRC = join(import.meta.dir, "../..");

function* walk(dir: string): Generator<string> {
  for (const entry of readdirSync(dir)) {
    if (entry === "node_modules" || entry.startsWith(".")) continue;
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) yield* walk(full);
    else if (/\.(ts|tsx)$/.test(entry)) yield full;
  }
}

const KEY_PATTERN = /["'`]((?:nav|common|leaderboard|teamboard|profile|settings|meta|footer|errors|docs|legal|auth|teams)\.[a-zA-Z0-9]+(?:\.[a-zA-Z0-9]+)*)["'`]/g;

function collectKeys(): Set<string> {
  const keys = new Set<string>();
  for (const file of walk(SRC)) {
    const rel = relative(SRC, file);
    if (rel.startsWith("lib/i18n/")) continue;
    if (rel.includes(".test.")) continue;
    for (const match of readFileSync(file, "utf8").matchAll(KEY_PATTERN)) {
      keys.add(match[1]);
    }
  }
  return keys;
}

function readPath(messages: unknown, key: string): unknown {
  let node = messages;
  for (const segment of key.split(".")) {
    if (node === null || typeof node !== "object") return undefined;
    node = (node as Record<string, unknown>)[segment];
  }
  return node;
}

describe("catalogue coverage", () => {
  const keys = collectKeys();

  test("every referenced key exists in the source catalogue", () => {
    const en = resolveMessages("en");
    const missing = [...keys].filter((key) => typeof readPath(en, key) !== "string");
    expect(missing).toEqual([]);
  });

  test("fork namespaces are fully translated (no silent English fallback)", () => {
    const zh = resolveMessages("zh-CN");
    const ja = resolveMessages("ja");
    const forkKeys = [...keys].filter((key) =>
      ["auth", "teams", "teamboard", "docs", "legal"].includes(key.split(".")[0]),
    );
    expect(forkKeys.length).toBeGreaterThan(100);
    for (const key of forkKeys) {
      expect(typeof readPath(zh, key), `zh-CN missing ${key}`).toBe("string");
      expect(typeof readPath(ja, key), `ja missing ${key}`).toBe("string");
    }
  });
});
