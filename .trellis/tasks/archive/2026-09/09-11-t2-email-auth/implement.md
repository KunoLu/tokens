# T2 实施

**依赖**：T0、T1（均已完成）  
**推进清单**：`docs/TODO.md` T2 节  
**设计**：本目录 `design.md`  
**父清单**：`.trellis/tasks/09-10-teamboard-teams-auth/implement.md` T2 节

未完整调用 `grill-with-docs`：父 PRD / D-1–D-4 / INV 已确认；本切片无新领域决策。可选 cron 跳过空 social_links **默认不实施**。

## Book Gate Plan

| Skill | 触发 | 阶段 | Gate state |
|---|---|---|---|
| book-refactoring-pass | required：改既有生产代码 | 首次实现编辑前 | planned |
| book-legacy-change-safety | required：认证改写、web 无单测、回归面大 | 首次行为修改前 | planned |
| book-ddia-data-design | required：`0024` 迁移、email 唯一、token 表 | 设计已在本文件前锁定 | planned → 实现前跑 reviewer |
| book-ddd-distilled-modeling | on-demand：无新术语/边界 | — | not-required |
| book-release-readiness | required：auth / email / cron / secrets | 验证之后、完成前 | planned |

## BDD

- 语言：中文场景 + 英文 Gherkin 关键词；无 `# language: zh-CN`
- 路径：`features/email-auth.feature`（仓库无既有 `.feature`，第一份默认）
- 无 Gherkin runner / web 单测：场景是行为 SOT；自动化靠 lint / typecheck / build / test:migrations + grep
- CLI device flow 场景 `@todo`，本切片不改 Rust

## 步骤

0. 落盘 `features/email-auth.feature`（若主会话已写则勿覆盖行为）。
1. 迁移 `0024_add_password_auth.sql`（DDL = PRD §7.1）+ journal idx 24 + **手写** `meta/0024_snapshot.json`（从 0023 + SQL 差量）。**禁止** live 目录 `bun run db:generate`。
2. `schema.ts`：`passwordHash`、`emailVerifiedAt`、`githubId` 可空、email 部分唯一索引、`emailVerificationTokens` 表。社交链接注释去掉「verified 徽章」。
3. `web/scripts/check-migrations.ts`：required tables 加 `email_verification_tokens`；断言新列、`github_id` 可空、`users_email_lower_unique`。
4. `web/src/lib/auth/password.ts`：WebCrypto PBKDF2 210k，格式 `pbkdf2$sha256$210000$…`；`validatePassword` 规则与注册/重置共用。
5. `web/src/lib/email/`：Resend `fetch`，失败 `waitUntil` / 缺 secret 只打日志，不回滚。
6. `POST /api/auth/{register,login,forgot-password,reset-password,verify-email}`。变更类 `hasAllowedOrigin`。Rate limit binding 见 design §5。
7. 页面 `/register` `/login` `/forgot-password` `/reset-password` `/verify-email`：`CONTAINER` + `PageHeader` + 现有 Input/Button。`returnTo` 用 `sanitizeAuthReturnTo`。
8. **删除** `lib/auth/github.ts`、`api/auth/github/`、`api/auth/github/callback/`。
9. 改写入口：`Navigation.tsx`（删仓库 GitHub 图标 + Sign in 去 `/login` + 头像 fallback）、`DeviceClient.tsx`、`profile/page.tsx`、`SettingsClient.tsx`、`middleware.ts`。
10. `web/src/lib/avatar.ts`：`avatarUrlFor`；替换所有 `github.com/${username}.png`。
11. 删 Profile 页 GitHub 按钮。
12. Docs：删整个 verified Section；CLI note `link your GitHub account` → `sign in`。Privacy / Terms / Settings 去掉「用 GitHub OAuth 登录」表述（见 design §7）。
13. cron：social links 刷新保留，去掉 `verified` 计数；追加清理过期 `email_verification_tokens`。邀请表不存在，不要写邀请清理。
14. wrangler：注释 secrets；`ratelimits` AUTH_RATE_LIMITER namespace 1001，10/60s。本地无 binding 则跳过。
15. **移除徽章**（父 implement 第 11 步全文，含 `web/worker.ts`）。
16. **不可触碰**：`session.ts` 签名、device flow 三端点、`personalTokens.ts`、`/api/submit`、`/api/me/stats`、CLI。
17. **重发验证邮件**（父 design §7）：`POST /api/auth/resend-verification` + `/verify-email` 失败态按钮 + 登录页入口。Forgot-password 不能代替。
18. **不实施**：cron 只同步非空 social_links；T5 shadcn；邀请回填。

## 验收

- 可注册登录；弱密码被拒；忘记密码恒 200
- 已登录用户可重发验证邮件；未登录 401；forgot-password 不写 `email_verified_at`
- `tokens login` + `tokens submit` 契约仍通（device flow 未改）
- OAuth 路由 404；导航无 GitHub 登录图标；Profile 无 GitHub 按钮
- `rg -n "VerifiedBadge|socialVerification|verifiedExpr" web/src web/worker.ts` 无命中
- `rg -n "email_verified_at|emailVerifiedAt" web/src` 仅 schema / 验证流程
- `githubSocials.ts`、`isValidGitHubUsername`、`users.github_id` 仍在
- `bun run lint`、`bun run typecheck`、`bun run build`、`bun run test:migrations` 通过
- 不新增 npm 依赖

## 验证命令

在 `web/`：

```bash
bun run lint
bun run typecheck
bun run build
bun run test:migrations
```

`rtk`：lint/typecheck/build 可用 rtk；`test:migrations` 有落地报告语义则 `skipped-for-report` 用原生命令。

Web E2E / Playwright：本切片 `not-needed`（无既有 Playwright 套件覆盖认证；不新装）。CLI 全链路若无本地 credentials 标 `blocked`，不假装跑过。
