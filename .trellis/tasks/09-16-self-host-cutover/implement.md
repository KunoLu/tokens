# 实施记录

## 落地顺序

1. `next.config.ts` 改为 phase 回调：仅 `PHASE_DEVELOPMENT_SERVER` / `PHASE_PRODUCTION_BUILD` 调用 `initOpenNextCloudflareForDev()`；`next start` 不再初始化 CF 绑定（commit 8a84c3eb）。
2. 活/死库隔离验证纯 `DATABASE_URL` 路径。
3. 删除 Workers 层并清引用（PRD R3–R6）。

## Gate Records

- **Legacy Change Safety Review**: `characterized`（复现证据：build 通过；next start 200；死库 5999 仍 200 证明 Hyperdrive 胜出）
- **Refactoring Review**: `proceed`（两步走：闸 → 验证 → 删除）
- **DDIA Data Design Review**: `confirmed`（SOT 不变；R2/DO/边缘缓存均为可重建派生缓存；无迁移/回填）。**修订（cron 可靠性 blocker）**：HTTP cron 不再 `void work` 后台跑——三个 sweep（refreshAllSocialLinks / deleteExpiredEmailTokens / expireInvitations）逐个 catch 后全部 await；任一失败返回 500 让调度器重试，全成功返回 200 + 计数。调度器走同机 loopback（system cron + curl），不经公网反代以避开 proxy 超时。
- **DDD**: 未触发——无领域歧义
- **BDD**: `traceable`——cron 接口契约变更落了 `web/features/maintenance-cron.feature`（401 自动化；200/失败场景 @todo，因 refreshAllSocialLinks 对每个用户请求 GitHub 且改写共享库快照）。200 与失败路径已一次性手动实证（见下）。
- **Release Readiness**: `ready`（两项残余风险由 owner 于 2026-09-16 显式接受）

## 关键事实

- `process.env.NEXT_PHASE` 在 `next start` 加载配置时不可靠（实测第一版 `!== "phase-production-server"` 仍初始化了 workerd）。必须用 config 导出函数接收 `phase` 参数。
- `getCloudflareContext` 引用点：`db/index.ts`、`rateLimit.ts`、`email/send.ts`、`api/cron/refresh-social-links/route.ts`、`next.config.ts`，已全部清除。
- CI `ci.yml` 的 `wrangler types` 步骤已删（typecheck 变纯 `tsc --noEmit`）。
- `expireInvitations` 原来只在 `worker.ts` cron；并入 HTTP cron route。
- 限流 key 只信反代覆写的 `X-Forwarded-For`：自建路径没有 Cloudflare，认 `CF-Connecting-IP` 会让直连客户端伪造新桶绕过 429（blocker 修复后探针验证：伪造旋转 CF 头仍 429）。

## 验证结果（2026-09-16）

- lint：0 error（1 条既有 warning）。typecheck：纯 `tsc --noEmit` 通过（修掉一次 TS2395 `DbClient` 合并声明冲突）。build：无 `DATABASE_URL` 也通过（与 CI 注释一致——触库页面都是 on-demand）。
- R2 双向隔离：活库 `POST /api/submit` → 401（查库）；死库（5999）同一请求 → 500。启动日志无 workerd。
- Compose `next dev`（去 Hyperdrive env 后）：Ready，全页面 200。
- `test:migrations`、`test:teams` 通过。Playwright 10/10（报告 `tests/e2e/reports/html/playwright-report-self-host-cutover-feature_teamboard-teams-auth-2026_09_16-15_09_30.*`）。
- 限流实测：XFF 同 IP 第 11 次起 429；伪造旋转 `CF-Connecting-IP` 仍 429（绕过已关）。
- cron 手动实证：正常 → `{"refreshedUsers":9,"expiredEmailTokens":3,"expiredInvitations":0}` 200；停库 → `{"error":"One or more cron jobs failed"}` 500；恢复 → 200。
- `poolMax` 默认从 1 提到 5（serverless 冷启动推理不适用于长驻进程；`DATABASE_POOL_MAX` 可调）。

## Release Readiness

- 生产路径：自建 Node `next start` 服务公开 web + CLI API；删掉的 Workers 路径本 fork 从未部署过，无线上系统受影响。
- 失败模式：库不可达 → 500（fail-closed 已验证）；cron 失败 → 500 + 任务清单，调度器重试；邮件失败只打日志。
- 容量：池 `max=5`/进程；限流 10/60s/IP 进程内（多实例需共享存储，已记录）。
- 回滚：重部署上一 commit；数据层两向不动（无迁移）。
- 残余风险（owner 2026-09-16 显式接受）：cron sweep 路径无正式 repeatable e2e（一次性手动已验证，BDD @todo 在案）；外网 HTTPS + 真实域名 + 外网 CLI 端到端只能上线时验。
