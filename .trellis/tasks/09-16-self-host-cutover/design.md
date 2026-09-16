# 技术设计：Self-host cutover

## 1. 部署拓扑决策

| 决策 | 结论 | 理由 |
|---|---|---|
| 生产运行时 | Node `next start`（长驻进程） | 自建云服务器是唯一目标；Workers 层（wrangler.jsonc、worker.ts、OpenNext 配置）整层删除 |
| DB 连接来源 | 仅 `DATABASE_URL` | 实测 CF dev context 存在时 Hyperdrive `localConnectionString` 压过 `DATABASE_URL`（死库 5999 仍 200）；`process.env.NEXT_PHASE` 在 `next start` 加载配置时不可靠，中间闸用 config 的 `phase` 回调参数（commit 8a84c3eb），删除后该调用整体消失 |
| 连接池 | 进程级单例，默认 `max=5`（`DATABASE_POOL_MAX` 1..5 可调） | 旧 `max=1` 是 serverless 冷启动防 `max_connections` 的推理，对长驻进程会把所有用户的查询串行化；5 覆盖最宽页面（`/u/[username]` 三条并行）并留余量 |
| TLS | `DATABASE_SSL` 显式开关，生产默认 `require` | 自建库通常无 TLS 时用 `disable`；保留对外部托管 PG 的兼容 |
| `prepare: false` | 保留 | `max_lifetime` 回收连接，prepared statement 是连接级的，跨回收边界会报 does not exist |

## 2. 缓存语义（Node 默认）

删 R2 / Durable Objects / `caches.default` 后，`unstable_cache` 与 `revalidateTag`/`revalidatePath` 回落到 Next 默认 cache handler（`.next/cache` 文件系统 + 进程内存）：

- TTL/tag 语义不变（60s revalidate、同名 tags），调用点零改动。
- 进程重启后缓存冷启动——都是 TTL 派生缓存，可接受。
- 边缘 HTML/SVG 缓存（OG、embed、badge、未登录首页/榜单）不复存在 → 每次回源渲染。是性能差异不是正确性问题；`/teamboard` 的 public→private 立即 404 不变式反而更强（共享 HTML 缓存已不存在）。
- `pageCacheable.ts` 与 `check-page-cacheable.ts` 随之删除（无消费者，边缘缓存机制本身没了）。

## 3. 登录限流（可信代理前提）

`authRateLimitAllowed(request)` 签名不变，四个调用点（register/login/forgot-password/resend-verification）零改动：

- 进程内固定窗口：每 IP 10 次 / 60s。
- Key 顺序：`CF-Connecting-IP` → `X-Forwarded-For` 第一段 → `0.0.0.0`。反代必须覆写 XFF，否则客户端可伪造绕过——部署文档已记。
- ponytail 上限：计数是单进程的；水平扩容需共享存储（在 rateLimit.ts 注释与部署文档标注）。

## 4. Cron 重试契约（替代 Worker 定时器）

`POST /api/cron/refresh-social-links`（`CRON_SECRET` Bearer，`timingSafeEqual` 比较）：

- 三个任务 `refreshAllSocialLinks` / `deleteExpiredEmailTokens` / `expireInvitations` 用 `Promise.all` + 各自 `.catch` 并发执行——保持 worker.ts 里三个独立 `waitUntil` 的互跳语义（social 失败不得跳过邀请过期）。
- 响应是调度器的重试信号：全成功 `200` + 各任务计数；任一失败 `500`。全部幂等 sweep，重试安全。
- 调度器从同机 loopback 调用（system cron + curl），不经公网反代，规避 proxy 超时截断长任务。
- 旧的「202 + 后台浮空 promise」被否决：Node 下 202 后进程可能 deploy/重启，`void work` 会静默丢任务。
- BDD：`web/features/maintenance-cron.feature`（401 场景自动化；200/失败场景 @todo——`refreshAllSocialLinks` 对每个用户请求 GitHub 且改写共享库快照，正式 e2e 会变成外部网络依赖）。

## 5. 回滚

- 回滚 = 重新部署上一个 commit；数据层两向不动（无迁移、无 schema 变更、无回填）。
- Workers 层的文件是整层删除，git revert 即可完整还原。
- 库连接行为回滚点：`db/index.ts` 是纯 `DATABASE_URL`，回滚后恢复 Hyperdrive 优先逻辑。

## 6. 不变量

- CLI 契约不变（device flow 三端点、`tt_` token、`TOKENS_API_URL`、`credentials.json`）。
- 会话 cookie / CSRF / 设备流行为不变。
- DB schema 与迁移链不变（仍到 0026）。
