# 实施计划

依赖图与任务表见 `docs/prd-teamboard-teams-auth.md` §11。本文件记录每个子任务的落地路径与完成判据。

执行顺序：**T0 → T12 → T1 → T2 → T3 → T4 → T6 → T10 → T5 → T7 → T8 → T11 → T9**

`T12` 紧挨 `T0` 之后、`T1` 之前：`T12` / `T1` / `T2` 都改 `Navigation.tsx`，禁止并行。`T5` / `T6` / `T8` 之间无文件冲突，可并行；`T7` 必须等 `T6`（共用列渲染组件）。`T10` 插在 `T5` / `T7` 之前，让新页面从第一天写字典 key 而非硬编码字符串；`T11` 收编既有页面文案，放在 `T8` 之后、`T9` 之前。

---

## T0 基线修复 `S`

**依赖**：无

- `web/scripts/check-migrations.ts` — 移除对 `groups` / `group_members` / `group_invites` 的断言（L195-216、L280-296、L363-365）
- `web/src/lib/db/migrations/meta/` — 补齐 `0022` / `0023` snapshot

**完成判据**：`bun run test:migrations` 通过。

---

## T1 删除 Shame + 导航占位 `S`

**依赖**：T0、T12（同改 `Navigation.tsx`，T12 须先完成 TokensMark）

删除 `web/src/app/(main)/shame/page.tsx`、`web/src/components/shame/BannedList.tsx`。

改写引用：`Navigation.tsx`（`NAV_LINKS`）、`u/[username]/BannedProfileView.tsx`、`(main)/docs/page.tsx`、`(main)/terms/page.tsx`、`worker.ts`（`PAGE_CACHEABLE`）、`app/error.tsx`、`schema.ts` 注释。

其中 `(main)/docs/page.tsx` 一并删除底部的「Architecture」「Sponsors」两节（Q1）与「iOS app」节 + `TESTFLIGHT_URL` 常量 + metadata / OG 里的 iOS 引用（D-4 取删除）。**边界**：只动这一个文件，全站共享的 `components/layout/ServiceFooter.tsx`（其 Sponsors 行与「Built on Tokscale」署名）**不动**——Docs 页的 Sponsors 是 Architecture 节内嵌的赞助说明，与页脚是两处独立内容。verified 章节的删除在 T2（D-3），该文件两任务串行执行（T1 → T2），勿并行。

导航项换成 `Teamboard`，先接一个占位页。

**完成判据**：`/shame` 404；`rg -i "hall of shame|/shame"` 无残留；封禁用户仍被榜单排除；`rg -ni "testflight|ios app|ios app" web/src` 仅剩迁移文件命中；`ServiceFooter.tsx` 未被改动。

---

## T2 认证改造 `L`

**依赖**：T0

1. 迁移 `0024_add_password_auth.sql`
2. `web/src/lib/auth/password.ts` — PBKDF2 派生与校验
3. `web/src/lib/email/` — Resend 发送与模板
4. 新增 `POST /api/auth/{register,login,forgot-password,reset-password,verify-email}`
5. 新增页面 `/register`、`/login`、`/forgot-password`、`/reset-password`、`/verify-email`
6. **删除** `lib/auth/github.ts`、`api/auth/github/`、`api/auth/github/callback/`
7. 改写 GitHub 入口：`Navigation.tsx`（图标 + Sign in 链接 + 头像 fallback）、`DeviceClient.tsx`、`profile/page.tsx`、`SettingsClient.tsx`、`middleware.ts`
8. 头像 fallback 由 `github.com/<user>.png` 改为按用户名生成的首字母占位图
9. cron **追加**清理过期邀请与过期验证 token；原 social links 刷新任务**保留**（Profile 社交链接图标行仍依赖它），但它的 `verified` 计数随徽章一起移除——涉及的三个文件与逐处改动见第 11 步，此处不重复。`email_verified_at` 照常新增供注册流程使用，与徽章无关
10. 新增 secret `RESEND_API_KEY`、`EMAIL_FROM`；接 Rate Limiting 绑定
11. **移除 `verified` 徽章**（决策 D-3 取 C，已确认）：
    - **删除** `web/src/lib/socialVerification.ts`、`web/src/components/ui/VerifiedBadge.tsx`
    - `lib/leaderboard/getLeaderboard.ts`：删 `SOCIAL_VERIFIED_THRESHOLD` import、`verifiedExpr()` 定义、4 处 select 投影、4 处 `Boolean(row.verified)` 映射、3 处内部类型的 `verified` 字段
    - `lib/leaderboard/types.ts`：删 `LeaderboardUser.verified`
    - `components/leaderboard/Leaderboard.tsx`：删 import 与 `DeveloperRow` 内渲染
    - `components/profile/ProfileView.tsx`：删 import、`verified?: boolean` prop、解构与渲染
    - `app/u/[username]/ProfilePageClient.tsx`：删 `isVerifiedBySocialLinks` import 与 `verified={...}` 传参
    - `app/(main)/docs/page.tsx`：删整个 `<Section id="verified">`（L315-372，已核查无 TOC 或其他锚点引用）。该段 L365 是本文件唯一的 `/shame` 链接，删掉它同时清掉了 T1 在本文件的改动点——两个任务都碰这个文件，按 T1 → T2 串行执行，勿并行
    - `lib/cron/refreshSocialLinks.ts`：**必须改**——删 `isVerifiedBySocialLinks` import、`let verified = 0` 与内层 `verified++` 累加、`RefreshSocialLinksResult.verified` 字段，返回值收敛为 `{ users: rows.length }`。任务本身保留，仍照常刷新 `social_links` 快照
    - `api/cron/refresh-social-links/route.ts`：**必须改**——L56 解构 `({ users, verified })` 改为 `({ users })`，L58 日志去掉 `${verified} verified`；顺带修正 L34-35 注释里"drives the verified badge"的描述
    - `web/worker.ts`：**必须改**——`scheduled` 处理器 L245-249 同样解构并打印 `verified`，改法同上。该文件**不在 `web/src` 下**，是最容易漏改的一处
    - 仅注释过期、代码不动：`githubSocials.ts`（L150、L157 提到 "leaderboard verified badge"）、`db/schema.ts`（L36 ">= 2 entries marks the user as verified"）——顺手更正措辞
    - **真正不动**：`users.social_links` / `social_links_synced_at` 列、`ProfileSocialLinks.tsx`（不引用 `verified`，与徽章相互独立）、`lib/auth/github.ts` 中的 `emails[].verified`（GitHub 邮箱字段，与徽章无关，随第 6 步整体删除）。**本步不产生迁移**
    - 可选优化（**默认不实施**）：cron 改为只同步 `social_links` 已非空的存量账号，避免为本地注册用户空转 404。它会改变一项未点名功能的行为面，需单独确认

**不可触碰**：`lib/auth/session.ts` 的签名与行为、device flow 三端点、`personalTokens.ts`、`/api/submit`、`/api/me/stats`。

**完成判据**：可注册登录；弱密码被拒；`tokens login` + `tokens submit` 全链路通过；`rg -i github web/src` 仅剩文档性文案；`rg -n "verified|VerifiedBadge|socialVerification" web/src web/worker.ts` 只剩迁移文件与 `email_verified_at` 相关命中（**范围必须带上 `web/worker.ts`**，它不在 `web/src` 下）；`bun run typecheck` 与 `bun run build` 通过——漏改的 `verified` 会以解构报错形式在此暴露；Profile 社交链接图标行仍正常渲染。

---

## T3 团队数据模型 `M`

**依赖**：T0

迁移 `0025_add_teams_and_groups.sql`（5 张表）+ `schema.ts` Drizzle 定义 + 类型导出 + `lib/teams/types.ts`。

`teams` 含 `visibility varchar(10) DEFAULT 'private' NOT NULL`（D-2）及部分索引 `teams_public_active_idx`。

**完成判据**：迁移正向执行成功；`bun run typecheck` 通过；`test:migrations` 通过。

---

## T4 Team/Group 领域服务与 API `L`

**依赖**：T2、T3

`web/src/lib/teams/` 内实现领域服务，API 清单见 docs PRD §9.2。

另含 `GET /api/users/search?q=`：限任一团队的 admin / subadmin 调用（否则 403）。按用户名 / 显示名匹配，返回 `username` / `displayName` / `avatarUrl`，**不返回 `email`**——避免向管理员暴露绑定邮箱。仅当 `q` 是完整邮箱时按邮箱精确匹配，且仍只回身份字段（不回显邮箱）。上限 10 条。不对普通用户开放，避免全站账户 / 邮箱枚举。

逐条落实 INV-1..INV-10；并发点按 `design.md` §6 处理，可见性鉴权按 `design.md` §5 处理。

可见性鉴权集中在一处 helper（`canViewTeam(viewer, team)`），供团队详情、成员、分组、Teamboard 四类读接口复用，避免分散判断出现遗漏。

**完成判据**：权限矩阵每一格可验证；解散/删除前置、subadmin 上限并发、邀请去重与过期、退出 Team 级联退出 Group 均有测试。

---

## T5 Team 管理页面 `L`

**依赖**：T4

新增 `/teams` 与 `web/src/components/teams/`；补 shadcn 组件 `select`、`dialog`、`checkbox`、`popover`、`form`、`label`、`textarea`。

UI 参照 `docs/demo/teamboard-demo.html` 的 Teams 页。

**邀请对话框为可搜索下拉多选（combobox）**，不再使用 textarea：输入实时筛选匹配账户（**用户名 / 显示名**，下拉只展示这两者，**不回显邮箱**），每个选项左侧有勾选框，选中即计数、可再点取消；输入完整邮箱时按邮箱精确匹配——命中已注册用户按用户名 / 显示名展示，未匹配则列表底部出现「邀请该邮箱」选项（未注册用户路径保留）。「加入后自动归入分组（可选）」保持不变。候选来自 `GET /api/users/search`（T4）。

**完成判据**：页面可完成全部 Team / Group 操作；建团表单强制选择可见性；admin 与 subadmin 可切换可见性；危险操作有名称二次确认；邀请下拉支持搜索 / 勾选 / 计数 / 邮箱路径。

---

## T6 Leaderboard 加两列 `M`

**依赖**：T3

`lib/leaderboard/types.ts` → `getLeaderboard.ts`（两组 LEFT JOIN + 缓存 key）→ `Leaderboard.tsx`（表头与 `DeveloperRow`）→ `LeaderboardSkeleton.tsx` → `api/leaderboard/route.ts`。

**关键**：`Tokens` / `Cost` 表头是数字缩写切换而非排序按钮，加列时不得改变该语义。

**完成判据**：两列正确渲染且空值留空；排序与格式切换无回归；移动端无横向溢出。

---

## T7 Teamboard 页面 `L`

**依赖**：T3、T6

`app/(main)/teamboard/page.tsx`、`components/teamboard/`、`lib/teamboard/getTeamboard.ts`、`api/teamboard/route.ts`、OG metadata。

筛选状态走 URL searchParams。

**可见范围**（D-2 已确认）：筛选器列出「`public` 且未解散的团队 ∪ 调用者所属团队」；`/api/teamboard` 对他人的 `private` 团队返回 **404**（而非 403，避免探测团队是否存在）；未登录访客可浏览公开团队。

**完成判据**：Team 单选 + Group 多选组合筛选正确；列序 `# / Developer / Group / Tokens / Cost`；未选团队时显示引导空态；他人 `private` 团队既不出现在筛选器中，直接请求也返回 404。

---

## T8 Profile 团队区块 `M`

**依赖**：T4

`components/profile/types.ts`（加 `isOwner`）、`ProfileView.tsx`、`lib/publicProfileData.ts`、退出操作 API 对接。

**完成判据**：本人可见退出按钮，他人不可见；无归属时不渲染区块；admin 退出被正确阻断。

注：D-4 已确认取「删除」，因此「适配」路径所需的公开 profile API Team / Group response contract 工作**取消**——iOS App 从站点移除，不再需要为它做实对外契约。

---


## T10 i18n 基础设施 `L`

**依赖**：T0

1. 新建 `web/src/lib/i18n/`：字典按页面分文件（`leaderboard.ts`、`teams.ts`、`auth.ts` 等），导出 `en` / `zh` 两份；`t(key, vars?)` 支持 `{var}` 插值
2. `tt_locale` cookie（httpOnly=false，1 年），服务端经 `cookies()` 读，客户端经 React context 取初始值
3. `app/layout.tsx`：`<html lang>` 从硬编码 `en` 改为跟随 `tt_locale`
4. 导航右上角主题切换左侧新增语言切换按钮（地球图标 + 下拉：English / 中文）
5. `worker.ts`：`caches.default` 的 synthetic cache key 并入 `tt_locale` 值；`u/[username]` 的 `unstable_cache` key 与 R2 渲染页缓存同理
6. `lib/format.ts` 的 `Intl.NumberFormat("en-US")` 硬编码改为跟随当前 locale

**关键约束**：不引入 next-intl 等运行时依赖；URL 不加语言前缀，路由不变。

**完成判据**：切换语言后已包裹页面文案跟随；两种语言各自缓存不错串；`<html lang>` 正确；`bun run typecheck` 通过。

---

## T11 全站文案包裹 `L`

**依赖**：T10，且 T1 / T2 / T5 / T6 / T7 / T8 已完成（它们产出需要包裹的文案）

逐页把硬编码字符串收进字典：Leaderboard、Teamboard、Teams、Profile、Settings、Docs、Privacy、Terms、认证页（register / login / forgot-password / reset-password / verify-email）、device 页、空态 / 错误态、toast、导航、页脚。邮件模板统一英文（本次只做 UI 语言切换，不引入按收件人语言的偏好，也不在 `users` 表加 `locale` 列）。

**Privacy / Terms** 的中文版为便利翻译，页内标注「英文版本为准」。

**保留页面的保真机制**：对 Docs / Privacy / Terms 等不改动内容的页面，字典的 `en` 值必须与原文案**逐字一致**——这样 default-English 的渲染产物与线上 tokens.ci 一致。源码会因 i18n 包裹而变化，但渲染产物不变；此处的「一致」指渲染产物，不是源码逐行不动。

**完成判据**：全站无遗漏硬编码文案（`rg` 扫描 JSX 文本节点无裸字符串）；`<html lang>` 正确；两种语言下所有页面完整渲染；Privacy / Terms 有「英文为准」标注。

---

## T12 品牌图标底色改紫 `S`（独立小改动，与本次重构无关）

**依赖**：T0

把品牌蓝色 `#2F6FDB` 改为紫色 `#7C3AED`（已确认），白色 T 图案不变。**本轮只改页面本体**：`Navigation.tsx` 的 `TokensMark` 内联 SVG + demo 同款品牌 tile。`tokens-mark.svg`（currentColor 单色版）不改。

**本轮不改（浏览器 / 安装图标，另一套 surface）**：`web/public/brand/tokens-favicon.svg` 是**文本 SVG**（非二进制，顺手可改，但属浏览器标签页图标，本轮不动）；位图资源（`favicon.ico`、`favicon-16x16.png` / `32x32.png`、`android-chrome-192x192.png` / `512x512.png`、`brand/tokens-app-icon-180.png` / `1024.png`、`brand/tokens-mark-rounded.png`）是**二进制资产**，仓库没有 SVG→PNG 生成流程，没有生成流程或明确要求之前不重着色。代价是标签页 / 安装图标暂时仍是蓝色——要同步需另建位图生成流程，由你明确点头后再做。

**完成判据**：导航品牌块（`TokensMark`）与 demo 同款 tile 均为紫色；白色 T 不变；`tokens-mark.svg` 未改动；favicon / 安装图标的位图本轮未改并在交付说明中写明。建议单独提交，与本次重构的 diff 分开。

---

## T9 文档与收尾 `M`

**依赖**：T1–T8、T10、T11

同步 `.trellis/spec`、README；把 `docs/upstream_policy.md` §2.2「计划态分歧区」里**已落地**的行逐条移入 §2.1 现状表并从 §2.2 删除；解除 §3 / §3.2 / §5 中带"T2/T3 落地后才适用"前缀的条目；最终回归。

复核 §2 开头的"代码零分歧"结论时，用该节给出的那两条命令（**两点语法，pathspec 为 `web`**）：

```bash
git diff --stat upstream/main HEAD -- web cli packages
git status --short -- web cli packages
```

不要用三点 `upstream/main...HEAD`——它比较的是 merge-base 与 HEAD，会忽略 `upstream/main` 独有的提交；`web/src` 也漏掉 `web/worker.ts`、`web/wrangler.jsonc`、`web/scripts/`。T0–T9 落地后这两条命令必然不再是空输出，届时需改写 §2 开头那段。

**完成判据**：`bun run lint`、`bun run typecheck`、`bun run test:migrations` 全绿；docs PRD §12 验收清单逐条勾选。

---

## 验证策略

| 层 | 手段 |
|---|---|
| 迁移 | `bun run test:migrations` |
| 类型 / 规范 | `bun run typecheck`、`bun run lint` |
| 领域不变式 | 针对 INV-1..INV-10 的聚焦测试。**Rust 侧已有内联测试框架并由 CI 运行，但 web/TS 侧确实没有测试框架**，`T4` 内为 TS 领域逻辑引入最小 runner |
| 用户可见行为 | `web/features/*.feature` 场景 |
| CLI 兼容 | `tokens login` + `tokens submit` 真机冒烟 |
| UI | 对照 `docs/demo/teamboard-demo.html`，桌面 + 390px 两档 |
