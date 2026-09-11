# Teamboard teams and email authentication

> 完整方案见 `docs/prd-teamboard-teams-auth.md`。本文件只保留 Trellis 生命周期所需的需求、约束与验收标准，不复制技术细节。

## Goal

把产品从"个人全局榜"扩展为"个人榜 + 组织榜"，同时把身份体系从依附 GitHub 改为自有邮箱账号。两者互相牵连：只有先有自有账号（邮箱 + 用户名），"按用户名或邮箱邀请成员"才成立。

实施分支：`feature/teamboard-teams-auth`

## Requirements

- **R1** 删除 Hall of Shame 页面功能，保留封禁机制本身。
- **R2** 在原 Hall of Shame 导航位新增 `Teamboard` 页面。
- **R3** 新增 Team / Group 两级组织体系，含 admin / subadmin 角色、邀请（可搜索下拉多选，含未注册邮箱路径）、以及"先解散后删除"的两段式生命周期。
- **R4** 认证由 GitHub OAuth 改为邮箱注册登录；密码需 ≥8 位且同时含大写、小写与至少 1 个特殊字符；移除全部 GitHub 入口。
- **R5** Leaderboard 在 `Developer` 后新增 `Team`、`Group` 两列；Teamboard 提供 Team 单选 + Group 多选筛选，且不含 `Team` 列。
- **R6** Profile 展示 Team / Group 并支持本人退出。
- **R7** 其余功能保持现状。
- **R8** 导航右上角主题切换左侧新增语言切换按钮（English / 中文），全站文案做中英文适配。
- **R9** 品牌块（导航左上角 TokensMark）蓝色底色改为 `#7C3AED`，白色 T 图案不变；favicon / 安装图标本轮不重着色（独立小改动，单独提交）。

## Constraints

- **CLI 契约不可破坏**：device flow 三端点、`tt_` token 格式、`credentials.json`、`TOKENS_API_TOKEN` / `TOKENS_API_URL` 全部不变。Rust 代码零改动。
- **Workers 运行时限制**：bcrypt / argon2 等原生模块不可用，密码哈希用 WebCrypto PBKDF2；SMTP 不可用，邮件走 HTTP API。
- **不新增 npm 运行时依赖**，以降低后续与上游同步的成本。
- **迁移 additive**：不删除既有列；新表回滚即 `DROP TABLE`。
- **组件只来自 `web/src/components/ui/`**，颜色只走语义 token，数字加 `.tabular`（见 `docs/upstream_policy.md`）。

## Key Domain Invariants

| # | 不变式 |
|---|---|
| INV-1 | 一个 User 至多属于一个 Team |
| INV-2 | Group 必属于某 Team；其成员必须先是该 Team 成员 |
| INV-3 | 一个 User 在其 Team 内至多属于一个 Group |
| INV-4 | `status='active'` 的 Team 恰有一个 admin（`disbanded` 的 Team 无成员、无 admin） |
| INV-5 | 每个 Team 至多 2 个 subadmin |
| INV-6 | `Delete` 仅在 `status='disbanded'` 且成员数为 0 时允许 |
| INV-7 | `Disband` 清空全部成员行（含 admin 行）；`created_by` 是 `disbanded` Team 唯一的删除授权主体 |
| INV-8 | 退出 Team 时自动退出其 Group |
| INV-9 | 封禁用户不出现在任何榜单 |
| INV-10 | 非成员只能浏览 visibility='public' 且 status='active' 的 Team；成员始终可浏览自己的 Team，无论其可见性 |

## Child Task Mapping

本任务按 `docs/prd-teamboard-teams-auth.md` §11 拆为 T0–T12。每个子任务的依赖写在其自身产物中，不靠父子树位置隐含。

推荐顺序：`T0 → T12 → T1 → T2 → T3 → T4 → T6 → T10 → T5 → T7 → T8 → T11 → T9`

`T12` 紧挨 `T0` 之后、`T1` 之前：三者都改 `Navigation.tsx`（T12 改 TokensMark、T1 改 NAV_LINKS、T2 改 GitHub 入口），串行以免三方冲突。`T10` 插在 `T5` 之前，让新页面从第一天写字典 key；`T11` 收编既有页面，放在 `T8` 之后、`T9` 之前。

## Acceptance Criteria

- [ ] `/shame` 返回 404，全仓无 Hall of Shame 残留引用，封禁机制仍生效
- [ ] 导航第二项为 `Teamboard`，指向 `/teamboard`
- [ ] 可用邮箱 + 用户名 + 密码注册；弱密码被拒并给出具体原因
- [ ] 全站不存在 GitHub 图标、按钮或 OAuth 入口
- [ ] `tokens login` 与 `tokens submit` 无需更新 CLI 即可工作
- [ ] 创建 Team 者成为 admin；可改名、改头像、邀请（用户名 / 邮箱 / 批量）
- [ ] subadmin 最多 2 个，且能做除解散与删除 Team 外的全部操作（含 Group 全部操作）
- [ ] 未解散的 Team / Group 无法删除；解散清空成员；已解散实体仍可由创建者删除
- [ ] Leaderboard 列序 `# / Developer / Team / Group / Tokens / Cost`，空值留空，排序无回归
- [ ] Teamboard 支持 Team 单选 + Group 多选，列序 `# / Developer / Group / Tokens / Cost`
- [ ] 建团表单强制选择可见性，未选择时落库 `private`；筛选器列出「公开团队 ∪ 本人所属团队」，请求他人 `private` 团队返回 404
- [ ] Profile 展示 Team / Group，本人可退出，无归属时不渲染该区块
- [ ] Docs 页删去「The verified badge」「Architecture」「Sponsors」「iOS app」四节，且全站不再展示 iOS App 入口（D-4 取删除）；Privacy / Terms / Settings / embed / badge / archive 行为无变化
- [ ] Docs 保留章节的 default-English 渲染内容与线上 tokens.ci 一致（基准是线上页而非 demo；源码因 i18n 包裹变化，渲染产物不变）
- [ ] 邀请对话框为可搜索下拉多选（勾选框 + 计数），输入完整邮箱可邀请未注册用户
- [ ] 语言切换按钮存在且提供 English / 中文；切换后全部页面文案跟随，`tt_locale` 持久化，`<html lang>` 跟随，缓存按 locale 分离
- [ ] 品牌块（导航左上角 `TokensMark` + demo 同款）蓝色底色改为紫色，白色 T 图案不变；favicon / 安装图标的位图本轮不重着色（无二进制生成流程），已明确说明未改
- [ ] `bun run lint`、`bun run typecheck`、`bun run test:migrations` 全绿

## Open Decisions

四项决策（D-1 / D-2 / D-3 / D-4）均已由用户明确选择，无待确认项，无阻塞。详见 docs PRD §13。

| # | 议题 | 取值 | 状态 |
|---|---|---|---|
| D-1 | 成员可否属于多个 Group | 否——一人一 Group，暂不支持一对多 | 已确认 |
| D-2 | Teamboard 的团队可见范围 | 公开团队可浏览 + 建团时选可见性 | 已确认 |
| D-3 | `verified` 徽章去留 | 选项 C——直接移除徽章 | 已确认 |
| D-4 | iOS App 的去留与适配 | 删除——从站点移除 iOS App 引用 | 已确认 |

D-1 取「一人一 Group」，因此 `group_members.user_id` 保留 UNIQUE（INV-3），`Group` 列单值渲染，
榜单两组 LEFT JOIN 相对 `users` 严格 0..1，不放大行数。改为多组属数据模型级返工，本次不预留结构。

D-3 取 C（直接移除徽章）。范围**严格限定为徽章本身**：删除 socialVerification.ts 与
VerifiedBadge.tsx，移除 6 个文件里的引用（含 Docs 页「The verified badge」章节）。
`social_links` / `social_links_synced_at` 两列、ProfileSocialLinks.tsx、githubSocials.ts 与
每日 cron 任务本身**保留**——Profile 页社交链接图标行仍在使用它，与徽章无代码耦合。
因此 **C 不需要任何新迁移，也不触碰「迁移 additive、不删既有列」约束**。但徽章的移除会
**连带三个文件必改**：`lib/cron/refreshSocialLinks.ts` 去掉 `isVerifiedBySocialLinks`
import、`verified` 计数与返回类型字段，其下游 `api/cron/refresh-social-links/route.ts`
与 `web/worker.ts`（不在 `web/src` 下，最易漏改）同步去掉对 `verified` 的解构与日志——
三处不同步改则 typecheck / build 必然失败。`email_verified_at` 照常新增供注册流程使用，
逐文件清单见 docs PRD §13 D-3 详述。

D-2 采用折中方案：`teams.visibility` 取 `public` / `private`，建团时由 admin 显式选择，
admin 与 subadmin 事后可改。DB 默认 `private`（fail-safe）。非成员只能浏览公开且未解散的团队，
成员始终可浏览自己的团队（INV-10）。不取「仅本人所属 Team」是因为受 INV-1 约束单选交互会形同虚设；
不取「无条件全公开」是因为暴露范围不由团队控制，且收紧时需回收已公开数据。

## Gate Records

- **DDD Boundary Review**: `confirmed`（`grill-with-docs` 未完整调用——环境未暴露该 Skill；边界基于源码、迁移历史与线上页面独立建立）
- **DDIA Data Design Review**: `confirmed`
- **BDD**: 场景落于 `web/features/*.feature`，中文场景文本 + 英文 Gherkin 关键词
- **Planning review**: 用户于 2026-09-11 确认 PRD、demo、紫色 hex `#7C3AED`，并授权进入实施。`grill-with-docs` 未完整调用（环境未暴露该 Skill）；DDD Boundary Review 已基于源码 / 迁移 / 线上页独立 `confirmed`。
