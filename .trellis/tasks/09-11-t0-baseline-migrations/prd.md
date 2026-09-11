# T0 基线修复

> 父任务：`.trellis/tasks/09-10-teamboard-teams-auth/`
> 推进清单：`docs/TODO.md` T0 节
> 完整方案：`docs/prd-teamboard-teams-auth.md` §2.6

## Goal

让 `bun run test:migrations` 与现实 schema 一致：去掉对已删除 group 表的断言，并把 snapshot 补到 journal 尾部 idx 23。否则后续 `db:generate` 会产出错误 diff。

## Depends on

无

## Blocks

T1、T2、T3、T10、T12

## Requirements

- 移除 `web/scripts/check-migrations.ts` 对 `groups` / `group_members` / `group_invites` 的断言
- 补齐 `web/src/lib/db/migrations/meta/` 的 `0022` / `0023` snapshot，使最新 snapshot 编号等于 journal 尾部 idx（23）

## Out of scope

父任务其余全部范围。不改 schema、不加迁移、不改生产运行路径。

## Acceptance Criteria

- [x] `bun run test:migrations` 通过
- [x] 脚本不再断言已删除的 group 表存在
- [x] 最新 snapshot 编号等于 journal 尾部 idx（23）
