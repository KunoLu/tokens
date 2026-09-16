# Self-host cutover: drop Cloudflare Workers layer

## Goal

自建（云服务器 Postgres + Node `next start`）是唯一部署目标。分两段落地：先闸住 `initOpenNextCloudflareForDev()` 并验证纯 `DATABASE_URL` 生产路径，再删除整个 Workers 层。

## Requirements

- **R1** `next.config.ts`：仅非 production-server 阶段调用 `initOpenNextCloudflareForDev()`（dev/build 保留现状，便于本地 Compose 开发）。
- **R2** 验证纯 `DATABASE_URL` 路径：`next start` 下 Hyperdrive 不再压过 `DATABASE_URL`；隔离实验（死 `DATABASE_URL`）必须失败、活库必须 200。
- **R3** 删除 Workers 层：`web/wrangler.jsonc`、`web/worker.ts`、`web/open-next.config.ts`、`web/open-next-worker.d.ts`、`web/worker-startup.cpuprofile`；`package.json` 的 `cf:*` 脚本与 `typecheck` 的 `wrangler types` 前缀；依赖 `@opennextjs/cloudflare`、`wrangler`。
- **R4** 清理 `@opennextjs/cloudflare` 引用：`db/index.ts`（纯 `DATABASE_URL` + 进程单例）、`rateLimit.ts`（换成 Node 进程内固定窗口限流，签名不变）、`email/send.ts`、`api/cron/refresh-social-links/route.ts`（去 `waitUntil` 分支）、`next.config.ts`（删 init 调用）。
- **R5** `expireInvitations()` 并入 HTTP cron route（带 `CRON_SECRET`），邀请过期能力不因删 Worker 而丢失。
- **R6** CI / compose / 文档 / spec 同步：`.github/workflows` 若有 wrangler 引用则改；compose 与 `docs/deploy/*`、`docs/local-dev-database.md`、`docs/upstream_policy.md` 托管行、`.trellis/spec/web/frontend/cloudflare-deployment.md`（及 index 引用）改成自建现实。

## Constraints

- 不改 CLI。CLI 契约（device flow、`tt_` token、`TOKENS_API_URL`）不变。
- 不改 DB schema / 迁移；库还是同一个 Postgres。
- 会话 cookie、CSRF、device flow 等 Web 行为不变。
- `web/src/lib/cache/pageCacheable.ts` 与 `scripts/check-page-cacheable.ts` 是 Worker 边缘缓存的遗留证据：Worker 删除后该缓存机制不存在，matcher 仅作为「不在边缘缓存 Teamboard」的历史约束；按干净切换处理（删除或保留需有明确理由）。

## Acceptance Criteria

- [x] `bun run lint`、`bun run typecheck`（纯 `tsc --noEmit`）、`bun run build` 全绿
- [x] `DATABASE_URL=<活库> next start`：核心页面 200；`DATABASE_URL=<死端口> next start`：页面报错（证明走 DATABASE_URL）
- [x] Compose `next dev`（去掉 Hyperdrive env 覆盖后）照常工作
- [x] `bun run test:migrations`、`bun run test:teams` 通过
- [x] Playwright 回归：`teams-gate`、`t9-acceptance`（Sign in 链接）、`teamboard` 通过
- [x] 全仓 grep 无 `@opennextjs/cloudflare`、`getCloudflareContext`、`wrangler.jsonc` 残留引用（文档历史叙述除外）
- [x] cron route 调 `expireInvitations`，且有 `CRON_SECRET` 保护
- [x] 限流在 Node 下仍生效（非放行）
- [x] 文档与 spec 不再把 Cloudflare/Neon 描述为本 fork 目标

## Notes

- 验收用本地 Compose Postgres（`127.0.0.1:5433`）；不对任何远程库跑迁移或测试。
- `rtk`：lint/typecheck 可用；产生报告文件的测试用原生命令。
