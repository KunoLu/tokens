# PRD：Teamboard、团队体系与邮箱认证改造

| 项目 | 内容 |
|---|---|
| 文档状态 | 已确认（2026-09-11） |
| 目标仓库 | `KunoLu/tokens`（fork 自 `missuo/tokens`，其上游为 `junhoyeo/tokscale`） |
| 实施分支 | `feature/teamboard-teams-auth` |
| Trellis 父任务 | `.trellis/tasks/09-10-teamboard-teams-auth/` |
| UI 验收物 | `docs/demo/teamboard-demo.html` |
| 关联文档 | `docs/upstream_policy.md`（同步策略，本次一并改写） |

---

## 1. 背景与改造动机

本仓库当前是一个**面向个人开发者的全局单一排行榜**：CLI 采集各 AI 编码客户端的 token 用量，提交到 Web 端，按个人聚合成一张全站榜。它没有任何组织概念——上游 `tokscale` 曾有的 group 排行榜，在 `missuo/tokens` 的重构中被明确移除（迁移 `0020_drop_group_tables.sql`），换成"一张全局榜"。

本次改造要把产品从"个人榜"扩展为"个人榜 + 组织榜"，并把身份体系从"依附 GitHub"改为"自有账号"。这两件事互相牵连：只有先有自有账号（邮箱 + 用户名），"按用户名或邮箱邀请成员"才成立。

同时，`Hall of Shame`（作弊者公示页）被移除，其导航位让给 `Teamboard`。

---

## 2. 改造前的技术栈与架构现状

### 2.1 仓库结构

```
tokens/
├── cli/                    Rust workspace，tokens-cli（仅 login/submit/serve/status + 各 provider 同步）
├── web/                    Next.js 应用（本次改造的全部范围）
├── packages/               9 个 npm 分发包（cli 主包 + 8 个平台二进制包）
├── docs/upstream_policy.md 上游同步策略
└── scripts/
```

### 2.2 Web 技术栈（改造前）

| 层 | 技术 | 版本 / 说明 |
|---|---|---|
| 框架 | Next.js App Router | `16.0.10`，React `19.2.0`，TypeScript 5 |
| 运行时 | Cloudflare Workers | 经 `@opennextjs/cloudflare` `1.20.2`；`compatibility_date 2026-07-24`；flags `nodejs_compat`、`global_fetch_strictly_public` |
| 数据库 | Neon Postgres | 经 Cloudflare Hyperdrive 绑定 `HYPERDRIVE`；Worker 定点 `aws:us-west-2` |
| ORM | Drizzle ORM `0.38.3` + `postgres.js 3.4.7` | Workers 侧 `max=3, prepare=false`，每请求 `WeakMap<ctx, db>` 单例 |
| 缓存 | R2（渲染页）+ Durable Objects（tag 失效、队列、purge） | `unstable_cache` revalidate 60s，tag `leaderboard` / `user-rank` |
| 样式 | Tailwind CSS v4 | CSS-first，无 `tailwind.config`，语义 token 定义在 `web/src/app/globals.css` |
| 组件 | shadcn/ui（vendored 到 `components/ui/`） | 现有 16 个；另有 `@base-ui/react`、Radix Tooltip、`lucide-react`、`next-themes` |
| 校验 | zod `3.24.1` | 集中在 `web/src/lib/validation/` |
| 通知 | react-toastify `11` | 经 `ThemedToastContainer` |
| 字体 | Geist + JetBrains Mono | `next/font`，数字统一用 `.tabular` 类 |
| 定时任务 | Wrangler cron `20 3 * * *` | 刷新 GitHub social links |
| CI | GitHub Actions | Rust：`cargo clippy --all-targets -- -D warnings`、`cargo build --release`、`--version` smoke、**`cargo test --manifest-path cli/Cargo.toml`**；Web：`bun run lint`、`wrangler types`、`tsc --noEmit`、`bun run build`。**Rust 侧有内联测试且 CI 会运行（`cli/` 下 91 个文件含 `#[cfg(test)]`）；web 侧无单元测试**，另有 `test:migrations` 迁移集成检查（未接入 CI） |

### 2.3 认证现状（改造前）

- **Web 登录**：仅 GitHub OAuth。`/api/auth/github` 写 `oauth_state` cookie（10 分钟）→ GitHub authorize（scope `read:user user:email`）→ callback 校验 state → 按 `github_id` upsert 用户 → 建会话。
- **会话**：非 JWT。64 位 hex 明文写入 `tt_session` cookie（httpOnly、生产 secure、sameSite lax、30 天），数据库只存 SHA-256。
- **CLI 认证**：OAuth 2.0 Device Flow。`POST /api/auth/device` → 浏览器 `/device` 输入 user code → `POST /api/auth/device/authorize` → CLI 轮询 `poll` 拿到 `tt_` 前缀的 personal API token，存 `credentials.json`（0600）。
- **API token**：`tt_` + 48 hex，库里存 SHA-256，经 `Authorization: Bearer` 使用。
- **权限**：**没有任何角色概念**。唯一的权限维度是 `users.banned_at` 封禁。
- **`verified` 徽章**：来源是 GitHub 社交链接数 ≥ 2（`social_links` jsonb），每日 cron 刷新。**本次按决策 D-3（选项 C）整体移除该徽章**；Profile 页的社交链接图标行保留。
- **不存在的能力**：无密码哈希库、无任何邮件发送能力。

### 2.4 数据模型现状（9 张活跃表）

`users`、`sessions`、`api_tokens`、`device_codes`、`submissions`、`submitted_devices`、`daily_breakdown`、`archived_breakdown`、`archived_window_totals`。

与本次改造直接相关的约束：

- `users.github_id` — `integer NOT NULL UNIQUE`（**改造必须放宽**）
- `users.username` — `varchar(39) NOT NULL UNIQUE`，另有 `lower(username)` 唯一索引
- `users.email` — `varchar(255)` **可空、无唯一约束**（改造需收紧）
- `users` **无 `password_hash`、无 `email_verified_at`**
- `submissions.user_id` **UNIQUE**——每用户仅一行汇总，团队级聚合不能复用该模型
- 迁移共 24 条（idx 0–23），惯例为 additive、uuid 主键、snake_case 列名、camelCase Drizzle 字段、敏感 token 只存 hash

### 2.5 现有页面与线上 UI 事实

经线上 `https://tokens.ci/` 核对：

- 导航（高 56px，容器 `max-w-[1200px]`，左右 24px）：`Tokens` 品牌 → `Leaderboard` / `Hall of Shame` / `Docs`（登录后多一个 `Profile`）→ 右侧 `GitHub 图标` / `主题切换` / `Sign in`
- Leaderboard：标题区 → 分隔线 → 三张统计卡（Tokens / Cost / Developers，登录后加 `Your rank`）→ 控件行（Period 五选一、Sort by 二选一、搜索框）→ 表格（`#` / `Developer` / `Tokens` / `Cost`）→ 分页
- `Tokens`、`Cost` 两个表头是"缩写/精确数字"切换按钮，**不是排序按钮**；排序由控件行的 `Sort by` 负责。这一点在本次加列时必须保留原语义。
- Profile 实为 `/u/[username]`，`/profile` 只做重定向；右上角有 `Embed` / `Share` / `GitHub` 三个操作。

### 2.6 现存缺陷（本次必须顺带修复）

1. `web/scripts/check-migrations.ts` 仍断言 `groups` / `group_members` / `group_invites` 三张表存在，而它们已在 `0020` 被删除——`test:migrations` 与现实矛盾。
2. `web/src/lib/db/migrations/meta/` 最新 snapshot 停在 `0021`，journal 已到 `23`。不补齐会导致 `db:generate` 产出错误的 diff。

---

## 3. 目标与非目标

### 3.1 目标

1. 删除 Hall of Shame 页面功能。
2. 在其导航位新增 `Teamboard` 页面。
3. 新增 Team / Group 两级组织体系，含 admin / subadmin 角色、邀请、解散与删除的两段式生命周期。
4. 认证体系由 GitHub OAuth 改为邮箱注册登录，并移除所有 GitHub 入口。
5. Leaderboard 增加 `Team`、`Group` 两列；Teamboard 提供团队单选 + 分组多选筛选。
6. Profile 展示 Team / Group 并支持退出。
7. 其余功能保持现状。
8. 品牌图标蓝色底色改为紫色（白色 T 图案不变）。**注**：这是与本次重构无关的独立小改动，应单独提交。

### 3.2 非目标（本次明确不做）

- 不改 Rust CLI 的任何代码。device flow、`tt_` token 格式、`credentials.json`、`TOKENS_API_TOKEN` / `TOKENS_API_URL` 全部保持契约不变。
- 不改 `submissions` / `daily_breakdown` 的采集与聚合语义。
- 不引入团队级的用量提交（团队数字始终由成员个人数据聚合而来）。
- 不做 SSO、不做组织付费、不做跨 Team 的成员共享。

---

## 4. 领域模型

### 4.1 实体与关系

```
User ──1:0..1── TeamMember ──N:1── Team
                                    │
                                    ├──1:N── Group
                                    │           │
User ──1:0..1── GroupMember ────────┴───────────┘

Team ──1:N── TeamInvitation
```

### 4.2 不变式（实现必须逐条保证）

| # | 不变式 | 强制位置 |
|---|---|---|
| INV-1 | 一个 User 至多属于一个 Team | `team_members.user_id` UNIQUE |
| INV-2 | Group 必属于某 Team；GroupMember 必须先是该 Team 的成员 | 外键 + 应用层校验 + 一致性查询 |
| INV-3 | 一个 User 在其 Team 内至多属于一个 Group | `group_members.user_id` UNIQUE |
| INV-4 | `status='active'` 的 Team 恰有一个 admin | 创建时写入；仅"移交"可改变持有者；解散时随成员表一并清空 |
| INV-5 | 每个 Team 至多 2 个 subadmin | 原子条件 UPDATE / 行锁 |
| INV-6 | `Delete` 仅在 `status='disbanded'` 且成员数 = 0 时允许 | 应用层前置校验 |
| INV-7 | `Disband` 清空全部成员行（含 admin 行）；`disbanded` 的 Team 没有任何成员、也没有 admin，`created_by` 是唯一的删除授权主体 | 事务内执行 |
| INV-8 | 退出 Team 时自动退出其 Group | 同事务级联 |
| INV-9 | 封禁用户不出现在任何榜单 | 沿用 `isNull(users.bannedAt)` |
| INV-10 | 非成员只能浏览 `visibility='public'` 且 `status='active'` 的 Team；成员始终可浏览自己的 Team，无论其可见性 | `teams.visibility` + 查询层鉴权分支 |

> **INV-4 与 INV-7 的边界**：需求要求"解散会把所有成员踢出"，又要求"成员为 0 才能删除"。若 admin 也被踢出，就没人有权删除了。解法是把"成员身份"与"创建者身份"分离——`created_by` 不随解散清除，已解散的 Team 仍可由创建者删除。
>
> 因此 INV-4 只对 `active` 的 Team 成立。`disbanded` 的 Team 处于**无成员、无 admin** 的终态，此时任何基于 `team_members.role` 的鉴权都会返回"无权限"，删除权必须改读 `teams.created_by`。实现时不要写成"查 admin 再校验"，否则解散后的删除必然失败。

### 4.3 状态机

```
Team:   active ──disband──> disbanded ──delete──> (物理删除)
                     ▲                    │
                     └────── 不可逆 ───────┘
Group:  同上（作用域限于所属 Team）

Invitation: pending ──accept──> accepted
                    ├─decline─> declined
                    ├─revoke──> revoked
                    └─expire──> expired
```

解散不可撤销。这是刻意的：可撤销的解散会让"成员为 0"这个删除前置条件失去意义。

---

## 5. 权限矩阵

创建 Team 的用户自动成为该 Team 的 admin，同时被写入 `teams.created_by`。这两者会在解散时分道扬镳：admin 成员行被清空，`created_by` 永久保留。

| 操作 | admin | subadmin | member | 说明 |
|---|:---:|:---:|:---:|---|
| 修改 Team 名称 / 头像 | ✅ | ✅ | ❌ | |
| 修改 Team 可见性（public / private） | ✅ | ✅ | ❌ | 依「subadmin 除解散与删除外权限等同 admin」的规则 |
| 邀请成员（单个 / 批量） | ✅ | ✅ | ❌ | 按用户名或邮箱 |
| 撤销待处理邀请 | ✅ | ✅ | ❌ | |
| 移除成员 | ✅ | ✅ | ❌ | subadmin 不能移除 admin |
| 指派 / 撤销 subadmin | ✅ | ❌ | ❌ | 上限 2 |
| 移交 admin | ✅ | ❌ | ❌ | 移交后原 admin 降为 member |
| **解散 Team** | ✅ | ❌ | ❌ | 需求明确排除 subadmin |
| **删除 Team** | ✅ ※ | ❌ | ❌ | 需求明确排除 subadmin；需已解散且成员 0 |
| 创建 Group | ✅ | ✅ | ❌ | |
| 修改 Group 名称 | ✅ | ✅ | ❌ | |
| 分配 / 移出 Group 成员 | ✅ | ✅ | ❌ | |
| 解散 Group | ✅ | ✅ | ❌ | subadmin 对 group 权限与 admin 相同 |
| 删除 Group | ✅ | ✅ | ❌ | 需已解散且成员 0 |
| 自行退出 Group | ✅ | ✅ | ✅ | Profile 页 |
| 自行退出 Team | ⚠️ | ✅ | ✅ | admin 须先移交或解散 |

※ **删除 Team 的授权主体是 `teams.created_by`，不是 admin 角色行。** 按 INV-7，可删除的前提是团队已解散，而解散已清空全部成员行——包括 admin 自己。矩阵里把它标在 admin 列，是因为正常生命周期下创建者就是那个 admin；但代码里必须校验"调用者 == `created_by`"，而非"调用者在 `team_members` 中 role = admin"。

---

## 6. 功能需求

### FR-1 删除 Hall of Shame

**删除**：`web/src/app/(main)/shame/page.tsx`、`web/src/components/shame/BannedList.tsx`。

**改写引用**：
- `Navigation.tsx` 的 `NAV_LINKS` 移除该项
- `web/src/app/u/[username]/BannedProfileView.tsx` 中指向 `/shame` 的链接
- `web/src/app/(main)/docs/page.tsx`、`terms/page.tsx` 中的 Hall of Shame 链接
- `web/worker.ts` 的 `PAGE_CACHEABLE` 列表
- `web/src/app/error.tsx`、`schema.ts` 中的相关注释

**保留**：封禁机制本身（`banned_at` / `ban_reason`、榜单过滤、`BannedProfileView`）。封禁仍然生效，只是不再有公示页。原 OAuth callback 中"封禁用户重定向到 `/shame`"的分支，改为重定向到登录页并带错误码。

### FR-2 新增 Teamboard

新增 `/teamboard`，镜像 Leaderboard 的整体布局。

- 导航中占据原 Hall of Shame 的位置，标题为 `Teamboard`
- 新增筛选项：**Team 单选**（一次只展示一个团队）、**Group 多选**（可同时展示多个分组的成员）
- 列：`#` / `Developer` / `Group` / `Tokens` / `Cost`——**不含 Team 列**，因为已经按单个 Team 过滤
- 保留 Period 五选一、Sort by、搜索、分页，与 Leaderboard 一致
- Team 筛选器的候选项 = **全部 `public` 且未解散的团队** ∪ **当前用户自己所属的团队**（自己的团队即使是 `private` 也始终可见）
- 登录用户默认选中自己所属的团队；未登录访客默认未选中，展示引导空态，可自行选择任意公开团队
- 对 `private` 且非本人所属的团队，筛选器不列出，直接请求其 `teamId` 返回 404（与 §9.3 一致：403 会确认团队存在，可被用来枚举）

### FR-3 Team / Group 管理

新增 `/teams` 页面（导航中登录可见）。

**Team 操作**：创建、改名、改头像、邀请成员（用户名或邮箱，支持批量）、移除成员、指派 subadmin（≤2）、移交 admin、解散、删除。

**Group 操作**：在 Team 下创建、改名、分配成员、移出成员、解散、删除。点击某个 Group 时，成员列表按该 Group 递进筛选。

**邀请**：
- **受邀人选择**：可搜索的下拉多选列表（combobox）。输入内容实时筛选匹配的已注册账户（**按用户名 / 显示名**，不按邮箱子串），每个选项左侧有勾选框，选中即计数并可以再次点击取消。**下拉只展示用户名与显示名，不回显绑定邮箱**。输入一个完整邮箱地址时，服务端按邮箱**精确匹配**：命中已注册用户则按用户名 / 显示名展示该用户；未匹配到任何账户时列表底部出现「邀请该邮箱」选项——未注册用户的邮箱邀请路径因此保留
- 候选数据来自新增的 `GET /api/users/search`（见 §9.2），仅在发起者具有某个团队的 admin / subadmin 角色时可用，避免任意用户枚举全站账户
- 已注册用户：站内待处理邀请 + 邮件通知
- 未注册邮箱：邮件邀请，注册后自动关联该邀请
- **归一化**：提交时在同一事务内把每条输入解析为具体的人——用户名必须能查到已注册用户，否则该条报错；邮箱若能查到已注册用户，也一并写入 `invited_user_id`。只有查不到用户的邮箱才保留 `invited_user_id = NULL`，只存 `invited_email`
- **去重**：同一 Team 对同一「人」的 `pending` 邀请唯一。已注册受邀人由 `(team_id, invited_user_id)` 部分唯一索引保证，未注册地址由 `(team_id, lower(invited_email))` 部分唯一索引保证。因为两个索引的 `WHERE` 条件互斥（前者要求 `invited_user_id IS NOT NULL`，后者要求 `IS NULL`），**用用户名和邮箱分别邀请同一个已注册用户只会落到同一行**，批量邀请可以安全重放
- **注册时回填**：账号提交成功后，用一个独立事务把该邮箱的 pending 邀请归一到新用户身上。**四个步骤有严格顺序，颠倒任意两步都会先撞 `pending_user_unique`**：
  1. 按 Team 分组，`SELECT ... FOR UPDATE` 锁定该 Team 下所有指向「这个邮箱」或「这个用户」的 pending 邀请。受两个互斥的部分唯一索引约束，每个 Team 最多锁到 2 行——一行邮箱行、一行用户行
  2. 定 winner：`created_at` 最早的一行胜出，完全并列时取 `id` 较小者。**不预设邮箱行更早还是更晚**，两种情况都可能发生
  3. 先把 loser 置为 `superseded` 并写 `responded_at`
  4. 再回填 winner。若 winner 是邮箱行，此时 `(team_id, invited_user_id)` 已无 pending 占用者，UPDATE 才不会冲突；若 winner 本就是用户行，则邮箱行已被置为 `superseded`，无需回填
- **回填只是归一化，不是正确性前提**：`GET /api/me/invitations` 同时匹配 `invited_user_id = 我` 与 `invited_user_id IS NULL AND lower(invited_email) = lower(我的邮箱)`。因此回填延迟或失败都不会让用户看不到邀请，重试也天然幂等；也正因如此，回填**不放进注册事务**，它失败不应该让注册失败
- 上述冲突只在并发提交、或日后引入换邮箱功能时才可能出现（新注册用户理论上不会已有用户行）。但**不要依赖这个前提写代码**——一旦成立就是一个 500，而按上述顺序实现的成本几乎为零
- 有效期 7 天

**危险操作**：解散与删除都需要输入 Team / Group 名称二次确认。

### FR-4 邮箱认证体系

**注册**必填三项：
- 邮箱地址（唯一，大小写不敏感）
- 用户名（唯一，沿用现有 39 字符与 `lower(username)` 唯一索引约束）
- 密码——**≥ 8 位，且同时包含大写字母、小写字母、至少 1 个特殊字符**

**移除**：GitHub OAuth 全链路（`lib/auth/github.ts`、`api/auth/github/`、`api/auth/github/callback/`）、导航右上角 GitHub 图标、Profile 页右上角 GitHub 按钮、`verified` 徽章全链路（决策 D-3 取 C，逐文件清单见 §13）。

**保持不变**：会话机制（`tt_session` cookie + DB 存 hash + 30 天）、device flow 三个端点的 JSON 契约、`tt_` token 格式与 Bearer 用法、`/api/submit`、`/api/me/stats`。**CLI 完全不需要改动。**

**新增页面**：`/register`、`/login`、`/forgot-password`、`/reset-password`、`/verify-email`。

**头像 fallback 必须一并改造**：`Navigation.tsx` 的 `avatarFor()` 当前在 `avatarUrl` 为空时回落到 `https://github.com/${username}.png`。GitHub 下线后这个地址对新用户一律 404，必须改为按用户名生成的首字母占位图（Demo 中已采用该形态）。

### FR-5 榜单列扩展

**Leaderboard** 列顺序：`#` / `Developer` / **`Team`** / **`Group`** / `Tokens` / `Cost`。两个新列在 `Developer` 之后，值为空时留空。`Tokens` / `Cost` 的既有排序与数字格式切换行为不变。

**Teamboard** 列顺序：`#` / `Developer` / **`Group`** / `Tokens` / `Cost`。

**移动端**：现有实现在 `sm` 以下把 Tokens/Cost 合并为一个 `Usage` 单元格。Team / Group 在移动端不占独立列，而是作为 `Developer` 单元格内 `@username` 下方的一行小字徽章展示，避免横向溢出。

### FR-6 Profile 团队信息

在 `/u/[username]` 的资料区展示 Team 与 Group；无则不展示该区块。

**仅本人**可见并可操作"退出 Group" / "退出 Team"两个按钮（当前 profile 组件没有 `isOwner` 概念，需新增）。admin 点击退出 Team 时，提示必须先移交 admin 或解散团队。

### FR-7 其余功能保持现状

Docs、Privacy、Terms、Settings、embed / badge SVG、archive 导入、CLI 全链路，除上文点名的引用改写外一律不动。

**经你显式确认的例外——只动 Docs 页**：删除 `web/src/app/(main)/docs/page.tsx` 的四个章节——「The verified badge」（决策 D-3 取 C）、「Architecture」、「Sponsors」，以及「iOS app」（决策 D-4 取删除）。同时移除该页的 `TESTFLIGHT_URL` 常量与 metadata / OG 里「or get the iOS app」的引用——站点的其余部分没有 iOS App 入口。

**删除范围的边界（明确）**：只动 `web/src/app/(main)/docs/page.tsx` 这一个文件。**不要**顺带删除全站共享的 `web/src/components/layout/ServiceFooter.tsx`——它有自己的 Sponsors 行（V.PS / Neon）与「Built on Tokscale」上游署名，不在点名范围内，保持原样。Docs 页的「Sponsors」指该页 Architecture 章节内嵌的 V.PS / Neon 赞助说明，与页脚的 Sponsors 行是两处独立内容，不要混为一谈。

Profile 页社交链接图标行不受影响；embed / badge SVG 经核查不渲染该徽章，无需改动。

### FR-8 多语言（i18n）

**入口**：导航右上角，主题切换按钮左侧新增语言切换按钮（地球图标）。点击展开下拉，提供 `English` 与 `中文` 两项，当前语言高亮。

**持久化**：选择写入 `tt_locale` cookie（httpOnly=false，前端可读，1 年有效期），与既有 cookie 惯例一致。

**实现**：自建轻量 i18n，**不引入运行时依赖**（不引入 next-intl 等）：

- `web/src/lib/i18n/` 下按页面组织字典文件，`t(key, vars?)` 查询函数支持插值
- `<html lang>` 从硬编码 `en` 改为跟随 `tt_locale`
- 服务端组件经 `cookies()` 读取；客户端组件经 React context 获取初始值
- `format.ts` 的 `Intl.NumberFormat("en-US")` 硬编码改为跟随当前 locale。zh-CN 与 en-US 的千分位与紧凑格式一致，榜单布局不受影响

**与缓存的交互（必须处理，否则两种语言会互相串）**：

- `worker.ts` 的 `caches.default` 页面缓存 key 当前只含 URL——把 `tt_locale` 值并入 synthetic cache key 的查询参数，两个语言各自缓存
- `u/[username]` 的 `unstable_cache` key 与 R2 渲染页缓存同理加入 locale
- `leaderboard` / `teamboard` 等本就动态读库的页面不受影响

**文案范围**：全部页面——Leaderboard、Teamboard、Teams、Profile、Settings、Docs、Privacy、Terms、认证页、device 页、空态 / 错误态、toast、导航、页脚。**Privacy 与 Terms 的中文版为便利翻译，页内标注「英文版本为准」**。CLI（Rust）不在范围内。

**邮件**：本次只做 UI 语言切换。注册 / 邀请 / 重置密码邮件统一用英文，不引入按收件人语言的偏好——那会需要在 `users` 表加 `locale` 列并处理存量回填，扩大了认证迁移面，而需求只要求 UI。若日后要按收件人语言发邮件，再作为独立需求评估。

### FR-9 品牌图标底色改紫（独立小改动，与本次重构无关）

把品牌块（"Tokens" 文字左侧的方块）的蓝色底色 `#2F6FDB` 改为紫色 `#7C3AED`（已确认），白色 T 图案保持不变。

**改动面（先只改页面里 Tokens 前的图标本体）**：
- `web/src/components/layout/Navigation.tsx` 的 `TokensMark` 内联 SVG（`fill="#2F6FDB"`）——页面品牌块本体
- demo 里的同款品牌 tile（`docs/demo/teamboard-demo.html`，规划产物）

**本轮不改，需说明**：浏览器标签页 / 安装图标是另一套 surface。其中 `web/public/brand/tokens-favicon.svg` 是**文本 SVG**（不是二进制，改它只是一行的成本，但它属于标签页图标，本轮不动）；而位图资源（`web/public/favicon-*.png`、`android-chrome-*.png`、`favicon.ico`、`brand/tokens-app-icon-*.png`、`brand/tokens-mark-rounded.png`）是**二进制资产**，仓库**没有**现成的 SVG→PNG 生成流程，没有生成流程或你明确要求之前不重着色。本轮这套 surface 一律不改——代价是浏览器标签页与安装图标会暂时仍是蓝色。是否要同步它们（SVG favicon 可顺手改，位图需建生成流程）需要你明确点头。

**不改**：`web/public/brand/tokens-mark.svg`——它用 `currentColor` 画在透明底上，是单色版，不带底色，不受影响。

**不要全局替换 `#2F6FDB`**。这个蓝色在别处还有用途——demo 里的用户头像、团队渐变、AVA 调色板都用了它，与品牌块无关。只改上表列出的品牌块本身（`Navigation.TokensMark` + demo 同款 tile），其它 `#2F6FDB` 一律不动。

**注**：这是一条与本次重构无关的独立小改动，应单独提交。

---

## 7. 数据结构变更

### 7.1 迁移 `0024_add_password_auth.sql`

```sql
ALTER TABLE "users" ADD COLUMN IF NOT EXISTS "password_hash" varchar(255);
ALTER TABLE "users" ADD COLUMN IF NOT EXISTS "email_verified_at" timestamp with time zone;
ALTER TABLE "users" ALTER COLUMN "github_id" DROP NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS "users_email_lower_unique"
  ON "users" (lower("email")) WHERE "email" IS NOT NULL;

CREATE TABLE IF NOT EXISTS "email_verification_tokens" (
  "id"         uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  "user_id"    uuid NOT NULL REFERENCES "users"("id") ON DELETE cascade,
  "token_hash" varchar(64) NOT NULL,
  "purpose"    varchar(20) NOT NULL,          -- 'verify_email' | 'reset_password'
  "expires_at" timestamp with time zone NOT NULL,
  "consumed_at" timestamp with time zone,
  "created_at" timestamp with time zone DEFAULT now() NOT NULL,
  CONSTRAINT "email_verification_tokens_token_hash_unique" UNIQUE("token_hash")
);
CREATE INDEX IF NOT EXISTS "idx_email_verification_tokens_user_purpose"
  ON "email_verification_tokens" ("user_id", "purpose");
CREATE INDEX IF NOT EXISTS "idx_email_verification_tokens_expires_at"
  ON "email_verification_tokens" ("expires_at");
```

`github_id` 由 `NOT NULL` 放宽为可空，是向后兼容变更；旧数据一行不动。`email` 的唯一索引采用部分索引（`WHERE email IS NOT NULL`），使历史上邮箱为空的行不会阻塞迁移。
### 7.2 迁移 `0025_add_teams_and_groups.sql`

```sql
CREATE TABLE IF NOT EXISTS "teams" (
  "id"           uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  "name"         varchar(100) NOT NULL,
  "slug"         varchar(100) NOT NULL,
  "avatar_url"   text,
  "visibility"   varchar(10) DEFAULT 'private' NOT NULL,  -- public | private
  "status"       varchar(10) DEFAULT 'active' NOT NULL,   -- active | disbanded
  "created_by"   uuid NOT NULL REFERENCES "users"("id") ON DELETE cascade,
  "disbanded_at" timestamp with time zone,
  "created_at"   timestamp with time zone DEFAULT now() NOT NULL,
  "updated_at"   timestamp with time zone DEFAULT now() NOT NULL,
  CONSTRAINT "teams_slug_unique" UNIQUE("slug")
);

-- Teamboard 的团队候选列表只查公开且未解散的团队，走覆盖此条件的部分索引
CREATE INDEX IF NOT EXISTS "teams_public_active_idx"
  ON "teams" ("name")
  WHERE "visibility" = 'public' AND "status" = 'active';

CREATE TABLE IF NOT EXISTS "team_members" (
  "id"         uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  "team_id"    uuid NOT NULL REFERENCES "teams"("id") ON DELETE cascade,
  "user_id"    uuid NOT NULL REFERENCES "users"("id")  ON DELETE cascade,
  "role"       varchar(10) DEFAULT 'member' NOT NULL,    -- admin | subadmin | member
  "invited_by" uuid REFERENCES "users"("id") ON DELETE set null,
  "joined_at"  timestamp with time zone DEFAULT now() NOT NULL,
  CONSTRAINT "team_members_user_unique" UNIQUE("user_id")   -- INV-1
);
CREATE INDEX IF NOT EXISTS "idx_team_members_team_role" ON "team_members" ("team_id", "role");

CREATE TABLE IF NOT EXISTS "groups" (
  "id"           uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  "team_id"      uuid NOT NULL REFERENCES "teams"("id") ON DELETE cascade,
  "name"         varchar(100) NOT NULL,
  "status"       varchar(10) DEFAULT 'active' NOT NULL,
  "created_by"   uuid NOT NULL REFERENCES "users"("id") ON DELETE cascade,
  "disbanded_at" timestamp with time zone,
  "created_at"   timestamp with time zone DEFAULT now() NOT NULL,
  "updated_at"   timestamp with time zone DEFAULT now() NOT NULL,
  CONSTRAINT "groups_team_name_unique" UNIQUE("team_id", "name")
);
CREATE INDEX IF NOT EXISTS "idx_groups_team_status" ON "groups" ("team_id", "status");

CREATE TABLE IF NOT EXISTS "group_members" (
  "id"        uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  "group_id"  uuid NOT NULL REFERENCES "groups"("id") ON DELETE cascade,
  "user_id"   uuid NOT NULL REFERENCES "users"("id")  ON DELETE cascade,
  "joined_at" timestamp with time zone DEFAULT now() NOT NULL,
  CONSTRAINT "group_members_user_unique" UNIQUE("user_id")  -- INV-3
);
CREATE INDEX IF NOT EXISTS "idx_group_members_group_id" ON "group_members" ("group_id");

CREATE TABLE IF NOT EXISTS "team_invitations" (
  "id"                uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  "team_id"           uuid NOT NULL REFERENCES "teams"("id") ON DELETE cascade,
  "invited_email"     varchar(255),
  -- 仅审计用：记录 admin 当初输入的字面值，不参与去重
  "invited_username"  varchar(39),
  -- 归一化结果：能解析到已注册用户时必填，未注册地址为 NULL
  "invited_user_id"   uuid REFERENCES "users"("id") ON DELETE cascade,
  "invited_by"        uuid NOT NULL REFERENCES "users"("id") ON DELETE cascade,
  -- pending | accepted | declined | revoked | expired | superseded
  "status"            varchar(10) DEFAULT 'pending' NOT NULL,
  "token_hash"        varchar(64) NOT NULL,
  "expires_at"        timestamp with time zone NOT NULL,
  "responded_at"      timestamp with time zone,
  "created_at"        timestamp with time zone DEFAULT now() NOT NULL,
  CONSTRAINT "team_invitations_token_hash_unique" UNIQUE("token_hash"),
  -- 归一化后至少要有一个可投递目标
  CONSTRAINT "team_invitations_target_present"
    CHECK ("invited_user_id" IS NOT NULL OR "invited_email" IS NOT NULL)
);

-- 已注册受邀人的去重键。无论最初按用户名还是按邮箱邀请，都已归一到 invited_user_id
CREATE UNIQUE INDEX IF NOT EXISTS "team_invitations_pending_user_unique"
  ON "team_invitations" ("team_id", "invited_user_id")
  WHERE "status" = 'pending' AND "invited_user_id" IS NOT NULL;

-- 邮箱去重键只负责尚未注册的地址；条件与上一个索引互斥，
-- 否则同一个人会同时命中两个索引却互不相斥，等于没有去重
CREATE UNIQUE INDEX IF NOT EXISTS "team_invitations_pending_email_unique"
  ON "team_invitations" ("team_id", lower("invited_email"))
  WHERE "status" = 'pending' AND "invited_user_id" IS NULL;
CREATE INDEX IF NOT EXISTS "idx_team_invitations_invited_user_status"
  ON "team_invitations" ("invited_user_id", "status");
CREATE INDEX IF NOT EXISTS "idx_team_invitations_expires_at"
  ON "team_invitations" ("expires_at");
```

设计沿用了已删除的 `0009_add_groups.sql` 的既有惯例（uuid 主键、`token_hash` 唯一、`role` 用 `varchar(10)`、外键上建索引），以便与仓库风格保持一致。

### 7.3 TypeScript 类型扩展

| 文件 | 变更 |
|---|---|
| `web/src/lib/leaderboard/types.ts` | `LeaderboardUser` 增加 `team: { id, name, slug } \| null`、`group: { id, name } \| null` |
| `web/src/components/profile/types.ts` | `ProfileUser` 增加同样两个字段；新增 `isOwner: boolean` |
| `web/src/lib/db/schema.ts` | 新增 5 张表定义与 `$inferSelect` / `$inferInsert` 类型导出 |
| 新增 `web/src/lib/teams/types.ts` | `TeamRole`、`TeamVisibility = 'public' \| 'private'`、`TeamDetail`（含 `visibility`）、`TeamMemberRow`、`GroupSummary`、`InvitationRow` |

### 7.4 榜单查询变更

`getLeaderboard.ts` 的两条主查询各增加两组 LEFT JOIN：

```
LEFT JOIN team_members  ON team_members.user_id  = users.id
LEFT JOIN teams         ON teams.id = team_members.team_id AND teams.status = 'active'
LEFT JOIN group_members ON group_members.user_id = users.id
LEFT JOIN groups        ON groups.id = group_members.group_id AND groups.status = 'active'
```

因为 `team_members.user_id` 与 `group_members.user_id` 都是 UNIQUE，这两组 JOIN 相对 `users` 严格是 0..1 关系，**不会放大行数**，既有的 `SUM()` 聚合结果不受影响。这是选择「一人一 Team、一人一 Group」模型的直接工程收益——若改为多对多（决策 D-1），必须改写为子查询聚合，否则所有历史数字都会翻倍出错。

`unstable_cache` 的 key 需要纳入 `teamId` / `groupIds`；成员或角色变更时 `revalidateTag('leaderboard')` 主动失效。

---

## 8. 技术栈变化

| 能力 | 现状 | 方案 | 理由 |
|---|---|---|---|
| 密码哈希 | 无 | **WebCrypto PBKDF2-HMAC-SHA256**，210,000 次迭代，每用户 16 字节随机 salt，存 `pbkdf2$sha256$<iter>$<salt_b64>$<hash_b64>` | bcrypt / argon2 是原生模块，Workers 上不可用。PBKDF2 由 SubtleCrypto 原生支持，**零新增依赖** |
| 邮件发送 | 无 | **Resend HTTP API**，用原生 `fetch` 调用，新增 `RESEND_API_KEY`、`EMAIL_FROM` 两个 secret | 不引入 SDK；Workers 上 SMTP 不可用，只能走 HTTP API |
| 频率限制 | 无 | Cloudflare **Rate Limiting binding**，作用于注册 / 登录 / 忘记密码 / 邀请 | 防撞库与邮件轰炸；平台原生能力，无需自建 |
| UI 组件 | 缺 Select / Dialog / Checkbox / Popover / Form / Label / Textarea | 用已有的 `shadcn` CLI 增补，vendored 到 `components/ui/` | 遵守 `upstream_policy` 的「组件只来自 `components/ui/`，不引入第三方组件库」 |
| 定时任务 | `20 3 * * *` 刷新 GitHub social links | **追加**清理过期邀请与过期验证 token；原刷新逻辑保留，仅去掉其返回值中的 `verified` 计数 | 新表需要定期清理。D-3 取 C 后徽章移除，但社交链接图标行仍依赖该刷新，故不停用 |
| 多语言 | 无（`<html lang="en">` 硬编码） | 自建字典 + `t()`，`tt_locale` cookie 持久化，`<html lang>` 跟随；worker 页面缓存 key 并入 locale | 不引入 next-intl 等运行时依赖；URL 不加语言前缀，路由不变。SEO 只索引英文版本，这是可接受的代价 |
| 用户搜索 | 无 | `GET /api/users/search`，限 admin / subadmin 角色；按用户名 / 显示名匹配，返回 `username` / `displayName` / `avatarUrl`（**不含 `email`**）；完整邮箱做精确匹配但仍只回身份字段 | 邀请下拉的候选来源；不对普通用户开放，且不回显邮箱，避免全站账户 / 邮箱枚举 |

**不需要新增任何 npm 运行时依赖**——这是有意为之，依赖越少，后续与上游同步越容易。

---

## 9. API 设计

全部遵循仓库现有惯例：Route Handler 内直写 `try/catch` + `NextResponse.json`，无统一 wrapper；变更类接口用 `getSessionFromRequest({ allowAuthorizationHeader: false })` 加 CSRF Origin 白名单；错误格式 `{ error: string }` 或 `{ error, details: string[] }`。

### 9.1 认证

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/auth/register` | 邮箱 + 用户名 + 密码；成功即建会话并发送验证邮件 |
| POST | `/api/auth/login` | 邮箱 + 密码 |
| POST | `/api/auth/forgot-password` | 无论邮箱是否存在都返回 200（防枚举） |
| POST | `/api/auth/reset-password` | token + 新密码；成功后吊销该用户全部会话 |
| POST | `/api/auth/verify-email` | 消费验证 token |
| POST | `/api/auth/resend-verification` | 已登录用户重发验证邮件；未登录 401。Forgot-password 不能代替 |
| — | `/api/auth/github`、`/api/auth/github/callback` | **删除** |
| — | `/api/auth/session`、`/logout`、`/token`、`/device/*` | **契约不变** |

### 9.2 Team / Group

| 方法 | 路径 | 权限 |
|---|---|---|
| GET / POST | `/api/teams` | GET 返回公开团队 ∪ 本人所属团队 / 创建需登录，请求体含 `visibility` |
| GET | `/api/teams/[teamId]` | 公开团队任何人可读；`private` 仅本团队成员，否则 404（不用 403，避免探测团队是否存在）|
| PATCH | `/api/teams/[teamId]` | admin、subadmin；可改名称 / 头像 / `visibility` |
| POST | `/api/teams/[teamId]/disband` | admin |
| DELETE | `/api/teams/[teamId]` | **`teams.created_by` 本人**（已解散的团队无成员行可查，不读 `role`），且已解散 + 成员 0 |
| GET / POST | `/api/teams/[teamId]/members` | GET 遵循团队可见性（INV-10）/ 批量邀请需 admin、subadmin；POST 在事务内归一化收件人并按 §6 FR-3 去重，重复项返回"已存在待处理邀请"而非报错 |
| PATCH / DELETE | `/api/teams/[teamId]/members/[userId]` | 改角色 admin 独有；移除 admin、subadmin |
| DELETE | `/api/teams/[teamId]/members/me` | 本人退出 |
| GET / POST | `/api/teams/[teamId]/groups` | GET 遵循团队可见性（INV-10）/ 建需 admin、subadmin |
| PATCH | `/api/teams/[teamId]/groups/[groupId]` | admin、subadmin |
| POST | `/api/teams/[teamId]/groups/[groupId]/disband` | admin、subadmin |
| DELETE | `/api/teams/[teamId]/groups/[groupId]` | admin、subadmin，且已解散 + 成员 0 |
| PUT / DELETE | `/api/teams/[teamId]/groups/[groupId]/members/[userId]` | admin、subadmin |
| DELETE | `/api/teams/[teamId]/groups/[groupId]/members/me` | 本人退出 |
| GET | `/api/me/invitations` | 本人待处理邀请 |
| GET | `/api/users/search?q=` | 限任一团队的 admin / subadmin；按用户名 / 显示名匹配，返回 `username` / `displayName` / `avatarUrl`（**不含 `email`**）；`q` 为完整邮箱时按邮箱精确匹配但仍只回身份字段；上限 10 条；仅用于邀请下拉 |
| POST | `/api/invitations/[id]/accept`、`/api/invitations/[id]/decline` | 受邀人本人 |
| DELETE | `/api/teams/[teamId]/invitations/[id]` | admin、subadmin 撤销 |

### 9.3 榜单

| 方法 | 路径 | 变更 |
|---|---|---|
| GET | `/api/leaderboard` | 响应体每行增加 `team` / `group` 字段 |
| GET | `/api/teamboard` | **新增**，参数 `teamId`（必填）、`groupIds`（可多值）、`period`、`sortBy`、`page`、`search`；`teamId` 须为 `public` 或调用者所属团队，否则 404（INV-10）|
---

## 10. 迁移与上线策略

### 10.1 存量用户处置

关键前提：本 fork 目前**没有独立的生产数据库**（线上 `tokens.ci` 属于上游 `missuo/tokens`）。因此推荐 A 方案。

**A 方案（推荐，全新部署）**：直接硬切换。`users.github_id` 允许为空，新注册用户全部走邮箱路径，无需兼容层。

**B 方案（若已有存量 GitHub 用户）**：

1. 上线邮箱注册的同时，在登录页保留「认领账号」入口
2. 存量用户输入其 GitHub 账号曾绑定的邮箱 → 收验证邮件 → 设置密码 → 完成认领
3. `email` 为空的存量用户无法自助认领，需人工处理
4. 观察 30 天后再移除认领入口

**必须提前告知的影响**：GitHub OAuth 一旦下线，`verified` 徽章的数据源（GitHub social links）对新用户随之失效——因为 `username` 不再保证是 GitHub login。**决策 D-3 已取 C：徽章整体移除**，因此不会留下一个静默失效的功能。代价是存量已点亮徽章的用户会看到它消失，这是一项用户可见的变更，需在上线公告里点名。Profile 页的社交链接图标行不受影响。

### 10.2 回滚

- `0025`（团队体系）：纯新增表，回滚即 `DROP TABLE`，不触碰任何既有数据。
- `0024`（认证列）：新增列 + 放宽约束。**代码回滚必须先于数据库回滚**，否则已注册的邮箱用户会失去登录入口。若已有邮箱用户注册，不建议回滚 `0024`，应改为向前修复。

### 10.3 部署顺序

执行迁移 `0024` / `0025` → 部署 Worker（含新 secret 与 Rate Limiting 绑定）→ 冒烟验证：注册、登录、device flow、submit、榜单渲染。

---

## 11. 任务拆分与依赖关系

### 11.1 依赖图

```
                    ┌──────────────────────────┐
                    │ T0 基线修复（缺陷 1、2） │
                    └────────────┬─────────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        ▼                        ▼                        ▼
┌───────────────┐      ┌──────────────────┐      ┌─────────────────┐
│ T1 删 Shame   │      │ T2 认证改造      │      │ T3 团队数据模型 │
│  + 导航占位   │      │ （邮箱注册登录） │      │  （0025 迁移）  │
└───────┬───────┘      └────────┬─────────┘      └────────┬────────┘
        │                       │                         │
        │                       └───────────┬─────────────┘
        │                                   ▼
        │                        ┌─────────────────────┐
        │                        │ T4 Team/Group API   │
        │                        │   + 权限 + 邀请     │
        │                        └──────────┬──────────┘
        │                                   │
        │              ┌────────────────────┼────────────────────┐
        │              ▼                    ▼                    ▼
        │    ┌──────────────────┐  ┌─────────────────┐  ┌───────────────┐
        │    │ T5 Team 管理页面 │  │ T6 Leaderboard  │  │ T8 Profile    │
        │    │                  │  │    加两列       │  │ 团队区 + 退出 │
        │    └──────────────────┘  └────────┬────────┘  └───────────────┘
        │                                   │
        └───────────────┬───────────────────┘
                        ▼
              ┌───────────────────┐
              │ T7 Teamboard 页面 │
              │   + 双层筛选      │
              └─────────┬─────────┘
                        ▼
              ┌───────────────────┐
              │ T9 文档/策略/收尾 │
              └───────────────────┘
```

### 11.2 任务清单

| ID | 任务 | 依赖 | 规模 | 可独立验收标准 |
|---|---|---|:---:|---|
| **T0** | 基线修复：清理 `check-migrations.ts` 的 group 断言；补齐 0022 / 0023 snapshot | — | S | `bun run test:migrations` 通过 |
| **T1** | 删除 Shame；导航项替换为 Teamboard（先接占位页） | T0 | S | `/shame` 404；导航显示 Teamboard；全仓无残留引用 |
| **T2** | 认证改造：`0024` 迁移、PBKDF2、注册/登录/改密/验证邮箱、删 OAuth、删所有 GitHub 入口、改头像 fallback | T0 | **L** | 可注册登录；device flow 与 submit 回归通过；全站无 GitHub 图标 |
| **T3** | 团队数据模型：`0025` 迁移 + Drizzle schema + 类型 | T0 | M | 迁移可正向执行；类型编译通过 |
| **T4** | Team/Group 领域服务与 API：权限、解散/删除两段式、邀请（含批量）、`GET /api/users/search` | T2、T3 | **L** | 权限矩阵逐格可验证；INV-1..INV-8 均有对应测试；search 接口非管理员调用返回 403 |
| **T5** | Team 管理页面 UI（含补齐 shadcn 组件）；邀请对话框为可搜索下拉多选（combobox），选中计数，邮箱路径保留 | T4 | **L** | 页面可完成全部 Team / Group 操作；邀请下拉支持搜索、勾选、计数 |
| **T6** | Leaderboard 加 Team / Group 两列（含骨架屏、移动端） | T3 | M | 两列正确渲染且空值留空；排序与格式切换无回归 |
| **T7** | Teamboard 页面 + Team 单选 / Group 多选筛选 + `/api/teamboard` | T3、T6 | **L** | 筛选组合正确；列顺序符合 FR-5 |
| **T8** | Profile 展示 Team / Group + 退出操作（新增 `isOwner`） | T4 | M | 本人可见退出按钮；他人不可见；admin 退出有正确阻断提示 |
| **T10** | i18n 基础设施：字典目录与 `t()`、`tt_locale` cookie、语言切换器、`<html lang>` 跟随、worker 缓存 key 并入 locale、`format.ts` locale 跟随 | T0 | **L** | 切换语言后已包裹的页面文案跟随；两种语言各自缓存不错串 |
| **T11** | 全站文案包裹：逐页把硬编码字符串收进字典（含邮件模板），Privacy / Terms 标注「英文为准」 | T10，且需在各页面任务完成后 | **L** | 全部页面无遗漏；`<html lang>` 正确；切换语言后所有页面完整跟随 |
| **T12** | 品牌图标底色改紫（独立小改动） | T0 | S | 导航品牌块 + SVG favicon + 位图资源全部改紫，白色 T 不变；favicon 与页面角落颜色一致；建议单独提交 |
| **T9** | 更新 `upstream_policy.md`、`.trellis/spec`、README；最终回归 | T1–T8 | M | 文档与实现一致；lint + typecheck + 冒烟通过 |

### 11.3 推荐实施顺序

**T0 → T1 → T2 → T3 → T4 → T6 → T10 → T5 → T7 → T8 → T11 → T9**

理由：`T1` 最便宜且立刻可见，先做能腾出导航位；`T2` 是最大的单点风险（触及全站认证），必须在团队功能压上来之前独立完成并回归；`T6` 提前到 `T5` 之前，是因为它验证了 `T3` 的数据模型能否真正被榜单消费——如果 JOIN 有问题，越早发现越好。

`T10` 插在 `T5` / `T7` 之前，让 Teams 与 Teamboard 两个新页面从第一天就写字典 key 而不是硬编码字符串——否则它们会在 `T11` 被二次包裹，是纯返工。`T11` 收编 `T1` / `T2` / `T6` / `T8` 产出的既有文案，因此放在它们之后、`T9` 之前。

`T5` 与 `T6`、`T8` 之间没有文件冲突，可并行；`T7` 必须等 `T6`，因为两者共用列渲染组件。`T10` 与 `T1`–`T6` 无冲突，可并行。`T12`（品牌底色改紫）独立，可在任意节点做，建议单独提交、与本次重构的 diff 分开。

---

## 12. 验收标准

- [ ] `/shame` 返回 404，全仓无 Hall of Shame 残留引用，封禁机制本身仍生效
- [ ] 导航第二项为 `Teamboard`，指向 `/teamboard`
- [ ] 可用邮箱 + 用户名 + 密码完成注册；弱密码被拒绝并给出具体原因
- [ ] 全站不存在 GitHub 图标、按钮或 OAuth 入口
- [ ] `tokens login`（device flow）与 `tokens submit` 在改造后无需更新 CLI 即可工作
- [ ] 创建 Team 者成为 admin；可改名、改头像、邀请（用户名 / 邮箱 / 批量）
- [ ] subadmin 最多 2 个；subadmin 能做除解散和删除 Team 外的全部操作，含 Group 全部操作
- [ ] 未解散的 Team / Group 无法删除；解散会清空成员；已解散实体仍可由创建者删除
- [ ] Leaderboard 列序为 `# / Developer / Team / Group / Tokens / Cost`，空值留空，排序行为无回归
- [ ] Teamboard 支持 Team 单选 + Group 多选，列序为 `# / Developer / Group / Tokens / Cost`
- [ ] 建团表单必须显式选择可见性，未选择时落库为 `private`
- [ ] Teamboard 筛选器列出「公开团队 ∪ 本人所属团队」；请求他人的 `private` 团队返回 404
- [ ] 团队从 `public` 改为 `private` 后，非成员立即无法再在 Teamboard 中选中它
- [ ] Profile 展示 Team / Group，本人可退出，无归属时不渲染该区块
- [ ] 全站无 `verified` 徽章渲染，`socialVerification.ts` 与 `VerifiedBadge.tsx` 已删除，全仓无残留引用
- [ ] Profile 页社交链接图标行仍正常展示（徽章移除不牵连社交链接功能）
- [ ] Docs 页不再有「The verified badge」「Architecture」「Sponsors」「iOS app」四个章节，`TESTFLIGHT_URL` 与 metadata / OG 里的 iOS 引用已移除，全站再无 iOS App 入口；Privacy / Terms / Settings / embed / badge / archive 行为无变化
- [ ] Docs 页保留章节（Install the CLI / Everyday use / Supported clients）的 **default-English 渲染内容与结构**与当前线上 tokens.ci 一致（仅删上述四节、替换 GitHub 登录文案）；**验收基准是 tokens.ci 的当前页面，不是 demo 文件**。注：T11 的 i18n 包裹会让源码变化，这里的「一致」指渲染产物，不是源码逐行不动
- [ ] 邀请对话框为可搜索下拉多选：输入实时筛选、选项左侧勾选框、选中计数、可再次点击取消；输入完整邮箱可邀请未注册用户
- [ ] 导航右上角主题切换左侧有语言切换按钮，提供 English / 中文；切换后所有页面文案完整跟随，无遗漏硬编码
- [ ] `tt_locale` 持久化跨会话生效，`<html lang>` 跟随
- [ ] worker 页面缓存按 locale 分离，切换语言后刷新页面不会看到另一种语言的缓存
- [ ] Privacy / Terms 中文页标注「英文版本为准」
- [ ] 品牌块（导航左上角 `TokensMark` + demo 同款）蓝色底色改为紫色，白色 T 图案不变；favicon / 安装图标的位图本轮不重着色（无二进制生成流程），已明确说明未改
- [ ] `bun run lint`、`bun run typecheck`、`bun run test:migrations` 全绿

---

## 13. 决策状态

**四项决策均已由你明确选择，无待确认项，本次改造不再存在决策类阻塞。**

| # | 议题 | 取值 | 状态 |
|---|---|---|---|
| D-1 | 成员能否同时属于多个 Group | **否——一人一 Group**，暂不支持一对多 | **已确认** |
| D-2 | Teamboard 的团队可见范围 | **公开团队可浏览 + 建团时由 admin 选择可见性** | **已确认**（详见下文） |
| D-3 | `verified` 徽章去留 | **选项 C——直接移除徽章** | **已确认** |
| D-4 | iOS App 的去留与适配 | **删除**——从站点移除 iOS App 引用 | **已确认**（详见下文） |

#### D-1 详述：一人一 Group（已确认）

原始需求把 Group 描述为「把 Team 成员编排标记进行分组，点某个 group 只看它的成员」，一人一组是能满足这句话的最小模型，也是让榜单两个 LEFT JOIN 保持 0..1、不放大行数的前提。据此 `group_members.user_id` 保留 UNIQUE 约束（INV-3），`Group` 列按单值渲染。

**这不是一道被永久关闭的题**：若日后要改为多组，`Group` 列需变为多值徽章，`group_members.user_id` 的唯一约束需移除，榜单 JOIN 随之放大行数，必须改写为子查询聚合。这属于数据模型级返工，届时应作为独立需求重新评估，本次不为它预留结构。


#### D-2 详述：Teamboard 的团队可见范围

原始需求只说明了「Teamboard 有 Team 单选筛选器」，**没有定义谁能选到哪些团队**。这是一道授权题而非 UI 细节，已单独确认，取值为**折中方案**：

- `teams.visibility` 取 `public` / `private`，**建团时由 admin 显式选择**，事后可由 admin 或 subadmin 修改
- **数据库默认 `private`**（fail-safe：任何未显式指定可见性的写入都不会意外公开）；建团表单强制二选一，未作选择时按 `private` 落库
- 非成员只能浏览 `public` 且未解散的团队；成员始终能浏览自己的团队，无论其可见性（INV-10）
- 未登录访客可以浏览公开团队，因此「Team 单选」交互是有实际意义的

**为什么不取两个极端**：
- 「仅本人所属 Team」看似最安全，但受 INV-1（一人至多属一个 Team）约束，筛选器实际只有一个选项，**你要求的单选交互会形同虚设**
- 「全部团队无条件公开」让团队成员构成变成可爬取的公开数据，且暴露范围不由团队自己控制；更麻烦的是**收紧时需要回收已经公开出去的数据**，成本远高于放开

折中方案把暴露范围的决定权交给每个团队的 admin，既保住单选交互的意义，又让默认状态是安全的。

**一个不受 D-2 影响的既成事实**：Leaderboard 的 `Team` / `Group` 列是你明确要求的，因此**团队名与分组名本来就对所有访客可见**，`private` 也不例外。D-2 控制的是「能否按团队聚合浏览其完整成员名单」，不是「团队名是否公开」。若你也希望 `private` 团队在 Leaderboard 上隐藏团队名，这是一项**额外需求**，需要另行确认——它会让该列对部分行留空，与你「有团队就显示」的原始描述不一致。

#### D-3 详述：`verified` 徽章去留（已确认取 C）

**这是一道你没有提出的题，但改造把它逼了出来。** 现状是：`verified` 徽章的判定条件是「GitHub 个人主页上的社交链接 ≥ 2 条」，数据由每日 cron 调 GitHub 拉取，靠 `users.username` 恰好是 GitHub login 来对应。需求 4 移除 GitHub 登录后，`username` 不再保证是 GitHub 用户名，**这个数据源对新用户就失效了**——这是移除 OAuth 的事实后果，不是一项可选改动。

「数据源失效之后怎么办」有三条互斥的路，**你已选定 C**：

| 选项 | 做法 | 代价 | 结论 |
|---|---|---|---|
| A | 代码与语义完全不动 | 徽章对新用户不会点亮，成为静默失效的功能；tooltip 里仍会提到 GitHub | 未采纳 |
| B | 语义改绑 `email_verified_at` | 改变了一项未提及功能的含义：老用户会掉徽章或凭空得到徽章 | 未采纳 |
| **C** | **直接移除徽章** | 删掉一项用户可见功能，但语义不含糊、不留静默失效的死功能 | **已采纳** |

##### C 的落地边界：只删徽章，不动社交链接

`verified` 徽章与 Profile 页的社交链接图标行是**两个独立功能**，代码上也不耦合——`ProfileSocialLinks.tsx` 完全不引用 `verified`。因此 C 的范围严格限定在徽章本身。

**删除文件（2 个）**
- `web/src/lib/socialVerification.ts`（`SOCIAL_VERIFIED_THRESHOLD`、`isVerifiedBySocialLinks`）
- `web/src/components/ui/VerifiedBadge.tsx`

**移除引用（6 个文件）**

| 文件 | 移除内容 |
|---|---|
| `web/src/lib/leaderboard/getLeaderboard.ts` | `SOCIAL_VERIFIED_THRESHOLD` import、`verifiedExpr()` 定义、3 处 `verified: verifiedExpr().as("verified")` 投影与 1 处 `select` 投影、4 处 `Boolean(row.verified)` 映射、3 处内部类型的 `verified` 字段 |
| `web/src/lib/leaderboard/types.ts` | `LeaderboardUser.verified` 字段 |
| `web/src/components/leaderboard/Leaderboard.tsx` | `VerifiedBadge` import 与 `DeveloperRow` 内的渲染 |
| `web/src/components/profile/ProfileView.tsx` | `VerifiedBadge` import、`verified?: boolean` prop、解构与渲染 |
| `web/src/app/u/[username]/ProfilePageClient.tsx` | `isVerifiedBySocialLinks` import 与 `verified={...}` 传参 |
| `web/src/app/(main)/docs/page.tsx` | 整个 `<Section id="verified">`「The verified badge」章节（L315-372）；已确认无 TOC 或其他锚点引用。该段 L365 同时是本文件唯一的 `/shame` 链接，与需求 1 的改动点重合 |
| `web/src/lib/cron/refreshSocialLinks.ts` | `isVerifiedBySocialLinks` import、`let verified = 0` 与内层 `verified++` 累加、`RefreshSocialLinksResult.verified` 字段；返回值收敛为 `{ users: rows.length }`。**刷新循环本身保留** |
| `web/src/app/api/cron/refresh-social-links/route.ts` | L56 解构 `({ users, verified })` 改为 `({ users })`、L58 日志去掉 `verified` 片段、L34-35 注释措辞 |
| `web/worker.ts` | `scheduled` 处理器 L245-249 的同一处解构与日志。**该文件不在 `web/src` 下，是最容易漏改的一处** |

上表后三行是徽章移除的**连带必改项**，不是可选项：`refreshSocialLinks.ts` 直接 import 了将被删除的 `isVerifiedBySocialLinks`，而它的返回值又被 cron route 与 `worker.ts` 解构使用。三处不同步修改，`tsc --noEmit` 与 `bun run build` 必然失败。

**保留不动（明确不在 C 的范围内）**
- `users.social_links` / `social_links_synced_at` 两列——Profile 页社交链接图标行仍在使用。因此 **C 不需要任何新迁移，也不与「迁移 additive、不删既有列」约束冲突**
- **social links 刷新任务本身保留**，继续为图标行供数；改动仅限上表列出的 `verified` 计数与日志。`ProfileSocialLinks.tsx` 不引用 `verified`，与徽章相互独立，完全不动
- `githubSocials.ts`（L150、L157）与 `db/schema.ts`（L36）只是注释里提到 "verified badge"，**代码不动**，顺手更正措辞即可
- `lib/auth/github.ts` 中的 `emails[].verified` 是 GitHub 邮箱字段，与本徽章无关，随需求 4 删除 OAuth 时整体移除，不要与徽章混为一谈
- `email_verified_at` 字段照常新增（注册流程本身需要它），与徽章无关

**一项需要你知晓的遗留退化**：该 cron 目前对**全部**用户按 `users.username` 请求 GitHub。OAuth 下线后本地注册用户的用户名多半不是 GitHub login，这些请求会 404，社交链接图标行对他们不会有内容。这不影响正确性（拉取失败即视为空），但会产生大量无效外呼。廉价缓解是只同步 `social_links` 已非空的存量账号——**已记入 T2 作为可选优化，默认不实施**，因为它会改变一项你未点名功能的行为面。

#### D-4 详述：iOS App 的去留与适配（已确认取「删除」）

**关键事实**：iOS App 不在本仓库——它是独立代码库，经 TestFlight 分发（`testflight.apple.com/join/NWmvqqTX`），且**没有登录**，只按手输的公开用户名读取个人资料。本仓库没有任何 iOS 专属 API 路由。

**因此本次 Team / Web 改动不会让 iOS App 失效**——「原样保留」是技术上完全可行的基线。它读公开资料的接口只增不减（`T8` 会给公开 profile API 加 team / group 字段，纯增量），又不走登录，邮箱认证替换与它无关。它只是看不到团队功能——这是功能缺口，不是兼容性损坏。

**三种取值，你已选定「删除」**：

| 取值 | 做法 | 本仓库要做的事 | iOS 代码库要做的事 | 复杂度 |
|---|---|---|---|---|
| **原样保留（基线）** | 不动 | 无 | 无 | 零 |
| **适配** | 在 App 内加 Team / Group 展示 | `T8` 把公开 profile API 的 Team / Group response contract 明确下来（含 null 语义、visibility 语义）并加回归覆盖，给外部 Swift App 一个稳定输入 | 读取 + 渲染新字段；要做 Teamboard 则另加筛选 UI（独立代码库，不在本仓） | 本仓小，App 侧小到中 |
| **删除（已采纳）** | 从站点移除 iOS App 引用 | 删 Docs 页 iOS 节、TestFlight 链接、metadata / OG 里的「or get the iOS app」 | 无（TestFlight 构建仍在，只是不再被展示） | 本仓极低 |

**「删除」的落地**：进入 `T1`，作为对 `docs/page.tsx` 的既有改动一并执行（该文件本就要删 `/shame` 链接与 verified 章节）。公开 profile API 不需要为 iOS 做任何契约工作——那是「适配」路径的需求，已随删除决定一并取消。

**适配的前置核实点**：iOS App 若读取公开 profile 里的 `verified` 字段，而 D-3 把它删了，App 会断——适配前先确认 App 没消费这个字段。

「适配不影响本仓库」这种说法是错的：本仓要把契约做实，Swift 改动才在仓外。

---

## 14. 上游同步

本仓库是三级 fork：`junhoyeo/tokscale` → `missuo/tokens` → `KunoLu/tokens`。本次改造**落地后**会大幅拉开与上游的距离（认证体系完全不同、多出一整个团队子域）。但**在 T0–T9 落地之前，本仓库在 `web/src` / `cli` / `packages` 上与 `upstream/main` 零分歧**，只多了文档与规划产物。

同步策略、各类上游改动的取舍规则、冲突高发区、以及迁移序号撞车的处理办法，见 **`docs/upstream_policy.md`**（已随本次改造一并改写）。该文件把「现状」（§2.1）与「计划态分歧区」（§2.2）分开列出，正是为了让 T0–T12 落地前的同步评审不会拿未来的边界去误判当期上游提交——T9 负责把已落地的行从 §2.2 移入 §2.1。
