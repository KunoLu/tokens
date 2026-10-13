---
schema_version: 1
id: 20261013-docs-private-mode
workflow_mode: strict
mode_source: user
mode_note: 用户明确要求 strict，并确认文档页只补私密模式、登录授权句和五条命令文案，不同步 brew services 或 bunx。
status: "done"
branch: docs/private-mode-section
created_at: '2026-10-13T13:48:36.117407+08:00'
updated_at: "2026-10-13T13:55:35.380119+08:00"
completed_at: "2026-10-13T13:55:35.380119+08:00"
---
# 文档页补私密模式说明

## 目标

在 `/docs` 补上已落地的私密模式用法，并修正与当前登录、上报流程不符的两句说明。不改各平台安装命令，不写回 `brew services` 或 `bunx`。

## 已确认范围

- 「日常使用」和「支持的客户端」之间加「私密模式」：命令、设置页、读令牌只读、设备名打码。中英文走字典。不写 tokens.ci、iOS、GitHub 登录。
- 「五条命令覆盖整个工作流程」改为上报流程，并指向下方私密模式。
- 安装说明下方加一句：先在本站注册并登录，再跑 `tokens login`，浏览器打开本站 `/device` 填授权码。
- 不把 `--private` 插入安装命令列表。

## 门禁

### Legacy Change Safety Review

Status: characterized
Behavior to change: 文档页补私密模式，并改两句与现登录/上报流程不符的文案。
Behavior to preserve: 安装命令顺序、预安装脚本、无 brew services、无 bunx tokens-cli、无 tokens.ci、macOS serve 说明含本站地址和 CLI 手册链接。
Current reproduction evidence: `tests/e2e/t9-acceptance.spec.ts` 与 `web/features/docs-page.feature` 锁定上述安装约束。页面源码与正在跑的 `/docs` 已核对。
Safety net: 现有 docs e2e 约束；新文案不插入安装命令列表，避免 `indexOf("tokens login")` 被提前命中。
Hidden dependencies / seam: 无。
Validation plan: `bun run typecheck`；对照 e2e 断言检查生成文案；本地 Compose curl `/docs`。
Review mode: normal

### Refactoring Review

Status: proceed
Review mode: normal
Existing-code scope: `web/src/app/(main)/docs/page.tsx` 与 `web/src/lib/i18n/t.ts`。
Behavior that must remain unchanged: 安装命令与平台说明。
Structural friction: 无。文案已在字典里。
Decision and smallest safe step: 不重构。加字典键和一节。
Safety net and validation: 同上。
Deferred refactors: 无。

### DDIA Data Design Review

Status: not-required。不改库、缓存或提交数据。

## 下一步

先改 feature，再改页面。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-10-13T13:48:36.609056+08:00 | planned | in-progress | 用户确认文档页调整范围，开始在 docs/private-mode-section 上实现 | 对话确认的三处文案调整 |
| 2026-10-13T13:55:09.072224+08:00 | in-progress | checking | 文档页文案已改，typecheck 与本地 /docs 中英文渲染已核对 | bun run typecheck pass; curl /docs contains Private mode and 私密模式; no tokens.ci, brew services, or bunx tokens-cli |
| 2026-10-13T13:55:35.380119+08:00 | checking | done | 文档页已按确认范围改完，本地 /docs 中英文渲染核对通过，准备 admin merge | commit 7dfcfe61; curl /docs English and Chinese; Playwright browser binary missing so spec not run |
