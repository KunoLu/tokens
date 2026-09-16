# 云服务器自建部署清单（Web + Postgres）


本 fork 的目标部署：云服务器自建 Postgres + Node 跑本仓 `web/`。**不是**上游 `missuo/tokens` 的 Neon + Hyperdrive + Cloudflare Workers 拓扑。

相关：本地开发栈 `docs/local-dev-database.md`、`docs/deploy/local-orbstack-compose.md`（OrbStack Compose，仅本地 dev）。**仓内目前没有生产 Dockerfile / compose / 编排**，本文件是上线前必须满足的条件清单。

## 结论

外网用户用**同一套上游 tokens CLI**（不改二进制）指向本站 HTTPS API 后，登录、上报、Leaderboard 与 Team（建团、角色、用户名邀请、Teamboard）可用。但以下 blocker 任一不满足，都会表现为「连了但进不了本站」或 Team 半残。

新自建库是**独立空库**：`tokens.ci` 的账号、`tt_` token、历史用量不会过来。用户须在本站重新注册、重新 device login。目前没有上游数据/账号迁移方案。

## Blocker（缺一条就不可用或不安全）

| # | 条件 | 依据 | 不满足的后果 |
|---|---|---|---|
| 1 | 公网 **HTTPS** 反代 + 证书 | CLI device flow 只接受 HTTPS；HTTP 仅 loopback（`cli/tokens-cli/src/auth.rs:246-249`）。生产会话 cookie 仅 `NODE_ENV=production` 标 `Secure`（`web/src/lib/auth/session.ts:101-105`） | CLI 拒绝打开 verification URL；HTTP 下会话 cookie 落不了 |
| 2 | `NEXT_PUBLIC_URL=https://你的域名` | device `verificationUrl`、验证信、邀请链接都按它拼，默认 `http://localhost:3000`（`web/src/app/api/auth/device/route.ts:56-63`、`web/src/lib/auth/emailTokens.ts:13`、`web/src/lib/teams/service.ts:338`）；CSRF 也按它放行本站 Origin（`web/src/lib/auth/requestSession.ts:15-30`） | CLI 打印的授权链接指向 localhost；邮件/邀请链接错 |
| 3 | 用户侧 `TOKENS_API_URL=https://你的域名` | 默认固定 `https://tokens.ci`（`auth.rs:215-217`） | login / submit 仍进上游，本站榜是空的 |
| 4 | 既有 `tokens.ci` 用户**先退出再登录** | `credentials.json` 只存 `token`/`username`，不存 API 基址；`tokens login` 发现凭据直接返回（`auth.rs:338-351`） | 带上游 token 打本站 `/api/submit` → **401** |
| 5 | 自建库跑本 fork 迁移到 **0026** | `0024` 邮箱密码认证、`0025` teams/groups、`0026` 邀请 `group_id`（`web/src/lib/db/migrations/`） | 没有邮箱认证和 Team 表 |
| 6 | 生产**不跑** `initOpenNextCloudflareForDev()`，只信 `DATABASE_URL` | 该调用在 `web/next.config.ts` 顶层无条件执行；实测：有 CF dev context 时 `createDb()` 优先用 Hyperdrive（`web/src/lib/db/index.ts:61-68`），隔离实验里 `DATABASE_URL` 指到不可达端口页面仍 200——Hyperdrive 胜出 | 云上按 `wrangler.jsonc` 的 `localConnectionString` 连 `127.0.0.1:5433`，或连上游 Hyperdrive id，无视你的 `DATABASE_URL` |
| 7 | **禁止**用仓内 `web/wrangler.jsonc` `wrangler deploy` | 顶层配置注释写明每次 deploy 直达 live；Hyperdrive id `35e35c2c…` 是上游线上绑定 | 流量/数据进上游 |

### 用户侧迁移口令（写进上线公告）

```bash
tokens logout                                        # 4：清掉 tokens.ci 凭据
TOKENS_API_URL=https://你的域名 tokens login          # 开 https://你的域名/device 授权
TOKENS_API_URL=https://你的域名 tokens submit
```

`TOKENS_API_TOKEN` 同理：指错站就 401。**双站并存**（同时用 tokens.ci 与本站）现成 CLI 做不到：`TOKENS_CONFIG_DIR` 可隔离两套配置目录，但没有 endpoint profile；要双站需先改 CLI，不能只靠部署文档。

## 自建后不会自动出现的能力（非冲突，是缺口）

| 能力 | Workers 上 | 自建 Node | 处置 |
|---|---|---|---|
| 过期邀请 `expireInvitations()` | `web/worker.ts` cron | 不跑；HTTP `/api/cron/refresh-social-links` **也不调它** | 自建定时任务调该逻辑，否则邀请永远 pending |
| 过期邮件 token、社交链接刷新 | 同上 cron | 可 `POST /api/cron/refresh-social-links`（Bearer `CRON_SECRET`），进程内执行 | 挂系统 cron / 定时任务 |
| 登录/注册限流（10 次/60s） | `AUTH_RATE_LIMITER` | **skip 放行**（`web/src/lib/auth/rateLimit.ts:16-24`） | 上线前补自己的限流，或接受裸奔 |
| 邮件（验证信/邀请信） | Resend + `RESEND_API_KEY`/`EMAIL_FROM` | 未配置则只打日志 | 配 Resend 或自建 SMTP 前，Team 邮件邀请视为 blocked |
| R2/DO/边缘缓存（OG、embed、未登录 HTML） | Cloudflare 绑定 | Next 默认缓存 | 可接受源站直出，或前置 CDN |

## Team 可用性分级

| 能力 | 条件 |
|---|---|
| 建团 / 角色 / 用户名邀请 / Teamboard / 榜上 Team·Group 列 | 库迁到 0026 + Web 起来即可 |
| 邮件邀请发信 | 需 `RESEND_API_KEY` + `EMAIL_FROM`；否则邀请落库但对方收不到 |
| 按邮箱邀请的接受 | 要求被邀请人 `emailVerifiedAt`（`web/src/lib/teams/service.ts:832-835`），没验证信会 403 |
| 邀请过期 | 需上一节的 `expireInvitations` 定时任务 |

## 已验证 / 未验证

**已验证（本机）**

- `bun run build` 通过；`next start -p 3001` + 自建 Compose Postgres，`/leaderboard` `/teamboard` `/teams` `/login` `/docs` 均 200。
- 但进程仍加载 workerd/DO 桩；且隔离实验（`DATABASE_URL` 指向空端口 5999）页面仍 200 → 当时的 200 **不是** `DATABASE_URL` fallback 的证据，走的是 Hyperdrive 本地串（同一库）。

**未验证**

- 关掉 `initOpenNextCloudflareForDev()` 后的纯 `DATABASE_URL` 生产路径（对应 Blocker #6）。
- 外网 HTTPS + 真实域名 + 外网 CLI 端到端。

## 上线步骤（顺序执行）

1. 云服务器装 Postgres 16，建库建号；不要用 Compose 的 `tokens/tokens` 口令。
2. 跑本仓迁移到 0026：**生产库只执行 `DATABASE_URL=… bun run db:migrate`（`drizzle-kit migrate`）**。不要对云上库跑 `bun run test:migrations`——它还会执行 `scripts/check-migrations.ts`，在事务里插入 `ci_migration_replay` / `CI Team` 等回放数据（虽 ROLLBACK，但会拿锁、跑断言，是验证工具不是迁移工具），且 `docs/local-dev-database.md` 明确禁止对未授权远程库跑它。`test:migrations` 只对本地 Compose 或一次性隔离验证库执行。

3. 改 `web/next.config.ts`：生产构建不调用 `initOpenNextCloudflareForDev()`（或按环境变量闸住），DB 只走 `DATABASE_URL`。
4. 反代 HTTPS；环境：`NODE_ENV=production`、`NEXT_PUBLIC_URL=https://你的域名`、`DATABASE_URL=…`、按 TLS 设 `DATABASE_SSL`（无 TLS 用 `disable`）。
5. `next build && next start`（或容器化等效）。**不**用 wrangler / OpenNext deploy。
6. 端到端验收（见下）。
7. 定时任务：`expireInvitations`；可选邮件 token 清理（`/api/cron/refresh-social-links` + `CRON_SECRET`）。
8. 登录限流替代；要邮件邀请再配 Resend。

## 验收（外网，不是 localhost）

- [ ] `TOKENS_API_URL=https://域名 tokens login` 打印 `https://域名/device?…`，浏览器授权后拿到本库 `tt_` token
- [ ] `tokens submit` 返回成功，本站 Leaderboard 出现该用户用量
- [ ] 未验证邮箱的用户按邮箱被邀请时接受被拒（403），验证后接受成功（配了 Resend 的前提下）
- [ ] 用户名邀请全流程（建团 → 邀请 → 接受 → Teamboard 选人 → 榜列显示 Team/Group）
- [ ] 过期邀请被定时任务标记 expired
- [ ] `tokens.ci` 老用户未 logout 直接 submit → 本站 401（确认拦截，不是误收上游数据）
- [ ] 页脚 Workers/Neon/V.PS 文案与真实拓扑一致，或已按产品决定移除

## 待开发项（不属于部署，属于缺口）

1. **CLI endpoint profile / `TOKENS_CONFIG_DIR` 双站方案**：允许 tokens.ci 与自建站并存。目前是单凭据单基址。
2. **`expireInvitations` 的 Node 定时触发**：包进 HTTP cron route 或独立脚本。
3. **登录限流自建替代**：当前 Node 下放行。
4. **生产容器化资产**：Dockerfile / compose / systemd，仓内尚无。
5. **页脚文案**：Workers/Neon/V.PS 为上游事实，自建后应删改（已建议只留 `Privacy · Terms · Built on Tokscale`）。
