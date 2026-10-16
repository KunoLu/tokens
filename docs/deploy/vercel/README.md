# Vercel 部署（Hobby）

[English](./README.en.md)

项目名 **`kunolu-tokens`**。域名买下来之前，用免费地址 **`https://kunolu-tokens.vercel.app`**。这是独立于 Docker/自建的一条路径：`docs/deploy/docker/prod/` 不改，自建默认仍是进程内限流、连接池默认 5、系统 cron `POST`。

## 项目设置

| 项 | 值 |
|---|---|
| Root Directory | `web` |
| Include files outside of the Root Directory in the Build Step | **打开** |
| Framework | Next.js |

构建脚本会从仓库根拷 `install.sh`、`pre-install-tokens.sh`、`pre-install-tokens.ps1`、`enable-tokens-service.sh`、`register-tokens-submit-task.ps1` 和 `.github/assets/client-*`。Root Directory 设成 `web` 时，这个开关关掉，构建上下文里没有这些文件，`bun run build` 失败。`web/vercel.json` 的 `installCommand` 也假定能看到上一层的 `bun.lock`。

## Neon：Preview 与 Production 分分支

- Production 与 Preview **各用一条 Neon 分支**。Preview 的 `DATABASE_URL` 不得指向生产分支。
- 优先用 Neon–Vercel 集成：每个 Preview 部署从生产分支 fork 出自己的分支（fork 时带上当时的 schema），而不是连生产库。
- 运行时 `DATABASE_URL` 用 **pooled** 连接串（`-pooler` 主机）。`drizzle-kit migrate` 要用 **direct / unpooled** 连接，pooled 上的会话特性会让迁移失败。Neon–Vercel 集成提供 `DATABASE_URL_UNPOOLED`。
- `DATABASE_SSL` 留空或 `require`。不要设 `disable`（Neon 要 TLS；代码在 `NODE_ENV=production` 且未设该变量时默认 `require`）。
- **`DATABASE_POOL_MAX=1`**，Production 和 Preview 都设。代码默认是 5，那是给 Docker 里长期存活的单进程用的（夹在 1..5）。Vercel 每个 isolate 自己开池，默认 5 会打满 Neon 连接数。不要改代码默认值。

## 迁移只在 Production 部署时跑

`web/vercel.json` 的 `buildCommand`：

```sh
if [ "$VERCEL_ENV" = "production" ]; then
  DATABASE_URL="${DATABASE_URL_UNPOOLED:-$DATABASE_URL}" bun run db:migrate
fi && bun run build
```

- 只有 `VERCEL_ENV=production` 才执行 `bun run db:migrate`（`drizzle-kit migrate`）。Preview 构建不迁移。
- 迁移优先用 `DATABASE_URL_UNPOOLED`，没有才回退 `DATABASE_URL`。
- Production 的 `DATABASE_URL` / `DATABASE_URL_UNPOOLED` 必须在 **构建期** 可用，否则生产构建在迁移处失败。Preview 环境变量里不要放生产库连接串。
- 长期 Preview 分支不会自动跟上生产迁移。用集成的 per-deploy fork，或在生产迁移之后从生产分支重新 fork。不要对 Preview 库跑 `db:migrate`，也不要对云上库跑 `test:migrations`。

## 环境变量

Production 与 Preview 都要设（值按分支分开）：

| 变量 | 说明 |
|---|---|
| `NEXT_PUBLIC_URL` | 买域名之前：`https://kunolu-tokens.vercel.app`。若控制台显示的 `*.vercel.app` 主机名不同，用控制台那个，不要猜团队后缀 |
| `DATABASE_URL` | 该环境 Neon 分支的 pooled 连接串 |
| `DATABASE_URL_UNPOOLED` | 该环境的 direct 连接串。生产构建迁移用它 |
| `DATABASE_POOL_MAX` | `1` |
| `CRON_SECRET` | 至少 16 字符的随机串。Vercel Cron 会把它放进 `Authorization: Bearer`。手动重试的 POST 用同一密钥 |
| `UPSTASH_REDIS_REST_URL` | Upstash Redis REST URL |
| `UPSTASH_REDIS_REST_TOKEN` | Upstash Redis REST token |

可选：`RESEND_API_KEY` / `EMAIL_FROM`、`CONTACT_EMAIL`、`GITHUB_CLIENT_ID` / `GITHUB_CLIENT_SECRET`。

### Upstash

两个变量都非空才离开进程内限流。只设其中一个仍走进程内实现（与自建默认相同：10 次 / 60 秒 / 客户端 IP）。Vercel 上进程内 Map 只活在单个 isolate 里，跨实例无效，所以 Production 和 Preview 都要设这两个变量。

已配置 Upstash 时，Redis 报错或超时 **失败关闭**（拒绝该次认证请求），不会退回进程内桶，否则各 isolate 会各算一份额度。不要在 Docker compose 里设这两个变量；那条路径不设，限流保持进程内。

### 客户端 IP

平台会设 `VERCEL=1`。此时限流 key 优先 `x-vercel-forwarded-for` 第一跳（Vercel 写入；前面再套代理时 `x-forwarded-for` 可能被改写），其次 `x-real-ip`，都没有才回退 `X-Forwarded-For` 第一跳。不认 `CF-Connecting-IP`。

自建不设 `VERCEL`，也不设 `TRUST_PROXY`，key 仍只是 `X-Forwarded-For` 第一跳。`TRUST_PROXY=1|true|yes` 才启用与 Vercel 相同的平台头优先顺序；Docker 栈不要设。

## Cron

`web/vercel.json` 把 `GET /api/cron/refresh-social-links` 排在 `17 3 * * *`（每天一次）。Hobby 拒绝一天多于一次的表达式；时刻精度是小时级（±59 分钟）。

GET 与 POST 共用 Bearer 检查和三个任务（社交链接刷新、过期邮件 token、邀请过期）。全成功 200，任一失败 500，未配密钥 503，密钥不对 401。自建系统 cron 仍 `POST` loopback。Vercel Cron 用 GET，失败不自动重试，看函数日志。

## 自定义域名

买域名之前用免费地址 `https://kunolu-tokens.vercel.app`（项目名 `kunolu-tokens`）。`NEXT_PUBLIC_URL` 先设成这个地址并部署一次。

以后改域名：

1. 在 Vercel 项目 `kunolu-tokens` 上绑定新域名。
2. 把 `NEXT_PUBLIC_URL` 改成 `https://<新域名>`。
3. **重新部署。** `NEXT_PUBLIC_*` 在构建期被内联。只改环境变量、不重新构建，device 链接、验证信、邀请链接、CSRF Origin、OG 和 embed 仍指向旧 origin。

用户机器上的 `TOKENS_API_URL` 也要改成新域名。凭据不存 API 基址，旧站登录过的用户要重新 `tokens login`。
