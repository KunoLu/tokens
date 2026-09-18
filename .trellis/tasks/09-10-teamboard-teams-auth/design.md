# 技术设计

> 数据结构、API 清单与迁移 DDL 的完整定义在 `docs/prd-teamboard-teams-auth.md` §7–§10。本文件只记录实现层面的关键决策与理由。

## 1. 密码哈希：为什么是 PBKDF2

Workers 上 bcrypt / argon2 是原生模块，不可用。候选只剩：

| 方案 | 结论 |
|---|---|
| `node:crypto` scrypt | `nodejs_compat` 下 API 存在但 CPU 开销易触碰 Workers 的 CPU 时间上限，且实现完整度不稳 |
| 第三方纯 JS bcrypt | 慢且引入新依赖，违反"不新增运行时依赖" |
| **WebCrypto PBKDF2-HMAC-SHA256** | **选用**：SubtleCrypto 原生、零依赖、CPU 可控 |

参数：210,000 次迭代（OWASP 2023 对 PBKDF2-HMAC-SHA256 的推荐值），每用户 16 字节随机 salt。

存储格式带算法前缀，为将来换算法留出迁移路径：

```
pbkdf2$sha256$210000$<salt_b64>$<hash_b64>
```

校验用 `crypto.subtle` 派生后以 `timingSafeEqual` 比较（`node:crypto` 已在用，见 `lib/auth/utils.ts`）。

## 2. 会话层不动

`tt_session` 的签发、存储、校验、过期逻辑完全保留。改造只替换"**如何确认用户身份**"这一步——从"OAuth callback 拿到 GitHub 用户"变成"校验邮箱 + 密码"。`createSession` / `setSessionCookie` / `getSession` 的签名与行为不变。

这个约束是刻意的，它直接带来两个结果：device flow 无需改动，CLI 无需改动。

## 3. 榜单 JOIN 不放大行数

`team_members.user_id` 与 `group_members.user_id` 都是 UNIQUE，因此两组 LEFT JOIN 相对 `users` 严格是 0..1 关系，不会让既有的 `SUM()` 聚合翻倍。这是选择"一人一 Team、一人一 Group"模型的直接工程收益。**决策 D-1 已确认取该模型**，暂不支持一对多；若日后改为多对多，必须改写为子查询聚合，否则所有历史数字都会错。

缓存 key 需纳入 `teamId` / `groupIds`；成员与角色变更时 `revalidateTag('leaderboard')`。

## 4. 解散与删除为什么必须分离

需求同时要求"解散把所有成员踢出"和"成员为 0 才能删除"。若 admin 也被踢出，就没有任何人有权删除——这是一个死锁。

解法是把**成员身份**与**创建者身份**分离：

- `team_members` 在解散时全量清空（含 admin 行）
- `teams.created_by` 不随解散清除
- 删除鉴权读 `created_by`，而非读 `team_members.role`

由此"每个 Team 恰有一个 admin"只在 `status='active'` 时成立（INV-4）。`disbanded` 是无成员、无 admin 的终态，任何走 `team_members.role` 的鉴权在该状态下都必然判定无权限，删除路径必须单独读 `created_by`。

Group 同理，只是删除权归 Team 的 admin / subadmin。

## 5. 团队可见性（D-2）

需求只说了 Teamboard 有「Team 单选」，没说谁能选到哪些团队。这是授权决策，单独确认后取折中方案。

两个极端都不成立：

| 取值 | 问题 |
|---|---|
| 仅本人所属 Team | 受 INV-1（一人至多一个 Team）约束，筛选器只会有一个选项，**单选交互形同虚设**，等于没实现需求 |
| 全部团队无条件公开 | 团队成员构成变成可爬取的公开数据，暴露范围不由团队控制；且日后收紧要回收已公开数据，成本远高于放开 |

取值：`teams.visibility ∈ {public, private}`，建团时由 admin 显式选择，admin / subadmin 事后可改。

三个刻意的设计选择：

1. **DB 默认 `private`**。任何绕过表单的写入路径（脚本、迁移回填、日后新增的 API）都不会意外公开数据。表单层强制二选一，两层各自独立成立。
2. **成员始终可见自己的团队，无论可见性**（INV-10）。否则 `private` 团队的成员自己也用不了 Teamboard。
3. **越权访问返回 404 而非 403**。403 等于承认「这个 team 存在但你没权限」，可被用来枚举团队；404 不泄露存在性。

鉴权集中在单个 `canViewTeam(viewer, team)` helper，由团队详情、成员列表、分组列表、Teamboard 四类读接口共用。分散判断是这类漏洞的常见来源。

**与 Leaderboard 的关系**：`Team` / `Group` 列是明确需求，团队名对所有访客可见，`private` 也不例外。可见性控制的是「能否按团队聚合浏览完整成员名单」，不是「团队名是否公开」。若日后要求 `private` 团队在 Leaderboard 隐名，属额外需求。

## 6. 并发点

| 场景 | 处理 |
|---|---|
| subadmin 名额（上限 2） | 带计数条件的原子 `UPDATE`：`WHERE (SELECT count(*) FROM team_members WHERE team_id=? AND role='subadmin') < 2` |
| 接受邀请 | 条件更新 `WHERE status='pending'`，受影响行数为 0 即视为已被处理 |
| 批量邀请重复 | 事务内先把用户名 / 邮箱归一为 `invited_user_id`，再依赖两个条件互斥的部分唯一索引：已注册用户走 `team_invitations_pending_user_unique`，未注册地址走 `team_invitations_pending_email_unique`（其 `WHERE` 含 `invited_user_id IS NULL`）。冲突即跳过 |
| 未注册邮箱注册后回填 | 有序四步，颠倒会先撞 `pending_user_unique`：① 按 Team `SELECT ... FOR UPDATE` 锁定指向该邮箱或该用户的 pending 行（每 Team 最多 2 行）② 定 winner = `created_at` 最早，并列取小 `id`，**不预设邮箱行更早或更晚** ③ **先**把 loser 置 `superseded` ④ **再**回填 winner；winner 是用户行时无需回填 |
| 解散 / 删除 | 单事务内先校验状态再变更 |

回填**不放进注册事务**，而是账号提交后的独立事务，失败不影响注册。之所以能这样，是因为
`GET /api/me/invitations` 同时匹配 `invited_user_id = 我` 与
`invited_user_id IS NULL AND lower(invited_email) = lower(我的邮箱)`——回填只是归一化，
不是用户能否看到邀请的前提，因此天然幂等、可重试。

## 7. 邮件

Resend HTTP API，原生 `fetch` 调用，不引 SDK。发送放在 `waitUntil` 中，失败不回滚业务事务——注册成功但验证邮件发送失败时，用户仍可用"重发验证邮件"自救，比整个注册回滚更好。

新增 secret：`RESEND_API_KEY`、`EMAIL_FROM`。

## 8. 前端

- 新增 shadcn 组件：`select`、`dialog`、`checkbox`、`popover`、`form`、`label`、`textarea`，用已有的 `shadcn` CLI vendored 到 `components/ui/`。
- Teamboard 与 Leaderboard 共用行渲染与列组件，避免两处各写一遍列逻辑。`T7` 因此依赖 `T6`。
- 筛选状态走 URL `searchParams`（与现有 period / sortBy / search 一致），保证可分享、可后退。
- 移动端：Team / Group 不占独立列，降级为 `@username` 下方的徽章行。已在 `docs/demo/teamboard-demo.html` 于真实 390px 视口验证无横向溢出。

## 9. 顺带修复

`T0` 必须先做，否则 `db:generate` 会产出错误 diff：

1. `web/scripts/check-migrations.ts` 仍断言 `0020` 已删除的 `groups` / `group_members` / `group_invites` 三表存在。
2. `web/src/lib/db/migrations/meta/` 最新 snapshot 停在 `0021`，journal 已到 `23`。

## 10. 风险

| 风险 | 缓解 |
|---|---|
| 认证改造触及全站，回归面大 | `T2` 独立成一个阶段完成并回归，不与团队功能并行 |
| `verified` 徽章数据源随 GitHub 下线失效 | **D-3 已确认取 C：徽章整体移除**，T2 第 9 步解除挂起。范围严格限定为徽章——`social_links` 两列、`ProfileSocialLinks`、social links **cron 任务本身**全部保留（Profile 图标行仍在用，与徽章无代码耦合），因此**无需新迁移，不触碰 additive 约束**。但必须连带修改三个文件：`lib/cron/refreshSocialLinks.ts`（删 `isVerifiedBySocialLinks` import、`verified` 计数、返回类型字段）及其下游 `api/cron/refresh-social-links/route.ts`、`web/worker.ts`（解构与日志）。漏改任一处即 build 失败 |
| 头像默认值当前 fallback 到 `github.com/<user>.png` | 改为按用户名生成的首字母占位图，`T2` 内一并处理 |
| 上游迁移序号与本仓库撞车 | 已写入 `docs/upstream_policy.md` §5 的处理规则 |
