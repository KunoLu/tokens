# 实施计划：生产 Docker 资产 + docker 目录分环境

## 目录重构

```text
docs/deploy/docker/
├── README.md              # 改成环境导览：dev 与 prod 的职责分界
├── dev/
│   ├── docker-compose.yml # 从 docker/ 挪入，内容不变（现运行副本一致的 dev 栈）
│   └── README.md          # 从 docker/ 挪入，首行改为「dev/ — 本地开发栈参考模板」
└── prod/
    ├── Dockerfile         # web 生产镜像（多阶段）
    ├── docker-compose.yml # postgres + web
    ├── .env.example       # 环境变量模板（全占位值）
    └── README.md          # 生产操作手册
```

引用更新：`docs/deploy/local-orbstack-compose.md` 里的「模板本体」指针改为 `docs/deploy/docker/dev/docker-compose.yml`；`docs/deploy/self-host-production.md` 待开发项删掉 P1（生产容器化资产），并在上线步骤第 4 步注明可用 `docs/deploy/docker/prod/`。

## prod/Dockerfile（多阶段，context = 仓库根）

- **deps**：`oven/bun:1`，WORKDIR `/app`；COPY `package.json` `bun.lock` + `web/package.json` + `packages/*/package.json`（workspace 解析需要）→ `bun install --frozen-lockfile`。
- **build**：COPY 全仓（web/ 源码 + 根 `install.sh` + `pre-install-tokens.*` + `.github/assets/`）→ `bun run --cwd web build`（build 脚本会拷 install.sh / pre-install / client 资产进 `public/`，然后 `next build`）。构建期不需要 `DATABASE_URL`。
- **runner**：`oven/bun:1` 精简运行面：`/app/node_modules` + `/app/web/{.next,public,package.json,next.config.ts}`；ENV `NODE_ENV=production`；EXPOSE 3000；CMD `bun run --cwd web start -- -p 3000`（或 `web` 目录下 `next start`）。
- 根目录新增 `.dockerignore`：`.git`、`node_modules`、`web/node_modules`、`web/.next`、`test-results`、`docs`、`tests/e2e/reports`、`.trellis`、`.env*`（镜像不带本机密物）。

## prod/docker-compose.yml

- `postgres:16`：命名卷 `pgdata`；**不映射宿主端口**（只在内部网络）；healthcheck `pg_isready`；env 从 `.env` 读 `POSTGRES_*`。
- `web`：`build: { context: 仓库根, dockerfile: docs/deploy/docker/prod/Dockerfile }`（compose 文件里用相对仓库根的路径 `../../../..`）；`ports: 127.0.0.1:3000:3000`；`restart: unless-stopped`；`env_file: .env`；`depends_on: postgres healthy`。
- compose 项目名与 dev 区分（如 `tokens-prod`），本地验证时不撞 dev 栈。

## prod/.env.example

占位值：`POSTGRES_PASSWORD`、`DATABASE_URL`（指向 `postgres:5432`）、`NEXT_PUBLIC_URL`、`DATABASE_SSL=disable`（有 TLS 改 require）、`CRON_SECRET`、可选 `CONTACT_EMAIL` / `RESEND_API_KEY` / `EMAIL_FROM`。注释里写「全部要改，勿直接用占位值上线」。

## prod/README.md

生产操作手册：首次上线（建库 → `db:migrate` → 配 `.env` → `up -d --build`）、发版（`git pull` → `up -d --build`）、回滚（checkout 上一版本 → `up -d --build`）、cron（系统 cron 打 loopback 维护端点）、与 `self-host-production.md` 的分工（它做编排可重复化，不替代 blocker/验收清单）。

## 验证

1. `docker build -f docs/deploy/docker/prod/Dockerfile .` 成功（仓库根 context）。
2. 本地用独立项目名/端口起 prod 栈（不干扰 dev 栈）→ `/leaderboard` 200；`POST /api/submit` 伪造 token → 401；停 postgres → 同一请求 500；恢复 → 200。
3. `rtk bun run lint` + `rtk bun run typecheck` 不回退（本次不动 src，理论上零变化）。
4. e2e 不需要重跑（无用户可见行为变化；文档 + 部署资产）。

## 门禁记录

- grill-with-docs：未完整调用（环境未暴露 Skill）；目录划分与交付物由 owner 直接拍板
- BDD：skipped——纯部署资产 + 文档，无用户可见行为变化
- Refactoring Review：proceed（目录移动 + 新增文件，不改既有代码）
- Legacy Change Safety：not-required（不修既有行为）
- DDIA：confirmed——无 schema/迁移；compose 卷的持久化语义（pgdata 命名卷）与回滚路径（checkout + rebuild）明确
- Release Readiness：planned——部署资产本身就是发布路径，收尾前跑正式 review
