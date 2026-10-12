# Production Docker assets + docker dir dev/prod split

## Goal

把「上线」从手工命令变成一份配置 + 一条命令。对应 `docs/deploy/self-host-production.md` 待开发项 P1（生产容器化资产）。

同时把 `docs/deploy/docker/` 按环境分目录：现有本地开发栈挪进 `dev/`，生产资产放 `prod/`。

## Requirements

- **R1** 目录重构：`docs/deploy/docker/docker-compose.yml` + `README.md` → `docs/deploy/docker/dev/`；新增 `docs/deploy/docker/prod/`；顶层 `README.md` 改成环境导览（dev 与 prod 的职责分界）。
- **R2** `prod/Dockerfile`：多阶段构建 web 生产镜像。context 为**仓库根**（build 需要根目录的 `install.sh`、`pre-install-tokens.*`、`.github/assets/client-*`）。deps（`bun install --frozen-lockfile`）→ build（`bun run build`）→ runner（`next start`）。构建期不需要 `DATABASE_URL`（已验证：触库页面都是 on-demand，leaderboard 容忍缺省）。
- **R3** `prod/docker-compose.yml`：`postgres:16`（不暴露宿主端口，仅内部网络）+ `web`（从 Dockerfile 构建，`127.0.0.1:3000:3000` 供反代，`restart: unless-stopped`，`env_file: .env`，依赖 postgres 健康）。
- **R4** `prod/.env.example`：`DATABASE_URL`（指向 compose 内 postgres）、`NEXT_PUBLIC_URL`、`DATABASE_SSL`、`CRON_SECRET`、可选 `CONTACT_EMAIL` / `RESEND_API_KEY` / `EMAIL_FROM`。全部占位值。
- **R5** `prod/README.md`：生产操作手册——首次上线（建库迁移 → 配 .env → up）、发版（git pull → up --build）、回滚（checkout 上个版本 → up --build）、cron 挂载（系统 cron 打 loopback 维护端点）、与 `self-host-production.md` 的关系（它做编排可重复化，不替代里面的 blocker/验收）。
- **R6** 本地验证：在本机把 prod 栈跑起来（独立端口/项目名，不干扰 dev 栈），冒烟 `/leaderboard` 200、`/api/submit` 假 token 401（证明走 `DATABASE_URL`）。

## Constraints

- 不动 CLI。
- 不改 dev 栈行为——dev/docker-compose.yml 内容与现运行副本一致。
- 不引入新依赖；镜像基础沿用 `oven/bun:1`。
- 生产镜像不含 CLI 构建（`packages/cli` 是 npm 分发物，与本镜像无关）。
- 不在 compose 里放 HTTPS 反代（运营方自选 nginx/caddy，清单已写）。

## Acceptance Criteria

- [x] `docs/deploy/docker/{dev,prod}/` 分目录就位，顶层 README 导览两个环境
- [x] 根目录 `.dockerignore` 排除 `.git`、`node_modules`、`web/.next`、测试报告、本地 compose/运行时数据、`.env*` 秘物
- [x] `docker build -f docs/deploy/docker/prod/Dockerfile .` 在本机成功
- [x] prod 栈本地拉起后 `/leaderboard` 200；`POST /api/submit` 带伪造 token → 401
- [x] 关掉 DB 后同一请求 → 500（证明只走 `DATABASE_URL`）
- [x] prod README 覆盖：首次上线、发版、回滚、cron、与清单的关系
- [x] lint / typecheck / build 不回退；`docs/deploy/self-host-production.md` 待开发项移除 P1 条目
