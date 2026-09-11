# Teamboard / Teams / Auth 实施 TODO

本文件是**推进依据**。每完成一项立刻勾选并写完成日期；新增或调整任务同步改这里，不要只改聊天记录。

完整需求与技术细节：`docs/prd-teamboard-teams-auth.md`  
Trellis 父任务：`.trellis/tasks/09-10-teamboard-teams-auth/`  
UI 对照：`docs/demo/teamboard-demo.html`  
上游策略：`docs/upstream_policy.md`  
本地数据库：`docs/local-dev-database.md`（OrbStack Compose，禁止 Homebrew Postgres）
实施分支：`feature/teamboard-teams-auth`

## 维护规则

- `[ ]` 未开始 · `[~]` 进行中 · `[x]` 已完成（括号内写完成日期）
- 当前指针只指向**一项**正在做的任务；做完再往下走
- 推荐顺序不可并行的项：`T12` / `T1` / `T2` 都改 `Navigation.tsx`；`T1` / `T2` 都改 Docs 页
- 完成一项后：勾选本文件 → 更新对应 Trellis 子任务 → 再开下一项
- 若计划变更（拆分、合并、加项），先改本文件「变更日志」，再改 Trellis 产物

## 当前指针

**当前：无 active 任务。** T2 已 archive `--no-commit` → `.trellis/tasks/archive/2026-09/09-11-t2-email-auth/`（`status=completed`，`commit=5aa99c88`）。未 start T3。T0 `02504efe`、T12 `d5cf8604`、LICENSE `f24b60d1`、T1 `6759e183`、T2 代码 `5aa99c88`、T2 SHA 元数据 `7669eab1` 均未 push。

推荐顺序：

`T0 → T12 → T1 → T2 → T3 → T4 → T6 → T10 → T5 → T7 → T8 → T11 → T9`

---

## 规划（已完成）

- [x] 仓库调研 + 线上 tokens.ci UI 核对（2026-09-10）
- [x] 写 PRD `docs/prd-teamboard-teams-auth.md`（2026-09-10，2026-09-11 确认）
- [x] 写 HTML demo `docs/demo/teamboard-demo.html`（2026-09-10）
- [x] 改写 `docs/upstream_policy.md`（2026-09-10）
- [x] BDD 场景 `web/features/*.feature`（2026-09-10）
- [x] Trellis 父任务产物 `prd.md` / `design.md` / `implement.md`（2026-09-10）
- [x] 决策 D-1 一人一 Group、D-2 可见性折中、D-3 移除徽章、D-4 删除 iOS（2026-09-10）
- [x] 紫色 hex 锁定为 `#7C3AED`；范围收窄为页面 TokensMark + demo tile（2026-09-11）
- [x] 用户确认 PRD / demo，授权进入实施（2026-09-11）

---

## 流程门（进入实施前）

- [x] 父任务 jsonl 按 1.3 收成 `.trellis/spec/**`（去掉源码路径）（2026-09-11）
- [x] 创建 Trellis 子任务 T0–T12（`--parent`，依赖写在子任务产物里）（2026-09-11）
- [x] T0 自有 `prd.md` / `implement.md` + 非空 `implement.jsonl` / `check.jsonl`（2026-09-11）
- [x] `task.py start` **T0**（不要 start 父任务；父任务只做规划与最终集成）（2026-09-11）
- [x] 本文件建立后，按当前指针推进（2026-09-11）

---

## 执行任务

### T0 基线修复 `S`

**状态**：已 archive（2026-09-11，`--no-commit`；git `02504efe`）
**依赖**：无  
**阻塞**：T1、T2、T3、T10、T12  
**目的**：让 `db:generate` / `test:migrations` 与现实一致，否则后续迁移会产出错误 diff。

- [x] `web/scripts/check-migrations.ts`：去掉对已删除表 `groups` / `group_members` / `group_invites` 的断言（2026-09-11）
- [x] `web/src/lib/db/migrations/meta/`：补齐 `0022` / `0023` snapshot，使最新 snapshot 编号等于 journal 尾部 idx（23）（2026-09-11）
- [x] 验证：`bun run test:migrations` 通过（2026-09-11，本地 pg_ctl 一次性起停，未用 brew services）

**完成判据**：迁移检查全绿；脚本不再断言已删 group 表存在。

---

### T12 品牌图标底色改紫 `S`（独立小改动，建议单独提交）

**状态**：已提交（2026-09-11，`d5cf8604`）
**依赖**：T0  
**文件冲突**：与 T1 / T2 同改 `Navigation.tsx`，必须在 T1 之前做完，禁止并行。

- [x] `Navigation.tsx` 的 `TokensMark`：`fill="#2F6FDB"` → `#7C3AED`，白色 T 不变（2026-09-11）
- [x] demo 同款品牌 tile 已是 `#7C3AED`（规划阶段已改，实施时核对）
- [x] **不改**：`tokens-mark.svg`（currentColor）；`tokens-favicon.svg`（文本 SVG，属标签页图标）；全部 favicon / app-icon 位图
- [x] **不要**全局替换 `#2F6FDB`（头像 / 团队渐变 / AVA 调色板在用）
- [x] 单独提交，与重构 diff 分开（`d5cf8604`）

**完成判据**：页面 Tokens 前图标为紫底白 T；favicon / 安装图标仍蓝，交付说明里写明未改。

---

### T1 删除 Shame + 导航占位 `S`

**状态**：进行中（实现完成，等 3.4）
**依赖**：T0、T12

- [x] 删除 `web/src/app/(main)/shame/page.tsx`、`web/src/components/shame/BannedList.tsx`
- [x] 改写引用：`Navigation.tsx`（NAV_LINKS）、`BannedProfileView.tsx`、Docs、Terms、`worker.ts` PAGE_CACHEABLE、`error.tsx`、`schema.ts` 注释
- [x] Docs 页删除 Architecture / Sponsors / iOS app 三节（verified 章节留给 T2）；`TESTFLIGHT_URL` 与 OG 里 iOS 引用一并删
- [x] **不动** `ServiceFooter.tsx`（页脚 Sponsors 与 Docs 页 Sponsors 是两处内容）
- [x] 导航第二项换成 Teamboard，先接占位页 `/teamboard`
- [ ] 验证：`/shame` 404；`rg -i "hall of shame|/shame"` 无残留；封禁机制仍生效（源码已无 `/shame`；404 等 3.4 后本地预览）

---

### T2 认证改造 `L`

**状态**：`[x]` archive `--no-commit`（2026-09-11）。路径：`.trellis/tasks/archive/2026-09/09-11-t2-email-auth/`
**依赖**：T0（与 T1 串行：两者都改 Docs 页）

- [x] 迁移 `0024_add_password_auth.sql`（password_hash、email_verified_at、github_id 可空、email 部分唯一索引、verification tokens 表）
- [x] `lib/auth/password.ts`：WebCrypto PBKDF2（210k 次，格式 `pbkdf2$sha256$210000$…`）
- [x] `lib/email/`：Resend HTTP，不引 SDK；失败走 waitUntil，不回滚业务
- [x] 页面与 API：`/register` `/login` `/forgot-password` `/reset-password` `/verify-email` + `POST /api/auth/resend-verification`
- [x] 删除 GitHub OAuth 全链路与全部 GitHub 入口
- [x] 头像 fallback 改为用户名首字母占位图
- [x] 移除 verified 徽章（D-3）：删 `socialVerification.ts`、`VerifiedBadge.tsx`，改 6 处引用；**必须改** `refreshSocialLinks.ts`、`api/cron/refresh-social-links/route.ts`、`web/worker.ts`（不在 `web/src` 下）
- [x] **保留**：`tt_session`、device flow、`social_links` 列、Profile 社交链接图标行、cron 任务本身
- [x] secret：`RESEND_API_KEY`、`EMAIL_FROM` 注释（部署前 `wrangler secret put` 这两个值）；Rate Limiting 用 wrangler `namespace_id` 1001，已写进配置，无需单独建 CF namespace
- [x] 验证：可注册登录；弱密码被拒；`bun run lint` / `typecheck` / `build` / `test:migrations` 过
- [x] `tokens login` + `tokens submit`：本地隔离配置跑通（2026-09-11）。`TOKENS_API_URL=http://localhost:3000` + `TOKENS_CONFIG_DIR=/tmp/tokens-t2-cli-e2e`；生产 `~/.config/tokens/credentials.json` mtime 未变。未对生产 submit。

---

### T3 团队数据模型 `M`

**状态**：未开始  
**依赖**：T0

- [ ] 迁移 `0025_add_teams_and_groups.sql`（teams / team_members / groups / group_members / team_invitations）
- [ ] `teams.visibility` 默认 `private`；部分索引 `teams_public_active_idx`
- [ ] Drizzle `schema.ts` + `lib/teams/types.ts`
- [ ] 验证：迁移正向成功；`typecheck` + `test:migrations` 通过

---

### T4 Team/Group 领域服务与 API `L`

**状态**：未开始  
**依赖**：T2、T3

- [ ] `web/src/lib/teams/` 领域服务；API 见 PRD §9.2
- [ ] `GET /api/users/search`：仅 admin/subadmin；不回显 email；完整邮箱才精确匹配
- [ ] 落实 INV-1..INV-10；并发按 design.md §6；可见性 `canViewTeam` 一处 helper
- [ ] 验证：权限矩阵、解散/删除前置、subadmin 上限、邀请去重与过期、退出 Team 级联退出 Group

---

### T5 Team 管理页面 `L`

**状态**：未开始  
**依赖**：T4（建议等 T10，新页面直接写字典 key）

- [ ] `/teams` + `components/teams/`；对照 demo Teams 页
- [ ] 补 shadcn：select / dialog / checkbox / popover / form / label / textarea
- [ ] 邀请：可搜索下拉多选（勾选 + 计数 + 未注册邮箱路径）
- [ ] 建团强制选可见性；危险操作名称二次确认
- [ ] 验证：全部 Team / Group 操作可走通

---

### T6 Leaderboard 加两列 `M`

**状态**：未开始  
**依赖**：T3

- [ ] types → getLeaderboard 两组 LEFT JOIN + 缓存 key → Leaderboard.tsx → Skeleton → API
- [ ] 列序 `# / Developer / Team / Group / Tokens / Cost`；空值留空
- [ ] Tokens / Cost 表头仍是数字格式切换，不是排序
- [ ] 移动端 Team / Group 降为 `@username` 下徽章行
- [ ] 验证：排序与格式切换无回归；390px 无横向溢出

---

### T7 Teamboard 页面 `L`

**状态**：未开始  
**依赖**：T3、T6（共用列渲染）

- [ ] `/teamboard` 真页面替换 T1 占位；筛选走 URL searchParams
- [ ] 列序 `# / Developer / Group / Tokens / Cost`（无 Team 列）
- [ ] 筛选器 = 公开未解散 ∪ 本人所属；他人 private → 404
- [ ] 未选团队显示引导空态
- [ ] 验证：单选 Team + 多选 Group 组合正确

---

### T8 Profile 团队区块 `M`

**状态**：未开始  
**依赖**：T4

- [ ] Profile 加 Team / Group；无归属不渲染
- [ ] 仅本人可见退出；admin 退出被阻断（须先移交或解散）
- [ ] 退出 Team 级联退出 Group
- [ ] 验证：他人 / 未登录看不到退出按钮

---

### T10 i18n 基础设施 `L`

**状态**：未开始  
**依赖**：T0  
**插入点**：T5 / T7 之前，让新页面从第一天写字典 key

- [ ] `web/src/lib/i18n/` 自建字典 + `t()`；不引入 next-intl
- [ ] `tt_locale` cookie；`<html lang>` 跟随
- [ ] 导航主题切换左侧：地球图标 + English / 中文
- [ ] worker / unstable_cache / R2 缓存 key 并入 locale
- [ ] `Intl.NumberFormat` 跟随 locale
- [ ] 验证：两种语言缓存不错串；`typecheck` 通过

---

### T11 全站文案包裹 `L`

**状态**：未开始  
**依赖**：T10，且 T1 / T2 / T5 / T6 / T7 / T8 已完成

- [ ] 逐页把硬编码字符串收进字典
- [ ] Docs / Privacy / Terms 的 `en` 值与原文案逐字一致（渲染产物对齐线上）
- [ ] Privacy / Terms 中文页标注「英文版本为准」
- [ ] 邮件模板保持英文（不加 `users.locale`）
- [ ] 验证：JSX 无裸中英文字符串遗漏；两语言完整渲染

---

### T9 文档与收尾 `M`

**状态**：未开始  
**依赖**：T1–T8、T10、T11

- [ ] 同步 `.trellis/spec`、README
- [ ] `upstream_policy.md` §2.2 已落地行移入 §2.1；解除 T2/T3 前缀条目
- [ ] 用两点语法复核代码分歧（`git diff --stat upstream/main HEAD -- web cli packages`）
- [ ] 父任务验收清单逐条勾选
- [ ] 验证：`bun run lint`、`bun run typecheck`、`bun run test:migrations` 全绿

---

## 明确不做（本次）

- 不改 Rust CLI
- 不改 submissions / daily_breakdown 采集语义
- 不做团队级用量提交
- 不做 SSO / 组织付费 / 跨 Team 成员共享
- 不重着色 favicon / 安装图标位图（无生成流程）
- 不全局替换 `#2F6FDB`
- 不删 `ServiceFooter.tsx` 的 Sponsors / Built on Tokscale
- 不把 `social_links` 列或 Profile 社交图标行一并删掉

---

## 验证命令（各任务按需）

```bash
bun run lint
bun run typecheck
bun run test:migrations
# T2 / 徽章漏改还会在 build 暴露：
bun run build
```

报告型测试不要用 `rtk` 缓存当唯一证据。本地 `test:migrations` 用 OrbStack 容器，见 `docs/local-dev-database.md`。

---

## 本地开发数据库（OrbStack）

- Compose：`/Users/lusonglin/docker-compose/tokens/docker-compose.yml`
- 容器：`tokens-postgres`（`postgres:16`），只绑 `127.0.0.1:5433`（本机 5432 已被 `keyboy-play-local-db` 占用，不抢）
- `DATABASE_URL=postgresql://tokens:tokens@127.0.0.1:5433/tokens`
- 禁止再 `brew install postgresql@*` / `brew services start postgresql@*`
- 明细：`docs/local-dev-database.md`

---

## 变更日志

| 日期 | 变更 |
|---|---|
| 2026-09-11 | 建立本文件。顺序定为 `T0 → T12 → T1 → …`（T12 提前以避免 Navigation.tsx 三方冲突）。紫色 hex 锁定 `#7C3AED`。 |
| 2026-09-11 | T0 实现完成：去掉 check-migrations 对已删 group 表的断言；补 0022/0023 snapshot；`test:migrations` 全绿。Homebrew postgresql@16 已 `brew services stop`，验证用 pg_ctl 起停。**未切 T12**：仍待 check / update-spec / scoped commit。 |
| 2026-09-11 | 按授权卸干净 Homebrew `postgresql@16` 并删除 `/opt/homebrew/var/postgresql@16`。卸载时 Homebrew **自动**卸掉依赖 `krb5`（未执行 `brew autoremove`）。本地库改为 OrbStack Compose：`/Users/lusonglin/docker-compose/tokens/`，端口 `127.0.0.1:5433`。约定写入 `docs/local-dev-database.md`。 |
| 2026-09-11 | T2 实现 + check 通过：邮箱密码认证、删 GitHub OAuth 与 verified 徽章、resend-verification。CLI 全链路未跑。等 3.4 确认 commit，未 start T3。 |
| 2026-09-11 | T2 已提交 `5aa99c88`（未 push）。未 start T3。 |
| 2026-09-11 | T2 archive `--no-commit` → `.trellis/tasks/archive/2026-09/09-11-t2-email-auth/`。CLI `tokens login`+`submit` 仍为 `[ ]`。未 start T3。 |
| 2026-09-11 | 本地隔离 `tokens login` + `tokens submit` 对 `localhost:3000` 跑通；生产 credentials 未改写。未 start T3。 |
