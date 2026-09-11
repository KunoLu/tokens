# 本地开发数据库

生产数据库是 **Neon PostgreSQL**，Worker 经 **Cloudflare Hyperdrive** 访问。本仓库没有独立 backend 包；API / Drizzle schema 在 `web/`。

本地开发与 `bun run test:migrations` **不要用 Homebrew PostgreSQL**。约定用 OrbStack 管理的 Docker Compose 容器。

## 位置

| 项 | 路径 / 值 |
|---|---|
| Compose 文件 | `/Users/lusonglin/docker-compose/tokens/docker-compose.yml` |
| 数据目录 | `/Users/lusonglin/docker-compose/tokens/postgres/data/`（容器卷，勿提交） |
| 引擎 | PostgreSQL 16（`postgres:16`） |
| 容器名 | `tokens-postgres` |
| 端口 | `127.0.0.1:5433`（容器内仍是 5432）。本机 5432 已被 `keyboy-play-local-db` 占用，不抢。 |
| 库 / 用户 / 密码 | `tokens` / `tokens` / `tokens`（仅本机，非生产） |
| `DATABASE_URL` | `postgresql://tokens:tokens@127.0.0.1:5433/tokens` |
| TLS | 关（本地容器无 TLS） |

生产：Neon + Hyperdrive binding `HYPERDRIVE`。不要把 Hyperdrive 叠在 Neon 的 PgBouncer pooler 上。

## 启停（OrbStack）

OrbStack 提供本机 `docker`。在 compose 目录：

```bash
cd /Users/lusonglin/docker-compose/tokens
docker compose up -d
docker compose ps
docker compose down          # 停容器，保留 ./postgres/data
docker compose down -v       # 会删数据，需明确确认
```

端口必须写成 `127.0.0.1:5433:5432`，不要 `5433:5432` / `5432:5432`（会绑到 `0.0.0.0`）。

健康检查通过后再跑迁移：

```bash
cd /Users/lusonglin/github/tokens/web
DATABASE_URL=postgresql://tokens:tokens@127.0.0.1:5433/tokens bun run test:migrations
```

`web/.env` / `.env.local` 可写同一 `DATABASE_URL`；不要提交真实生产连接串。

## 禁止

- 不要为跑迁移再 `brew install postgresql@*`。
- 不要 `brew services start postgresql@*`（登录自启）。
- 不要把本机 Homebrew cluster 当项目运行时。
- 不要对生产 Neon 跑 `test:migrations`。
- 不要把容器密码用于生产。

## Homebrew 残留（已按授权清理）

2026-09-11 曾为 T0 验证安装 `postgresql@16`，cluster 在 `/opt/homebrew/var/postgresql@16`。已卸载公式并删除该 cluster。卸载时 Homebrew **自动**卸掉了当时作为依赖装上的 `krb5`（未执行 `brew autoremove`）。若其它软件需要 `krb5`，需自行重装。
