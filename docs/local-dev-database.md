# 本地开发数据库与 Web

本仓库没有独立 backend 包；API / Drizzle schema 在 `web/`。

**本 fork 的库是自建 Postgres**（本地 OrbStack Compose，见 `docs/deploy/local-orbstack-compose.md`；云服务器同法自建）。Workers/Hyperdrive 层已在 self-host cutover 中删除；云上清单见 `docs/deploy/self-host-production.md`。**Neon + Hyperdrive 是直接上游 `missuo/tokens` 的历史拓扑，与本 fork 无关。**




本地开发与 `bun run test:migrations` **不要用 Homebrew PostgreSQL**。启停与 rsync 以 `docs/deploy/local-orbstack-compose.md` 为准。



## 位置

| 项 | 路径 / 值 |
|---|---|
| Compose 文件 | `/Users/lusonglin/docker-compose/tokens/docker-compose.yml` |
| Postgres 数据 | `.../tokens/postgres/data/` |
| 源码副本 | `.../tokens/app/`（从 git checkout rsync，不直接挂仓库） |
| 仓库根 `node_modules` | `.../tokens/node_modules/` |
| `web/node_modules` | `.../tokens/web_node_modules/` |
| `web/.next` | `.../tokens/next/` |
| 容器名 | `tokens-postgres`、`tokens-web` |
| Postgres 端口 | `127.0.0.1:5433`（容器内仍是 5432）。本机 5432 已被 `keyboy-play-local-db` 占用，不抢。 |
| Web 端口 | `127.0.0.1:3000`（容器内 Next `next dev` 听 `0.0.0.0:3000`） |
| 库 / 用户 / 密码 | `tokens` / `tokens` / `tokens`（仅本机，非生产） |
| 宿主机 `DATABASE_URL` | `postgresql://tokens:tokens@127.0.0.1:5433/tokens` |
| Web 容器 `DATABASE_URL` | `postgresql://tokens:tokens@postgres:5432/tokens`（服务名，不要写 `127.0.0.1`） |
| TLS | 关（本地容器无 TLS；Web 设 `DATABASE_SSL=disable`） |

Web 容器跑的是 **`next dev`**，数据库直连 `DATABASE_URL`（Compose 指向 `postgres:5432`）。没有任何 Cloudflare 绑定。安装用 `bun install --frozen-lockfile`，不改仓库 `bun.lock`。浏览器打开 `http://localhost:3000` 验 UI。邮件未配 `RESEND_API_KEY` 时只打日志。




## 启停

步骤与命令见 `docs/deploy/local-orbstack-compose.md`。不要把 git checkout 直接写进 Compose `volumes:`。


端口必须写成 `127.0.0.1:5433:5432` 和 `127.0.0.1:3000:3000`，不要省略 `127.0.0.1`（会绑到 `0.0.0.0`）。

本机若已有 `bun run dev` 占用 3000，先停掉再起容器。

健康检查通过后再跑迁移（在**宿主机**，连映射端口）：

```bash
cd /Users/lusonglin/github/tokens/web
DATABASE_URL=postgresql://tokens:tokens@127.0.0.1:5433/tokens bun run test:migrations
```

`web/.env` / `.env.local` 可写同一宿主机 `DATABASE_URL`；不要提交真实生产连接串。


## 禁止

- 不要为跑迁移再 `brew install postgresql@*`。
- 不要 `brew services start postgresql@*`（登录自启）。
- 不要把本机 Homebrew cluster 当项目运行时。
- 不要对未授权的远程库跑 `test:migrations`（含上游 Neon、尚未交付的云上库）。只打本机 `127.0.0.1:5433`。
- 不要把 Compose 里的 `tokens/tokens` 密码用到云上自建库。


## Homebrew 残留（已按授权清理）

2026-09-11 曾为 T0 验证安装 `postgresql@16`，cluster 在 `/opt/homebrew/var/postgresql@16`。已卸载公式并删除该 cluster。卸载时 Homebrew **自动**卸掉了当时作为依赖装上的 `krb5`（未执行 `brew autoremove`）。若其它软件需要 `krb5`，需自行重装。
