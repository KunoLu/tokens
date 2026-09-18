# docker/ — 部署资产总览（dev / prod 分目录）

本目录按环境分两套 Docker 资产，职责互不重叠：

| 目录 | 用途 | 拓扑 | 什么时候用 |
|---|---|---|---|
| `dev/` | 本地开发栈**参考模板** | Postgres 16 + `oven/bun:1` 跑 `next dev`，源码走 rsync 副本 | 本机开发、UI/Playwright 验证 |
| `prod/` | 生产编排**本体** | 多阶段 Dockerfile 构建 web 生产镜像 + Postgres 16（命名卷，不暴露宿主端口） | 云服务器上线、发版、回滚 |

## dev/ — 本地开发栈

`dev/docker-compose.yml` 是本地栈（`tokens-local`）的净化参考副本：真实运行副本在开发机的 compose 目录，不入库。起栈方式、rsync 约定、验收命令见 `dev/README.md` 与操作手册 `docs/deploy/local-orbstack-compose.md`——以手册为准。

## prod/ — 生产编排

`prod/` 把「上线」变成一份配置 + 一条命令：`Dockerfile`（context 为仓库根的多阶段构建）、`docker-compose.yml`（postgres + web）、`.env.example`（全占位环境变量模板）、`README.md`（首次上线 / 发版 / 回滚 / cron 操作手册）。它只做编排可重复化；上线前必须满足的 blocker（HTTPS、`NEXT_PUBLIC_URL`、迁移到 0026 等）与外网验收清单仍在 `docs/deploy/self-host-production.md`，两者配合使用。
