# 实施记录

## 落地顺序

1. `next.config.ts` 改为 phase 回调：仅 `PHASE_DEVELOPMENT_SERVER` / `PHASE_PRODUCTION_BUILD` 调用 `initOpenNextCloudflareForDev()`；`next start` 不再初始化 CF 绑定。
2. 活/死库隔离验证纯 `DATABASE_URL` 路径。
3. 删除 Workers 层并清引用（见 PRD R3–R6）。

## Gate Records

- **Legacy Change Safety Review**: `characterized`（复现证据：build 通过；next start 200；死库 5999 仍 200 证明 Hyperdrive 胜出）
- **Refactoring Review**: `proceed`（两步走：闸 → 验证 → 删除）
- **DDIA Data Design Review**: `confirmed`（SOT 不变；R2/DO/边缘缓存均为可重建派生缓存；无迁移/回填）。**修订（cron 可靠性 blocker）**：HTTP cron 不再 `void work` 后台跑——三个 sweep（refreshAllSocialLinks / deleteExpiredEmailTokens / expireInvitations）逐个 try/catch 后全部 await；任一失败返回 500 让调度器重试，全成功返回 200 + 计数。调度器走同机 loopback（system cron + curl），不经公网反代以避开 proxy 超时。

- **DDD / BDD**: 未触发——无领域歧义；无新增用户可见行为（运维拓扑变更）
- **Release Readiness**: 收尾前运行（部署敏感变更）

## 关键事实

- `process.env.NEXT_PHASE` 在 `next start` 加载配置时不可靠（实测第一版 `!== "phase-production-server"` 仍初始化了 workerd）。必须用 config 导出函数接收 `phase` 参数。
- `getCloudflareContext` 引用点：`db/index.ts`、`rateLimit.ts`、`email/send.ts`、`api/cron/refresh-social-links/route.ts`、`next.config.ts`。
- CI `ci.yml:174` 有 `wrangler types` 步骤，需删除（typecheck 变纯 `tsc --noEmit`）。
- `expireInvitations` 原来只在 `worker.ts` cron；并入 HTTP cron route 的 background chain。
