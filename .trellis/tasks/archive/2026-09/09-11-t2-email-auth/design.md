# T2 技术设计：邮箱密码认证

> 父任务：`.trellis/tasks/09-10-teamboard-teams-auth/`
> 数据 DDL / API 表：`docs/prd-teamboard-teams-auth.md` §7.1、§8、§9.1
> 本文件只锁 T2 实现决策。Team / Group、邀请回填、T5 shadcn 增补不在本切片。

## 边界

**做**

- `0024_add_password_auth.sql` + Drizzle schema + journal 尾 snapshot
- 邮箱注册 / 登录 / 忘记密码 / 重置密码 / 验证邮箱
- 删除 GitHub OAuth 全链路与全部 GitHub **登录入口**
- 头像 fallback 改为用户名首字母占位图
- 移除 `verified` 徽章（D-3 取 C）
- Docs 删除 verified 节；Privacy / Terms / Settings / Docs CLI 文案里「用 GitHub 登录」改为邮箱账号
- cron 追加清理过期 `email_verification_tokens`；social links 刷新任务保留，去掉 `verified` 计数
- wrangler 声明 `RESEND_API_KEY` / `EMAIL_FROM`（secret，不写值）+ Rate Limiting binding

**不做**

- 不改 `lib/auth/session.ts` 签名与行为、device flow 三端点 JSON、`personalTokens.ts`、`/api/submit`、`/api/me/stats`、Rust CLI
- 不建 `team_invitations`、不做注册后邀请回填（T3 / T4）
- 不实施「cron 只同步 `social_links` 已非空账号」（父 implement 默认不实施）
- 不新增 npm 依赖（无 Resend SDK、无 bcrypt、无 next-intl）
- 不补 T5 的 select / dialog / checkbox / popover / form / label / textarea
- 不删 `users.social_links` / `social_links_synced_at`、不删 `githubSocials.ts`、不删 `ProfileSocialLinks.tsx`
- 不把 `/login` 等认证页加入 `worker.ts` `PAGE_CACHEABLE`

## 1. 密码

Workers 上 bcrypt / argon2 不可用。用 **WebCrypto PBKDF2-HMAC-SHA256**，210,000 次，每用户 16 字节随机 salt。

存储：`pbkdf2$sha256$210000$<salt_b64>$<hash_b64>`（约 96 字符，`varchar(255)` 足够）。

实现：`web/src/lib/auth/password.ts`

- `hashPassword(plain)` / `verifyPassword(plain, stored)`
- 派生后用已有的 `node:crypto` `timingSafeEqual` 比较
- 未知算法前缀 → 校验失败，不抛

密码规则（注册与重置同一函数）：≥ 8 位，且同时含大写、小写、至少 1 个特殊字符。失败返回 `{ error, details: string[] }`。

## 2. 会话层不动

`tt_session` 签发 / 存储 / 校验 / 30 天 cookie 完全保留。T2 只替换「如何确认身份」：邮箱 + 密码 → `createSession` / `setSessionCookie`。

因此 device flow 与 CLI 零改动。

登录封禁：不建会话。Web 表单与 API 返回 403 `{ error: "Account banned" }`（与错误密码区分，避免把封禁账号锁在「密码错误」死循环；OAuth 时代是重定向到 `/u/{username}`，邮箱表单没有 OAuth callback，用 JSON 状态码）。

未验证邮箱：**可以登录并持有会话**（PRD：注册成功即建会话并发送验证邮件）。`email_verified_at` 只记录验证完成时间。

存量行：`github_id` 可空只是保留旧行，**不是认证路径**。没有 GitHub 登录。`password_hash` 为 null 的登录与错误密码一样 401。有邮箱的存量账号只能通过忘记密码**设置密码**后再用邮箱+密码登录；无邮箱的存量账号本切片不提供迁移 UI。

## 3. 迁移 `0024`

按 PRD §7.1 手写 SQL，additive：

- `users.password_hash varchar(255)` 可空
- `users.email_verified_at timestamptz` 可空
- `users.github_id` `DROP NOT NULL`（UNIQUE 保留；Postgres 允许多个 NULL）
- 部分唯一索引 `users_email_lower_unique` ON `lower(email) WHERE email IS NOT NULL`
- 表 `email_verification_tokens`（`purpose` ∈ `verify_email` | `reset_password`；`token_hash` UNIQUE SHA-256 hex 64）

旧行不动。历史上邮箱为空不阻塞；若生产已有重复非空 email，迁移会失败——先在 apply 前用 `SELECT lower(email), count(*) FROM users WHERE email IS NOT NULL GROUP BY 1 HAVING count(*) > 1` 确认，有重复则停，不静默去重。

`check-migrations.ts`：把 `email_verification_tokens` 加入 required tables；断言 `password_hash` / `email_verified_at` 存在、`github_id` 可空、`users_email_lower_unique` 为 UNIQUE 且含 `lower((email)::text)`。代表 insert 仍可只插 `github_id` + `username`（两列仍够）。

**Snapshot**：从 `0023` snapshot + 本 SQL 差量手写 `meta/0024_snapshot.json`。**禁止**在 live migrations 目录跑 `bun run db:generate`（LESSON-20260911-640-migration-checker-dropped-tables）。journal `when` 必须严格大于 0023。

## 4. 邮件

`web/src/lib/email/send.ts`：`fetch("https://api.resend.com/emails")`，Bearer `RESEND_API_KEY`，`from: EMAIL_FROM`。不引 SDK。

发送放在 `waitUntil`（Workers）或 `void promise`（本地）。失败只 `console.error`，**不回滚**已提交的用户/token 行。缺 secret 时同样只记日志，注册/忘记密码仍 200。

明文 token 只出现在邮件链接里：`${NEXT_PUBLIC_URL}/verify-email?token=` / `/reset-password?token=`。库中只存 `hashToken`（现有 SHA-256 hex）。

有效期：验证邮件 24h，重置 1h。同 user+purpose 新发时把未消费旧行的 `consumed_at` 置 now（单活跃 token）。

忘记密码：**无论邮箱是否存在都 200** `{ ok: true }`，防枚举。

重置成功：更新 `password_hash`，消费 token，**删除该用户全部 `sessions` 行**（不碰 `api_tokens`）。

## 5. API

路径见 PRD §9.1。惯例：`try/catch` + `{ error }` / `{ error, details }`。

变更类（POST）走 `hasAllowedOrigin`；缺 Origin 或 Origin 不在白名单 → 403。注册/登录/忘记密码**不**要求已有 session。

| 路由 | 行为 |
|---|---|
| `POST /api/auth/register` | body `{ email, username, password }`。username 沿用 `isValidGitHubUsername`（1–39，`[A-Za-z0-9-]`）。email `lower` 后查重。成功 insert（`github_id` null）、`createSession`、写 verify token、异步发信。冲突 409。 |
| `POST /api/auth/login` | `{ email, password }`。无用户 / 无 password_hash / 校验失败 → 统一 401 `{ error: "Invalid email or password" }`。封禁 403。成功建会话。 |
| `POST /api/auth/forgot-password` | `{ email }`。命中且未封禁才写 reset token 并发信。响应恒 200。 |
| `POST /api/auth/reset-password` | `{ token, password }`。无效/过期/已消费 400。成功后吊销全部 web session。 |
| `POST /api/auth/verify-email` | `{ token }`。无效 400。成功写 `email_verified_at`（已验证则幂等 200）。 |
| `POST /api/auth/resend-verification` | **需要会话**。已验证 → 200 不发信；未验证 → 再发 `verify_email` token。未登录 401。Forgot-password 不能代替本路径。 |
| `GET/POST /api/auth/github*` | **删除** |

`/api/auth/session`、`/logout`、`/token`、`/device/*` 契约不变。

Rate limit：wrangler

```jsonc
"ratelimits": [{
  "name": "AUTH_RATE_LIMITER",
  "namespace_id": "1001",
  "simple": { "limit": 10, "period": 60 }
}]
```

作用于 register / login / forgot-password / resend-verification。key = 客户端 IP（`CF-Connecting-IP` 否则 `0.0.0.0`）。超限 429。`getCloudflareContext().env.AUTH_RATE_LIMITER` 在本地 next dev 可能不存在 → **跳过限制**，不虚构计数表。

Secrets 只在 `wrangler.jsonc` 用注释标明 `wrangler secret put RESEND_API_KEY` / `EMAIL_FROM`，不写值。

## 6. 页面与入口改写

新页（`(main)` 外、无榜单壳，用 `CONTAINER` + `PageHeader` + 现有 `Input`/`Button`）：

- `/register`、`/login`、`/forgot-password`、`/reset-password`、`/verify-email`
- 客户端表单 POST 上述 API；`returnTo` 一律 `sanitizeAuthReturnTo`
- 登录页链到注册 / 忘记密码 / 重发验证；`/verify-email` 在失败态对已登录用户提供 Resend

入口替换（全部指向 `/login?returnTo=…`，不再打 `/api/auth/github`）：

- `Navigation.tsx`：删仓库 GitHub 图标按钮；Sign in 去 `/login`；删 `GitHubIcon` 若无其它引用
- `middleware.ts`：`/settings` 未登录 → `/login?returnTo=`
- `DeviceClient.tsx`：文案改为 Sign in，链 `/login?returnTo=/device`
- `profile/page.tsx`、`SettingsClient.tsx` 未登录跳转

头像：新增 `web/src/lib/avatar.ts` 的 `avatarUrlFor({ username, avatarUrl })`——有 `avatarUrl` 用原值，否则返回首字母 SVG data URI（两字符，与现 `AvatarFallback` 一致）。替换 Navigation / Leaderboard / ProfileView / SettingsClient 里的 `github.com/${username}.png`。

Profile 页右上角 **GitHub 按钮整颗删除**（含 `GitHubMark`，若只服务该按钮）。

## 7. 去掉 GitHub 登录 ≠ 清掉所有 `github` 字符串

完成判据里 `rg -i github web/src`「仅剩文档性文案」与**保留** `githubSocials.ts` / `isValidGitHubUsername` / Profile 社交链接冲突。本切片以父 implement 第 6–8、11 步清单为准：

**必须消失**：OAuth 路由与 `lib/auth/github.ts`、导航 GitHub 图标、Sign in 的 GitHub 图标、Profile GitHub 按钮、`github.com/<user>.png` fallback、Docs/Privacy/Terms/Settings 中「用 GitHub 登录 / OAuth」表述、verified 徽章。

**保留（允许 grep 命中）**：`githubSocials.ts`、`isValidGitHubUsername`（用户名格式，不改名）、embed「copy into GitHub」、社交链接 provider 名、`users.github_id` 列。

Docs：删除整个 verified `<Section id="verified">`（T1 已去掉该节 `/shame` 链，整节留给 T2）。CLI 安装步骤 `link your GitHub account` → `sign in`。

Privacy / Terms：认证改为邮箱注册；删除「From GitHub when you sign in / OAuth scopes」作为登录方式的段落。社交链接仍可由公开 GitHub 资料刷新的，保留为可选资料来源，不要写成登录依赖。

Settings：`Profile information is synced from GitHub and cannot be edited here` 改为本地账号资料不可在此编辑（本切片不开放改用户名/邮箱表单）。

## 8. 移除 verified 徽章

按父 implement T2 第 11 步逐文件，**含 `web/worker.ts`**（不在 `web/src` 下）。

cron：`refreshAllSocialLinks` 返回值收敛为 `{ users }`；同一 `scheduled` / HTTP cron 在刷新之后 **再** `DELETE`（或更新 `consumed_at`）`email_verification_tokens WHERE expires_at < now()`。不碰不存在的邀请表。

`schema.ts` / `githubSocials.ts` 里「>= 2 entries marks verified」注释改为社交链接快照用途，不提徽章。

可选优化（只同步已有 social_links）**不实施**。

## 9. BDD

仓库无既有 `.feature`。语言：中文场景 + 英文 Gherkin 关键词，无 `# language: zh-CN`。路径：`features/email-auth.feature`。

无 Gherkin runner、无 web 单测：场景是行为 SOT；自动化追踪靠 `bun run typecheck` / `build`、迁移检查、以及完成判据里的 grep。CLI device flow 场景标 `@todo`，手工/后续用真 CLI 验，本切片不改 Rust。

## 10. 风险

| 风险 | 处理 |
|---|---|
| 漏改 `worker.ts` verified 解构 | typecheck/build 会炸；grep 必须带上该文件 |
| `db:generate` 再吐已应用 DDL | 手写 0024 snapshot，禁止 live generate |
| Rate limit namespace 未在账号创建 | wrangler 声明即可；本地无 binding 则跳过 |
| 存量重复 email | 迁移前查询；有重复则停 |
| 缺 Resend secret | 注册仍成功；已登录用户可在 `/verify-email` 重发。发信失败不回滚 |
