---
schema_version: 1
id: 20261013-sync-upstream-v2713
workflow_mode: strict
mode_source: user
mode_note: 用户明确要求 strict；确认方案为三 CLI 修复、图表体积修复，以及按本仓入口重做私密模式。
status: "checking"
branch: sync/upstream-20261013
created_at: '2026-10-13T12:10:20.800774+08:00'
updated_at: "2026-10-13T12:53:42.567752+08:00"
completed_at: null
---
# 上游 v27.1.3 同步

## 目标

在保留本仓邮箱认证、Team/Group、Teamboard、i18n、自建部署和版本 `1.0.1` 的前提下，拣入 `v27.1.1` 之后到 `v27.1.3` 的解析器与图表修复，并按本仓入口重做私密模式。

## 已确认范围

- 拣入 `b728ee27`、`392053ba`、`651e7573`、`4e3d4929`。
- 不 cherry-pick `16724c87`。私密模式迁移为 `0027`。
- 公开榜与公开 Teamboard 排除私密账号；团队管理名单保留成员。
- 跳过 release、merge、Nowdex、空状态图标、OG。
- 确认后提 PR，admin merge，删除开发分支，并 rsync 本地 Compose。

## 进度

- 基线已核对：`upstream/main` 等于 `v27.1.1` `7e8830f6`；`v27.1.3` 只在 tag 上。共同祖先仍是 `f114057f`。CLI 两点 diff 仅版本号。
- 用户已确认按此方案执行。

## 门禁

### Legacy Change Safety Review

Status: characterized
Behavior to change: Pi 代理模型归因、Claude 1 小时缓存写入计价、可选模型前缀剥离、用量图路径体积、私密账号的公开可见性。
Behavior to preserve: 邮箱登录、封禁页、Team/Group 列、团队管理名单、版本 1.0.1、公开账号默认仍上榜、提交令牌不能把账号改回公开。
Current reproduction evidence: 上次同步后 CLI 与 `v27.1.1` 源码除版本外一致；现有 `cargo test` 覆盖解析器。本轮在首次生产编辑前复跑该套件作为当前树证据。
Safety net: 现有 Rust 测试随拣入提交一起落地；私密模式用 `web/features/private-mode.feature` 与针对本地库的不变量脚本，在改查询前先红。
Hidden dependencies / seam: 榜单与 Teamboard 共用 `leaderboard` cache tag；资料页缓存不能区分私密与不存在。
Validation plan: `cargo test --manifest-path cli/Cargo.toml --workspace`；`bun test` 图表路径；`bun run test:migrations` 与私密不变量脚本打 `127.0.0.1:5433`。
Review mode: normal

### Refactoring Review

Status: proceed
Review mode: normal
Existing-code scope: `cli/` 解析器与 `web/` 榜单、资料、设置、Teamboard。
Behavior that must remain unchanged: 见上。
Structural friction: `16724c87` 把迁移号、英文设置页和本仓分歧文件捆在一起。`getLeaderboard` 与 `getTeamboard` 多处重复 `isNull(users.bannedAt)`。
Decision and smallest safe step: 不先重构。CLI 用 cherry-pick。私密过滤抽一个 `publicUserCondition()`，只替换本次要改的公开查询，不顺手重写榜单。
Safety net and validation: 同上。
Deferred refactors: 不把所有 `bannedAt` 调用点收成一个模块以外的清理。

### DDIA Data Design Review

Status: confirmed
Data owner and source of truth: `users.is_private` 与 `read_tokens.token_hash` 在本仓 Postgres。读令牌明文只在签发响应出现一次。CLI 消息缓存格式 8 只在本机，不是网站库。
Write / read / async / failure paths: `POST /api/me/private` 在一个事务里打开私密并签发读令牌。设置页 PATCH 用浏览器会话改回公开。公开读路径把私密账号当成不存在。缓存失效沿用账号删除的 `leaderboard` tag。
Consistency model: 开关与令牌签发是同一事务的强一致。公开缓存最多 60 秒，开关后主动 `revalidateTag`。
Idempotency / ordering / retry / deduplication: 重复 `login --private` 保持私密并再发一个读令牌。读令牌与 `tt_` 提交令牌前缀不同，不能互认。
Schema / migration / backfill / rollback / replay: `0027` 只加列和表，默认 `is_private=false`，不改既有行。回滚是停用功能后的逆向 DDL，不在本次自动执行。CLI cache 由上游格式 7 到 8 迁移，失败时重解析活日志。
Observability and repair: 迁移由 `db:migrate` 与 `check-migrations` 证明。私密不变量脚本证明公开查询排除私密行。
Required tests: 迁移检查、私密不变量、CLI 随补丁测试、图表 `bun:test`。

## 下一步

拣入四个 SHA，然后实现 `0027` 与本仓入口。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-10-13T12:10:27.758320+08:00 | planned | in-progress | 用户确认方案，开始在 sync/upstream-20261013 上实现 | ai/tasks/20261013-sync-upstream-v2713/task.md |
| 2026-10-13T12:53:42.567752+08:00 | in-progress | checking | CLI、图表和私密模式检查已通过，准备提交并合入 | cargo test 2432 passed; bun test usageChartPaths 7 passed; bun run typecheck pass; DATABASE_URL local test:migrations and test:private pass |
