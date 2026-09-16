# 本地 OrbStack Compose（给 Agent）

本 fork **本地**数据库是这份 OrbStack Compose 里的自建 Postgres。云服务器自建是**计划**，切流 pending：`web/wrangler.jsonc` 仍有真实 `HYPERDRIVE` 绑定（注释写的是上游 Neon）。Neon + Hyperdrive 是直接上游 `missuo/tokens` 的线上拓扑，不是本 fork 已落地的云上部署。


本地 UI 验证用 Compose，**不**挂 git checkout。仓库改完必须先 rsync 再起 web。Compose 文件不在本仓库。

相关：`docs/local-dev-database.md`（库约定与禁止项）；`docs/upstream_policy.md` §2.1 托管行。


## 目录结构

```
/Users/lusonglin/github/tokens/          # git checkout（改代码只在这里）
/Users/lusonglin/docker-compose/tokens/  # 唯一允许的 Compose volume 根
├── docker-compose.yml
├── postgres/data/                       # PG 数据（不要当源码同步）
├── app/                                 # 源码副本 → 容器 /app
├── node_modules/                        # → 容器 /app/node_modules
├── web_node_modules/                    # → 容器 /app/web/node_modules

└── next/                                # → 容器 /app/web/.next
```

| 宿主机路径 | 容器路径 | 作用 |
|---|---|---|
| `.../tokens/postgres/data` | `/var/lib/postgresql/data` | Postgres 16 数据 |
| `.../tokens/app` | `/app` | 从 checkout rsync 的可运行副本 |
| `.../tokens/node_modules` | `/app/node_modules` | 仓库根 workspace 依赖 |
| `.../tokens/web_node_modules` | `/app/web/node_modules` | web 包依赖 |
| `.../tokens/next` | `/app/web/.next` | Next 构建缓存 |

禁止再把 `/Users/lusonglin/github/tokens` 写进 `volumes:`。

## 服务

| 项 | 值 |
|---|---|
| 项目名 | `tokens-local` |
| 容器 | `tokens-postgres`、`tokens-web` |
| 引擎 | `postgres:16`、`oven/bun:1` |
| Web 命令 | 仓库根 `bun install --frozen-lockfile`，然后 `bun run --cwd web dev -- --hostname 0.0.0.0 --port 3000` |
| 浏览 | http://localhost:3000（`127.0.0.1:3000`） |
| 宿主机 `DATABASE_URL` | `postgresql://tokens:tokens@127.0.0.1:5433/tokens` |
| 容器 `DATABASE_URL` | `postgresql://tokens:tokens@postgres:5432/tokens` |
| TLS | `DATABASE_SSL=disable` |
| 上游绑定覆盖 | `CLOUDFLARE_HYPERDRIVE_LOCAL_CONNECTION_STRING_HYPERDRIVE` 指到 Compose `postgres:5432`（`web/` 仍读 wrangler Hyperdrive 绑定，数据在自建库） |

Web 是 **`next dev`**，不是 wrangler preview。本机若已有进程占 3000，先停掉。云上自建库尚未落地，不要把这份 Compose 密码当云上配置。


## 仓库改动后（Agent 必做）

每次改完 git checkout 里的代码、要在浏览器或 Playwright 对 Compose 验 UI 时，按顺序执行。不要只 `docker compose restart`：`./app` 不会自动跟着 git 变。

```bash
cd /Users/lusonglin/docker-compose/tokens
rsync -a --delete --delete-excluded \
  --exclude node_modules \
  --exclude web/node_modules \
  --exclude web/.next \
  --exclude .git \
  --exclude test-results \
  --exclude '.env' \
  --exclude '.env.local' \
  --exclude '.env*.local' \
  --exclude '.env.*.local' \
  /Users/lusonglin/github/tokens/ ./app/




docker compose up -d
docker compose logs -f web
```

等到日志出现 `Ready`。`bun install --frozen-lockfile` 不得改仓库 `bun.lock`；若 compose 把 lockfile 写脏了，在 checkout 里 `git checkout -- bun.lock`。

依赖变了（改 `package.json` / `bun.lock`）同样走上面 rsync + `up -d`，让容器按冻结 lockfile 重装。不要在容器里跑未冻结的 `bun install`。

## 验收

```bash
curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:3000/leaderboard
curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:3000/teamboard
curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:3000/login
```

期望 200。Playwright 打已在跑的 Compose 时用 `web/playwright.config.ts` 的 `reuseExistingServer`，宿主机 `DATABASE_URL` 用 `127.0.0.1:5433`。

## 停

```bash
cd /Users/lusonglin/docker-compose/tokens
docker compose down
```

保留 `postgres/data` 与 bind 目录。不要对正在用的本地库 `down -v`。

## 禁止

- 不要 `brew install postgresql@*` / `brew services start postgresql@*`
- 不要对未授权远程库跑 `test:migrations`（含上游 Neon、尚未交付的云上库）
- 不要把 Compose 密码用到云上自建库
- 不要把 checkout 的 `.env` / `.env.local` / `.env*.local` / `.env.*.local` 同步进 `./app`；保留 `.env.example`。容器 `DATABASE_URL` 只来自 compose `environment`


- 不要假设改 checkout 后容器里已经是新代码

