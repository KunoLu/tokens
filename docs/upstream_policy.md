# Upstream Policy

本文件说明 `KunoLu/tokens` 如何从上游取用改动、拒绝什么、以及哪些必须由人工判断后才能合入。

它存在的理由很具体：上游的一个 PR 里经常同时包含"我们想要的解析器修复"和"我们不要的 UI"，没有规则的话，评审者会倾向于合并那个更容易合并的部分。

---

## 1. Fork 链路

这是一条**三级链路**，而不是常见的两级：

```
junhoyeo/tokscale        ← 原始项目（5.3k stars，活跃）
        │  （压平导入：410 文件汇成单个 root commit，非 GitHub fork）
        ▼
missuo/tokens            ← tokens.ci 的线上仓库（157 stars）
        │  （GitHub fork）
        ▼
KunoLu/tokens            ← 本仓库
```

本地 remote 约定：

```bash
origin    git@github.com:KunoLu/tokens.git     # 本仓库
upstream  git@github.com:missuo/tokens.git     # 直接上游
# 原始项目（CLI 解析器修复的真正源头）。只读取用，故用 https，无需对该仓库有 SSH 权限：
grand     https://github.com/junhoyeo/tokscale.git
```

链路形状带来三个直接后果：

- **`missuo/tokens` 与 `junhoyeo/tokscale` 没有共同 Git 历史。** 这一条是实测得到的，不是从 GitHub 的 `isFork` 字段推断的——`isFork` 只描述 GitHub 上的 fork 关系，不代表 Git 对象图里的祖先关系，用它下结论是错的。验证方式与观测结果：

  ```bash
  git merge-base upstream/main grand/main    # exit 1、无输出 → 不存在共同祖先
  git rev-list --max-parents=0 upstream/main # 26c3e844
  git rev-list --max-parents=0 grand/main    # 73db7c1c
  ```

  两条链各有独立的 root commit。**已证实的部分**：`missuo/tokens` 的 root commit `26c3e844`（`feat: Tokens — AI coding usage leaderboard`）一次提交了 410 个文件、100,085 行插入，且 `0009_add_groups.sql` 与 `0020_drop_group_tables.sql` 这两个语义相反的迁移同时出现在这一个提交里。**属于推断的部分**：这些观测与"一次压平/快照式导入"一致，但仅凭对象图无法区分具体手段（归档导入、squash、`commit-tree` 等都能产生同样的形状），也无法单独证明该 tree 精确对应 tokscale 的某个时点。

对同步操作来说，需要确定的结论只有一条：**`missuo` 没有接入 tokscale 的提交历史**，所以两边的改动无法靠共同祖先自动对齐。
- **但没有共同历史并不妨碍 cherry-pick。** `git cherry-pick <sha>` 是拿该提交对它自己的父提交求 diff，再以三方合并应用到 HEAD，因此并不要求与 HEAD 共享历史。实测 `git cherry-pick -n 355f67a8`（tokscale 的一个 CLI 修复）确实执行了，并把补丁内容留在了工作区，失败原因是**路径在本仓库不存在**，与历史无关：

  ```
  CONFLICT (modify/delete): crates/tokscale-cli/src/commands/usage/claude.rs
    deleted in HEAD and modified in 355f67a8.
    Version 355f67a8 of crates/tokscale-cli/src/commands/usage/claude.rs left in tree.
  ```

  所以真正的障碍是**目录布局与文件内容的分歧**，对策是路径翻译（见 §4.2），而不是放弃 cherry-pick。
- CLI 的 provider / 解析器修复通常**首发于 `tokscale`**，之后才可能被 `missuo` 采纳。想及时拿到这类修复，必须直接盯 `grand`，不能只等 `upstream`。

---

## 2. 本仓库的形态

下面两节决定了后面所有规则。**必须先分清哪些已经落地、哪些只是计划**——用计划态去判断上游改动，会把本该直接合并的提交误判成需要重写。

截至本文写作时，本仓库在**代码上与 `upstream/main` 没有分歧**。`feature/teamboard-teams-auth` 分支只改动文档与规划产物，`web/`、`cli/`、`packages/` 一行未动。Teamboard、Team / Group 表、邮箱密码认证、Hall of Shame 的移除**都还没有落地**，它们是 T0–T12 的计划（见 `docs/prd-teamboard-teams-auth.md` §11）。

任何时候都可以自己确认当前处于哪个阶段：

```bash
# 比较两端完整的已提交代码树（两点语法）
git diff --stat upstream/main HEAD -- web cli packages
# 已提交之外，还要覆盖工作区与未跟踪文件
git status --short -- web cli packages
```

两条都为空输出，才说明代码零分歧。**注意必须用两点 `A B`，不能用三点 `A...B`**：`git diff` 的三点语法比较的是 `merge-base(A,B)` 与 `B`，只回答"HEAD 自分叉点以来改了什么"，完全忽略 `upstream/main` 在分叉点之后独有的提交——本地落后上游时它照样是空的，据此判断会得出"两端相同"的错误结论。pathspec 也要用 `web` 而不是 `web/src`，否则漏掉 `web/worker.ts`、`web/wrangler.jsonc`、`web/scripts/` 这些同样是运行代码的路径。

### 2.1 现状

| | tokscale（原始） | missuo/tokens（直接上游） | 本仓库现状 |
|---|---|---|---|
| CLI | 完整 TUI 面板 + 报表命令 | 无 TUI 与 `models` / `monthly` / `hourly` / `graph` / `wrapped` / `pricing` 等报表命令；保留提交、账户管理、后台提交、导入与各 provider 集成命令（顶层 `Commands` 共 15 个：`login`、`logout`、`whoami`、`status`、`import`、`submit`、`serve`、`autosubmit`、`headless`、`codex`、`cursor`、`antigravity`、`trae`、`warp`、`delete-submitted-data`） | 与直接上游一致，**且本次改造也不打算改动** |
| 报表 | 终端内 | Web 端 | 与直接上游一致 |
| 托管 | 自建 Docker + 同机 Postgres | Cloudflare Workers（OpenNext）+ Neon Postgres（经 Hyperdrive）；Worker 定点到数据库所在区域 `aws:us-west-2` | 与直接上游一致 |
| 缓存 | — | R2 渲染页 + Durable Objects 标签失效；`/api/og` 与 SVG 图片端点另有显式 edge cache | 与直接上游一致 |
| 前端 | 上游自有组件 | 重建于 shadcn/ui | 与直接上游一致 |
| SEO / 元数据 | — | 多个页面各自声明 per-page Open Graph card | 与直接上游一致 |
| 反作弊 | — | 跨设备去重、重复提交单调性校验、封禁、**公开 Hall of Shame** | 与直接上游一致：公示页仍在（`web/src/app/(main)/shame/page.tsx`、导航项 `Hall of Shame`） |
| 身份 | 用户名 | GitHub OAuth + 社交链接验证徽章 | 与直接上游一致：GitHub OAuth 仍是唯一登录方式 |
| 组织 | team / group 排行榜 | 已移除，仅一张全局榜 | 与直接上游一致：schema 只有 9 张表，无 `teams` / `groups` |
| 迁移序号 | 已到 `0029` | 停在 `0023` | 与直接上游一致，停在 `0023` |
| 测试 | 有 | 保留 Rust 内联测试并由 CI 运行（`cargo test --manifest-path cli/Cargo.toml`）；另有迁移集成检查 `web/package.json` 的 `test:migrations`；web 侧只有 lint + typecheck，无单元测试 | 与直接上游一致 |
| 仓库布局 | `crates/`、`packages/frontend/` | `cli/`、`web/` | 与直接上游一致 |
| 文档与规划产物 | 有 provider / 解析器与恢复方案等设计文档（`docs/9router-bridge.md`、`docs/providers/`、`docs/ratchet-inflation-recovery.md`、`docs/sessions-column-budget.md`）；通常只作为理解提交意图的材料，不直接合并 | 仅 `docs/upstream_policy.md`（且不含 `AGENTS.md`） | **本仓库独有**：`docs/prd-teamboard-teams-auth.md`、`docs/demo/`、`web/features/`、`.trellis/`、`AGENTS.md`；`docs/upstream_policy.md`（本文件）被整篇改写 |

**所以在 §2.2 中任何一行生效之前，代码类上游改动一律按 §3 的通用规则评估，不存在本 fork 特有的冲突区。** 现存的唯一冲突面是文档：上游若改动 `docs/upstream_policy.md`（该文件继承自上游，本仓库已整篇改写），或新增与 `docs/` / `web/features/` 同名的文件。

### 2.2 计划态分歧区（随各自任务落地后**逐行**生效；目前均未落地）

下表是改造完成后**将会**出现的高发冲突区，登记在此是为了让同步者提前知道未来的边界。

**每一行从它的"落地任务"全部完成时起独立生效，不必等整个 T0–T12 跑完。** 落地是分阶段的，开发期间会长时间处于"一部分行已生效、T9 尚未开始"的状态；此时若按整节无效来处理，就会拿 §2.1 的规则去对待已经分歧的文件。未完成的行不参与判断；T9 只负责把已生效的行迁入 §2.1 并从本节删除。判断某行是否已生效，用 §2 的两条命令核对该行"相关路径"。

| 区域 | 落地任务 | 落地后的分歧内容 | 相关路径 |
|---|---|---|---|
| 导航 | T1 / T2 / T7 | Hall of Shame 换成 Teamboard 导航项（T1）与页面（T7）；右上角移除 GitHub 图标（T2） | `web/src/components/layout/Navigation.tsx` |
| 认证 | T2 | GitHub OAuth 全部移除，改为邮箱 + PBKDF2 密码 | `web/src/lib/auth/**`、`web/src/app/api/auth/**`、`web/middleware.ts` |
| 用户模型 | T2 | `users.github_id` 放宽为可空；新增 `password_hash`、`email_verified_at` | `web/src/lib/db/schema.ts`、迁移 `0024` |
| 定时任务 | T2 | social links 刷新**保留**（Profile 社交链接图标行仍需），仅去掉返回值里的 `verified` 计数；追加过期邀请 / token 清理 | `web/wrangler.jsonc`、`web/worker.ts`、`web/src/lib/cron/**` |
| 组织体系 | T3 / T4 | 新增 5 张表与整个 Team 子域 | 迁移 `0025`、`web/src/lib/teams/**`、`web/src/app/api/teams/**` |
| 榜单 | T6 | `LeaderboardUser` 增加 `team` / `group`；查询多两组 LEFT JOIN | `web/src/lib/leaderboard/**`、`web/src/components/leaderboard/**` |
| Teamboard | T7 | 新增页面、组件、查询与 API，不改动上一行列出的路径 | `web/src/app/(main)/teamboard/**`、`web/src/components/teamboard/**`、`web/src/app/api/teamboard/**` |
| 多语言（i18n） | T10 / T11 | **全站 UI 文案被包裹进字典**，几乎所有 `tsx` 文件都会与上游文案改动冲突；上游任何文案调整需要映射到字典 key，不能直接合并文本 | `web/src/lib/i18n/**`、全部页面 `page.tsx` / 组件 |
| 徽章 | T2 | **`verified` 徽章整体移除**（决策 D-3 取 C）：两个文件删除，6 个文件的引用移除，含 Docs 页「The verified badge」章节。上游对这两个已删文件的任何改动**直接丢弃**；上游若在其他文件新增徽章渲染点，同样丢弃该片段。**社交链接本身不是分歧区**——`social_links` 两列与 `ProfileSocialLinks.tsx` 与上游一致，照常采纳 | 删除 `web/src/lib/socialVerification.ts`、`web/src/components/ui/VerifiedBadge.tsx`；改写 `web/src/lib/leaderboard/{getLeaderboard,types}.ts`、`web/src/components/leaderboard/Leaderboard.tsx`、`web/src/components/profile/ProfileView.tsx`、`web/src/app/u/[username]/ProfilePageClient.tsx`、`web/src/app/(main)/docs/page.tsx` |

**落地后，`Navigation.tsx`、`schema.ts`、`getLeaderboard.ts` 会成为冲突最集中的三个文件；T10 / T11 落地后，i18n 字典包裹会让几乎所有页面的文案都成为分歧点——上游的任何文案微调都无法直接合并，需要改写字典。** 届时上游只要改动它们，默认走"读意图、重新实现"——这不是因为 cherry-pick 不可用，而是这三个文件会被大幅改写，逐行解冲突的收益低于照意图实现。**在 T1 / T2 / T6 落地之前，这三个文件与上游逐字节一致，照常 cherry-pick。**

---

## 3. 取舍规则

| 类别 | 决策 | 例子 |
|---|---|---|
| 品牌、命名、域名、文案 | **永不合并** | 任何出现 `tokscale` / `tokens.ci` 的内容；logo；上游营销文案 |
| 新 provider、新客户端扫描器、解析器修复 | **总是合并** | 支持新 IDE / CLI；修正的 token 字段 |
| 前端**数据能力** | **合并能力，重写实现** | 新增图表维度或统计口径 → 取数据逻辑，用我们的组件重画 |
| 前端**样式、组件、布局** | **永不合并** | 上游的 styled-components、HeroUI 用法、配色、间距 |
| CLI 展示、交互、报表功能 | **跳过** | TUI 主题、更漂亮的表格、wrapped 图片 |
| 提交管线、安全、正确性 | **总是合并** | 解析溢出、重复计数、去重 |
| **认证与身份** | 现状：按 §3.2 区分"身份提供方"与"会话机制"。**T2 落地后**：身份提供方类改动默认拒绝，逐个人工评估 | 落地后上游的 OAuth 改动与本仓库的邮箱体系不兼容 |
| **team / group 相关** | 现状：无冲突，照常评估。**T3 落地后**：默认拒绝 | 落地后上游若恢复 group 排行榜，其语义与本仓库的 Team 不同，不可混用 |
| 部署、缓存、SEO 元数据 | **保留本仓库运行拓扑，逐项评估** | Worker 区域定点、图片端点的 edge cache、per-page OG card——上游改动不可直接覆盖 |
| 数据库迁移 | **逐条人工评审** | 见 §5 |
| 不确定的 | **开一个 draft PR，列出提交并询问** | — |

### 3.1 为什么前端实现从不照搬

取能力，不取代码。上游一个既加数据维度、又带自己样式的 PR，应当只读它的数据逻辑，然后在我们的组件体系里重新实现，而不是 cherry-pick。

具体约束：

- 组件只来自 `web/src/components/ui/`（vendored 的 shadcn）。不引入新的第三方组件库。
- 颜色只走语义 token（`bg-background`、`text-muted-foreground`、`border`）。不写死 hex，不手写 `dark:` 分支——两套主题都必须由 token 自然推导出来。
- 数字统一加 `.tabular`，保证列对齐。
- 不新增 styled-components 用法。

理由很窄：只要放进来一次上游样式，站点就同时背着两套设计语言。之后每次同步都会加深这个裂缝，最终没人说得清哪套才是对的。守住这条线，上游就始终只是我们数据能力的输入，而不是设计债的来源。

### 3.2 认证类改动为什么默认拒绝

**本节分两段：现状与 T2 落地之后。**

**现状（T2 未落地）**：本仓库的 GitHub OAuth 代码与上游一致，上游的 OAuth 改动照常按通用规则评估合并，没有理由默认拒绝。

**T2 落地后**：本仓库不再存在 GitHub OAuth，上游任何"改进 OAuth 流程""增加 OAuth scope""调整 GitHub 用户字段同步"的提交，在这里都无处落地。

但有一个例外必须留意：**会话与 token 的安全修复要合并**。`tt_session` 的 cookie 属性、`token_hash` 的比较方式、CSRF Origin 白名单、device flow 的过期处理——这些代码本仓库原样保留，上游在这些地方的安全修复同样适用于我们。区分标准是：*修复对象是"身份提供方"还是"会话机制"*。前者跳过，后者合并。

---

## 4. 如何同步

### 4.1 从直接上游（`missuo/tokens`）

有共同历史（`git merge-base main upstream/main` 返回 `f114057f`），cherry-pick 是常规手段。

```bash
git fetch upstream
git switch -c sync/upstream-$(date +%Y%m%d) main
git log --oneline main..upstream/main        # 先读一遍再动手
git cherry-pick -x <sha>                     # -x 记录来源提交
```

1. 从 `main` 开分支，用 `-x` cherry-pick 以记录来源。**挑之前先看父提交数**：`upstream/main` 里目前有 7 个真正的多父 merge commit（例如 `42667fd4`，两个父 `192efd63` + `497ef504`），对这类 SHA 不指定 mainline 的 `git cherry-pick` 会直接报错停下。

   ```bash
   git show --no-patch --format=%P <sha>    # 输出一个 SHA = 单父，两个 = merge commit
   ```

   单父提交沿用上面的命令即可。GitHub PR 的 merge commit，在确认第一个父确实位于 `upstream/main` 主线后用 `git cherry-pick -m 1 -x <sha>`；否则直接去挑该 PR 里的单父提交，**不要猜 mainline**。注意提交信息以 `Merge pull request` 开头并不代表它是 merge commit——上游存在被压平成单父的这类提交（如 `f114057f`），只看 message 会判断错。
2. 在 PR 描述里列出**跳过的提交及每一条的理由**。没有理由的跳过，会在下一次同步时变成一个谜。
3. 运行 `cargo check --manifest-path cli/Cargo.toml --workspace --all-targets` 与 **`cargo test --manifest-path cli/Cargo.toml --workspace`**（CI 会跑后者，本地先跑可以省一轮往返），以及 `web/` 下的 `bun run lint` 与 `bun run typecheck`。
4. 额外运行 `bun run test:migrations`。这一步目前带着两个**继承自上游、与本次改造无关的既存缺陷**：`web/scripts/check-migrations.ts` 仍在断言 `0020` 已经删除的 `groups` / `group_members` / `group_invites` 三张表，且 `meta/` 快照停在 `0021` 而 journal 已到 `23`。T0 负责修掉它们；在此之前该脚本的失败需要人工分辨真假。等 T2 / T3 带来本仓库自有的迁移后，这一步会比现在更重要。

### 4.2 从原始项目（`junhoyeo/tokscale`）

没有共同历史（§1 已实测），但**仍然可以 cherry-pick**，只是冲突率更高，且路径需要翻译。

```bash
git remote add grand https://github.com/junhoyeo/tokscale.git   # 一次性
git fetch grand --no-tags
git log --oneline grand/main -- crates/tokscale-core/            # 只看 CLI 核心
```

路径映射（tokscale → 本仓库）：

| tokscale | 本仓库 |
|---|---|
| `crates/tokscale-core` | `cli/tokens-core` |
| `crates/tokscale-cli` | `cli/tokens-cli` |
| `packages/frontend` | `web` |
| `Cargo.toml`（仓库根） | `cli/Cargo.toml` |
| `Cargo.lock`（仓库根） | `cli/Cargo.lock` |

**后两行不能靠上面的 `sed` 配方处理。** grand 的 Rust workspace 清单与 lockfile 在仓库根，本仓库的在 `cli/` 下，而路径改写只匹配目录前缀，根文件名匹配不到。真实的 CLI 修复 `85568f4c` 就同时改了 grand 的根清单、lockfile 和两个 crate 清单——直接应用会在本仓库不存在的根 `Cargo.toml` 上产生 modify/delete 冲突（非三方模式下则直接拒绝），依赖与 lockfile 的变更落不进真正的 workspace。这类提交要把根文件的改动**人工移植**到 `cli/Cargo.toml` 与 `cli/Cargo.lock`，其余部分再走下面的流程。

按成本从低到高依次尝试：

1. **直接 cherry-pick。** 路径恰好一致时可用：`git cherry-pick -x <sha>`。它以三方合并应用，出现的冲突照常解决即可。
2. **补丁 + 路径改写。** 布局不同时的主力手段：

   ```bash
   git format-patch -1 --stdout <sha> \
     | sed -e 's#crates/tokscale-core#cli/tokens-core#g' \
           -e 's#crates/tokscale-cli#cli/tokens-cli#g' \
           -e 's#packages/frontend#web#g' \
     | git apply -3
   ```

   **`-3` 与 `--reject` 不能同时使用**，Git 会直接拒绝执行：`error: options '--reject' and '--3way' cannot be used together`（退出码 128），一个补丁都不会被应用。二者选一：默认用 `-3` 走三方合并，冲突以带冲突标记的形式留在工作树和索引里，照常人工解决；确实需要 `.rej` 文件时，在干净工作树上改用不带 `-3` 的 `git apply --reject` 重跑。

   还要注意 `sed` 是**无差别改写补丁全文**的，不只是 diff 的路径头，因此不能假定它安全。斜杠模式确实避开了 Rust 里的下划线 crate 名（`tokscale_core`），但改不对的情形是真实存在的：grand 提交 `3f2e10c5` 在 TypeScript 字符串里含 `../../../../crates/tokscale-core/src/clients.rs`，而对应文件在本仓库位于更浅一层，正确的相对路径应少一个 `../`，`sed` 只会机械地产出仍带四个 `../` 的结果。注释、URL、`include_str!`、Cargo 的 `path =` 与 CI 的 path filter 同样会被一并改写。稳妥做法是**先把改写后的补丁存成临时文件逐个 hunk 审阅**，再决定是否应用；或者只对 `diff --git` / `---` / `+++` / rename 这些元数据行做路径感知的替换，正文里的命中逐条人工适配。
3. **整体取用文件。** 见 §4.3。
4. **读意图重新实现。** 仅在上述都不适用时使用，例如该修复缠绕在本仓库并不携带的模块里。

关于测试：**本仓库携带 Rust 内联测试，并且 CI 会真的运行它们**（`.github/workflows/ci.yml` 执行 `cargo test --manifest-path cli/Cargo.toml`），`cli/` 下含 `#[cfg(test)]` 的文件目前有 91 个。所以从 `grand` 取来的补丁里的测试**默认要一起落地**，不要成片剥掉——那等于主动删除适用于本仓库的回归测试。只在某个测试确实依赖本仓库不携带的模块（TUI、报表命令等）时，才单独去掉那一个。

无论走哪条路径，都在提交信息里写明来源 `tokscale` 的提交 SHA，便于追溯。

### 4.3 当补丁应用不划算时

路径经常冲突。如果我们的某个文件除该修复外与上游完全一致，最快且正确的做法是**整体取用上游修复后的文件，再剥掉我们不携带的部分**。

Grok 的 `turn_completed` 解析器修复就是这么落地的：修复前我们的生产代码与上游逐字节相同，唯一差异是该文件里一段依赖本 fork 不携带模块的测试。取来修复后的文件、去掉那一段，就精确得到了该改动，完全没有冲突需要解决。**这是针对单个文件的判断，不能推广成"本仓库不要测试"**——见 §4.2。

---

## 5. 迁移为什么逐条人工评审

一个在上游正确的迁移，在这里可能是错的，因为数据形状不同。2026-07-24 曾有一次同步相关改动在生产上造成跨设备重复计数，修复过程留下了四张备份表。**要读的是这个迁移对既有行做了什么，而不只是它对 schema 做了什么。**

迁移按约定是 additive 的。删除或重写数据的迁移，在合并前必须先有一份针对表内既有行的处理方案。

本仓库新增的额外约束：

- **序号撞车是明确风险。** 实测：本仓库与 `upstream/main` 当前都停在 `0023`（journal `idx` 上限 23），本方案拟占用 `0024`、`0025`（**尚未创建**），而 `grand/main` 的迁移已经到 `0029`。这些事实确定的是：**一旦从 `grand` 或 `upstream` 引入 `0024` 及以上的同号迁移，就会与本方案撞号**。它们并不能证明 `missuo/tokens` 接下来一定会采纳其中某一条，也不能证明它采纳时不会自行重编号——所以每次同步都要先看实际 journal，而不是无条件重编号。
- **重编号要连 snapshot 一起做，不能只手改 journal 的三个字段。** 处理办法是把上游迁移重新编号为本仓库的下一个可用序号，但必须同时产出**同号的 `meta/NNNN_snapshot.json`**。`web/scripts/check-migrations.ts` 会显式校验这几件事，写错会被直接拦下（不是"不会报错"）：`idx` 序列连续且与数组顺序一致、`tag` 的数字前缀必须等于 `idx`、`when` 严格递增、journal 引用的 `.sql` 必须存在、磁盘上不能有无 journal 条目的孤儿 `.sql`，以及**最新的 `meta/NNNN_snapshot.json` 编号必须等于 journal 尾部的 `idx`**（历史号段的空缺是允许的，只有尾部必须是最新的）。尾部 snapshot 落后的后果不止是测试失败：`drizzle-kit generate` 是基于字典序最新的 snapshot 生成的，基线过旧会让它重新发出已经应用过的 DDL（例如重复的 `ADD COLUMN`，应用时报错）。因此优先用 `drizzle-kit generate` 在本仓库当前基线上产出迁移与 snapshot，再把上游的数据语句移植进去并逐条评审，而不是手工编辑元数据。
- **（T2 / `0024` 落地后才适用）** 任何触及 `users` 表的上游迁移都要额外检查：届时本仓库已放宽 `github_id` 为可空，并新增 `password_hash`、`email_verified_at` 与 `users_email_lower_unique`。在此之前 `users` 表与上游一致。
- **（T3 / `0025` 落地后才适用）** 上游若再次引入名为 `groups` 的表，**必须改名**：届时本仓库的 `groups` 表语义完全不同（从属于 `teams`），同名会造成迁移冲突和语义混淆。在此之前本仓库没有 `groups` 表，上游恢复该表不构成冲突。

---

## 6. 本策略不覆盖的范围

上游的发布流程、CI 与打包不在跟踪范围内。我们的发布管线是我们自己的——它发布不同的平台包，来自不同的 workspace 路径，且不含 TUI 的可选特性。

---

## 7. 定期同步节奏（建议）

| 频率 | 动作 |
|---|---|
| 每两周 | `git fetch upstream && git log --oneline main..upstream/main`，分类当期提交 |
| 每月 | `git fetch grand`，只筛 `crates/` 下的 provider / 解析器修复；留意是否连带改动根 `Cargo.toml` / `Cargo.lock`（见 §4.2） |
| 每次同步 | 开 `sync/upstream-YYYYMMDD` 分支，PR 里附"合并 / 跳过"两张清单 |
| 每季度 | 用 §2 的两条命令复核 §2.1 现状表与 §2.2 各行的生效状态；已生效的行从 §2.2 移入 §2.1 |

同步 PR 不要与功能开发混在一个分支里。功能分支（如 `feature/*`）与同步分支的评审关注点完全不同——前者看行为是否正确，后者看有没有把不该带的东西带进来。
