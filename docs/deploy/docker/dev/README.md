# dev/ — 本地开发栈参考模板

`docker-compose.yml` 是本地开发栈的**参考副本**（真实运行副本在开发机的 compose 目录，不入库）。它把 fork 的自建拓扑跑起来：Postgres 16 + `next dev`，完全不依赖 Cloudflare / Neon。

## 它是什么

- 两个服务：`postgres`（`127.0.0.1:5433`）+ `web`（`127.0.0.1:3000`，`oven/bun:1` 里跑 `bun install --frozen-lockfile && next dev`）。
- 所有卷都是 compose 文件旁的相对路径：`postgres/data`（库数据）、`app`（仓库的 rsync 副本，**不是**活挂载）、`node_modules` / `web_node_modules` / `next`（构建缓存）。
- 环境变量全部是 dev 占位值（`tokens/tokens`、`local-cron-secret`），仅 localhost 用，**不要**搬到任何共享/云上环境。

## 怎么用它起本地栈

```bash
mkdir -p ~/docker-compose/tokens && cd ~/docker-compose/tokens
cp <本仓库>/docs/deploy/docker/dev/docker-compose.yml .
mkdir -p app node_modules web_node_modules next
rsync -a --delete --delete-excluded \
  --exclude node_modules --exclude web/node_modules --exclude web/.next \
  --exclude .git --exclude test-results \
  --exclude '.env' --exclude '.env.local' --exclude '.env*.local' --exclude '.env.*.local' \
  <本仓库>/ ./app/
docker compose up -d
```

之后每次改了仓库代码要验证，重新 rsync `./app/` 再 `docker compose up -d`（不能只 restart——`./app` 不会自动跟着仓库变）。完整约定、验收命令、禁止项见 `docs/deploy/local-orbstack-compose.md`。

## 与本目录其他文档的关系

- `docs/deploy/local-orbstack-compose.md`：本地栈的**操作手册**（启停、rsync、验收、禁止项）——以它为准；本目录是模板本体。
- `docs/deploy/self-host-production.md`：云上生产清单（HTTPS、迁移、限流、cron、CONTACT_EMAIL）。本模板只覆盖本地 dev，不是生产编排。
- `docs/deploy/tokens-cli-usage.md`：CLI 接到这个本地栈的手册（`TOKENS_API_URL=http://localhost:3000`）。

## 要改的地方

- 端口冲突：改 `5433` / `3000` 的宿主侧映射（保持 `127.0.0.1` 前缀，别绑 `0.0.0.0`）。
- 库名/账号：三处 `tokens` 占位一起改，且与 `DATABASE_URL` 保持一致。
- 常驻后台上报：本栈不需要 `tokens serve`；CLI 侧见 `tokens-cli-usage.md`。
