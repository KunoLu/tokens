/// <reference types="bun-types" />
import { expect, test } from "bun:test";
import { publicDeviceDisplayLabel } from "./shared";

const cloudKey = "cloud_claude_0123456789abcdef0123456789abcdef";

// Trace: cloud-sessions.feature — 云端设备自定义名不能替换固定标签。
test("public cloud label ignores a custom name and remains unmasked when unnamed", () => {
  expect(publicDeviceDisplayLabel(cloudKey, "my laptop")).toBe("Claude Code cloud");
  expect(publicDeviceDisplayLabel(cloudKey, null)).toBe("Claude Code cloud");
});

test("public non-cloud device names retain the existing privacy mask", () => {
  expect(publicDeviceDisplayLabel("dev_laptop", "my laptop")).toBe("my****op");
  expect(publicDeviceDisplayLabel("dev_short", "猫咪")).toBe("猫***");
  expect(publicDeviceDisplayLabel("dev_unnamed", null)).toBe("Unnamed device");
});
