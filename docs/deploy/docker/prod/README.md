# prod/ — 生产编排（web + Postgres）

把「上线」变成一份配置 + 一条命令。本目录是生产编排**本体**：`Dockerfile`（仓库根 context 的多阶段构建）、`docker-compose.yml`（`postgres:16` 内部网络 + `web` 生产镜像）、`.env.example`（环境变量模板）。

与 `docs/deploy/self-host-production.md` 的分工：本目录只把编排可重复化；**不替代**那份清单——HTTPS 反代、`NEXT_PUBLIC_URL`、用户侧 `TOKENS_API_URL`、迁移到 0026 等 blocker 与外网验收项以它为准。先读清单，再用本目录执行。

## 拓扑

- `postgres`：`postgres:16`，命名卷 `pgdata`，**不映射宿主端口**（只在 compose 内网可达）；`pg_isready` 健康检查。
- `web`：从 `Dockerfile` 构建（`next build` + `next start`，构建期不需要 `DATABASE_URL`）；`127.0.0.1:${WEB_PORT:-3000}:3000` 只绑 loopback，供本机反代转发；`restart: unless-stopped`；依赖 postgres 健康后启动。
- compose 项目名固定 `tokens-prod`，与本地 dev 栈（`tokens-local`）不冲突。

## 首次上线

前置：云服务器已装 Docker + compose 插件；已按 `self-host-production.md` 决定域名并备好 HTTPS 反代（nginx/caddy 自选，反代必须覆写 `X-Forwarded-For`）。

```bash
git clone <本仓库> && cd <本仓库>/docs/deploy/docker/prod
cp .env.example .env
# 编辑 .env：POSTGRES_PASSWORD / DATABASE_URL（两处密码一致）/
# NEXT_PUBLIC_URL=https://你的域名 / CRON_SECRET，按需配 CONTACT_EMAIL、
# RESEND_API_KEY、EMAIL_FROM。占位值一个都不许留。
docker compose up -d --build
# 建库迁移（在 web 容器内跑 drizzle-kit migrate，目标 0026）：
docker compose exec web bun run --cwd web db:migrate
```

反代把 `https://你的域名` 转发到 `127.0.0.1:3000`（或你设的 `WEB_PORT`）。之后按 `self-host-production.md` 的「验收」节做外网端到端验收。

## 发版

```bash
cd <本仓库> && git pull
cd docs/deploy/docker/prod
docker compose up -d --build        # 有迁移时先 exec … db:migrate（见上）
```

镜像构建走 Docker 层缓存：deps 层只在 `package.json` / `bun.lock` 变化时重装。

## 回滚

```bash
cd <本仓库> && git checkout <上一个可用版本>
cd docs/deploy/docker/prod
docker compose up -d --build
```

`pgdata` 命名卷不动，数据保留。**注意**：只适用于无破坏性迁移的回退；若两版本之间跑过后向不兼容的库迁移，先按迁移内容评估再回滚。

## cron（每日维护）

没有托管调度器。系统 cron 在同机打 loopback 维护端点（同步执行社交链接刷新 + 过期邮件 token 清理 + 邀请过期，全幂等，非 2xx 重试安全）：

```cron
# /etc/cron.d/tokens — 每天一次，失败非 0 退出交给 cron 邮件/告警
17 3 * * * root curl -fsS -X POST http://127.0.0.1:3000/api/cron/refresh-social-links -H "Authorization: Bearer <.env 里的 CRON_SECRET>"
```

未配 `CRON_SECRET` 端点返回 503；密钥错误 401。

## 常用排障

```bash
docker compose logs -f web                 # web 日志
docker compose ps                          # 健康状态
docker compose exec postgres psql -U tokens -d tokens   # 进库（凭据见 .env）
```

本地整栈验证（不干扰 dev 栈）时用 `WEB_PORT=3001 docker compose up -d --build` 换宿主端口；postgres 本来不占宿主端口，无需额外处理。
