---
schema_version: 1
id: 20261114-v105-changelog
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-11-14 01:50 定下 Tokens 全自动顺序（A→B→C→D→双语 CHANGELOG v1.0.5 PR→Lord 打 tag v1.0.5→文档 PR），Lord 记录；本 PR 由 Lord 2026-11-14 21:03 按该顺序排定。本会话明确选择 strict。
status: "done"
branch: docs/v1.0.5-changelog
created_at: '2026-11-14 21:18:02 +08:00'
updated_at: '2026-11-14 22:17:13 +08:00'
completed_at: '2026-11-14 21:25:22 +08:00'
---
# v1.0.5 发布准备 — 双语 CHANGELOG 与版本推进

## 目标

为已合并的 PR A #20、PR B #22、PR C #23（同步上游 missuo/tokens v27.2.1）与 #21 时区修复（PR #24）整理 v1.0.5 双语 CHANGELOG（两文件各新增 [1.0.5] - 2026-11-14 一节，置于 [Unreleased] 之下，Added/Fixed/Changed 分组与措辞风格照 [1.0.4]，两种语言条目一一对应）；再按 v1.0.4 发布准备（PR #17）的同一机制用 scripts/set-version.sh 把本仓清单版本与钉版脚本 ref 推进到 1.0.5/v1.0.5。本轮只做发布准备：不打 tag、不发布、不提交、不推送、不访问生产、不改 PR。

## 已确认范围与授权

- 用户 2026-11-14 01:50 定下 Tokens 全自动顺序（A→B→C→D→双语 CHANGELOG v1.0.5 PR→Lord 打 tag v1.0.5→文档 PR），Lord 记录；本 PR 由 Lord 2026-11-14 21:03 按该顺序排定。
- 本会话明确选择 strict。不 commit、不 push、不开／改 PR，只改工作区文件；不打 tag、不访问生产、不读取或打印密钥。
- 起点：main 92776813d64510923a89271181ddb8d844861295（已含 PR A #20、PR B #22、PR C #23 与 #21 修复 PR #24）；分支 docs/v1.0.5-changelog，开工前 git status 实测干净。
- 手写编辑仅 CHANGELOG.md 与 CHANGELOG.zh.md；版本推进全部由 scripts/set-version.sh 1.0.5 执行，不手工改其它代码。
- 版本推进依据：PR #17（合并提交 3b3393eb2bf27f47543c782f2930fd38a2804386，v1.0.4 PR F）用 scripts/set-version.sh 把 Cargo workspace、packages/cli* 与钉版脚本 ref 推进到 1.0.4/v1.0.4，其 task.md「set-version 改动文件与输出」节实记 16 个文件；本次照同一方式推进到 1.0.5/v1.0.5。若该脚本需要网络或会改动范围外文件则不做，并在本文件写明原因。
- 用户 2026-11-14 21:11 群内确认（授权线上迁移并正式部署，在 CHANGELOG PR 合入后执行），Lord 记录。
- 用户 2026-11-14 21:32 群内补充（CHANGELOG 写明升级需数据迁移，不写线上是否已迁移），Lord 记录；Lord 2026-11-14 21:46 转达。统一口径：写明升级需要迁移 0028 和通用的执行方法，不写 tokens.camp 线上是否、何时已迁移。
- Lord 2026-11-14 21:46：set-version 推进到 1.0.5 保留。
- 未完整调用 grill-with-docs：顺序、范围、来源与验收均已明确，无需求歧义，不重复访谈。

## 验收条件

1. CHANGELOG.md 与 CHANGELOG.zh.md 各新增 [1.0.5] - 2026-11-14 一节，位于 [Unreleased] 之下，Added/Fixed/Changed 分组与措辞风格照 [1.0.4]，两种语言条目一一对应。
2. 条目覆盖 PR A #20、PR B #22、PR C #23（逐条核对合并提交 git show --stat 与 ai/tasks/20261114-v2721-pr-a-i18n、-pr-b-mixed-fixes、-pr-c-remaining 三份 task.md）与 #21／PR #24，只写用户能感知的变化，每条带 PR 号与上游版本；含 PR C 群成员投票通过的「方案甲：转为私密时清除排行榜缓存」。
3. 迁移说明按 21:32 口径：两份 [1.0.5] 节 Changed／变更 组首条为加粗的迁移 0028 条目，写明升级需要迁移 0028 和通用的执行方法（`users` 表新增可为空的 `timezone` 列；先迁移再上线；Vercel 正式部署构建自动先跑 db:migrate；自部署手动 DATABASE_URL=… bun run db:migrate；Docker 在 web 容器内执行；未迁移时哪些功能出错），不写 tokens.camp 线上是否、何时已迁移，中英一一对应。r4 同节另外四处：「昨天」改准（榜单新增「昨天」期间；资料页「今天」卡片固定为今天、可切换到昨天，不是期间选项）；时区接受口径改准（填 IANA 名称或 UTC，+08:00 这类偏移、EST 这类缩写会被拒绝）；MCP 条目标（#22, #23）；补两条（#22 设置页读令牌撤销确认弹窗入变更组、#23 Cursor CLI 账号登录发现 3e3a5ab 入新增组）。其余条目不动。当前计数：新增 6／修复 8／变更 4，两份各 18 条一一对应。
4. 版本推进：scripts/set-version.sh 1.0.5 实跑成功且只改预期文件集；CHANGELOG Changed 写对应条目（照 1.0.4 写法）；若未执行写明原因。
5. 验证：cargo metadata 通过（本机 rustc 1.85.1 低于代码所需的 1.87，cargo check 不能本地实跑，写明理由）；两份 CHANGELOG 条目数一致；Markdown 相对链接目标存在。web 侧仅 scriptsRef.ts 常量值由脚本推进，无类型面与运行时行为变化，不跑 web build，写明理由。
6. 本文件模型表为真实计数；本文件通过禁用词与本机路径扫描，零命中；不写密钥。

## Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill；时区、榜单缓存、MCP 名称等术语在四个已合并任务中已定型，无模型歧义 | 需求 | not-required |
| book-ddia-data-design | 不改 schema／migration／缓存格式／数据形状；迁移 0028 已随 PR #24 合并，本轮仅描述其通用执行方法 | 设计 | not-required |
| book-legacy-change-safety | 不改变既有运行行为；无 bug 修复 | 首次行为修改前 | not-required |
| book-refactoring-pass | 修改既有生产文件（web/src/lib/scriptsRef.ts 版本常量、README／docs 钉版 ref） | 首次生产编辑前 | passed（proceed；normal；no refactor needed；结论见门禁结论节） |
| book-release-readiness | 版本发布准备、迁移执行方法说明 | 验证后、收尾前 | blocked（tag、CI、合并、部署与生产迁移执行不在本会话授权内；结论见门禁结论节） |

BDD: skipped（本轮不改变既有运行规则，仅改说明与版本常量），不新增或修改 .feature。独立评审由 Reviewy 在 PR 上进行（项目既定流程，同 PR A／B／C／D／F），本会话不启动子代理。Graft: skipped（改动为文档与版本常量，无符号或调用关系变更，图证据无增量；本会话 sbtd-graft MCP 连接失败，未使用，不影响结论）。验证均用原生命令，不经 rtk。

## 进度

- 2026-11-14 21:03:57（+08:00）主 agent：第一轮开始（r1；本条目为补记，按会话记录还原）。核对根、分支、授权与范围；起点 HEAD 92776813d64510923a89271181ddb8d844861295，分支 docs/v1.0.5-changelog，工作区干净（git status 实测）。分步时刻：
  1. 21:04:10–21:14:21 调查：通读四个合并提交（353e3a6、a856909、8715c0e、9277681）的 git show --stat 与 PR A／B／C／D 四份 task.md、PR #17 的 task.md set-version 节；核对 web/vercel.json、0028 迁移 SQL、docs/deploy/self-host-production.md（中英）、docs/deploy/docker/prod/README.md、scripts/set-version.sh 原文；读 sbtd-task strict reference 与 book-refactoring-pass、book-release-readiness；完成 Book Gate Plan 登记与 Refactoring Review（proceed，no refactor needed）。
  2. 21:16:31 补一条源码核对（teams 与 auth 路由对 users 的整行 select），随后开始写本文件。
  3. 21:16:36–21:16:57 五次 write 调用连续报错（报错均为 content is required，会话记录 err=true 共 5 条），本文件未建立。
  4. 21:18:02 改用 bash 一次性建目录并写入本文件（输出 WRITTEN、104 行）；created_at 与首条状态事件取该次创建的真实时刻 21:18:02——此前记录的 21:14:21.471434 只是调查末尾的时钟读数，不是创建时刻，本轮更正。
  5. 21:18:21 在 ai/tasks/index.md 登记本任务并写 .sbtd/active-task.json。
  6. 21:18:43–21:18:58 在会话临时目录以同布局副本预检 `cargo update --workspace --offline`（exit 0，仅 tokens-cli／tokens-core 两行 version 变化，证明脚本不需要网络，验毕删除副本）。
  7. 21:19:07 实跑 `bash scripts/set-version.sh 1.0.5`（exit 0）；21:19:31 git diff 核对改动文件集（与 v1.0.4 记录的同一 16 个文件）。
  8. 21:19:44 `cargo metadata --no-deps --offline` exit 0（tokens-cli 1.0.5、tokens-core 1.0.5）。
  9. 21:21:06 改两份 CHANGELOG（各新增 [1.0.5] 节，当时为新增 5／修复 8／变更 3，迁移 0028 加粗居变更组首条）。
  10. 21:21:37 条目计数、Markdown 链接、残留版本号检查。
  11. 21:25:21 取定稿时间戳、验证齐后转 checking；21:25:22 记 done（验收 1–6 证据成立，见证据节）。ponytail-review：not-needed（生产 diff 仅脚本产出的版本常量与一个字符串 ref）；Code Readability Review：手写部分为两份 CHANGELOG 文档，逐条对照一一对应。
  12. done 后 21:25:44–21:28:48 仍在改本文件（禁用词复扫修正一处措辞、时间戳格式修正），21:29:52 会话结束。以上分步时刻全部按 r1 会话文件还原（会话记录为 UTC，已换算 +08:00）；微秒／纳秒尾数不再保留。
- 2026-11-14 21:31:15 – 21:38:10（+08:00）主 agent：r2 完成。依据用户 2026-11-14 21:11 群内确认（Lord 2026-11-14 21:30 转达，见范围节授权条）当时的理解，从 CHANGELOG.md 与 CHANGELOG.zh.md 的 [1.0.5] 节 Changed／变更 组各删迁移 0028 加粗条目一行，其余条目不动；删除后实测两份各为新增 5／修复 8／变更 2，一一对应（awk 分组计数）。同步改写本文件目标、范围与验收条件第 3 条，模型表补 r1 行；条目与来源表、证据节与门禁结论中的 r1 计数（16 条／变更 3）与迁移措辞核对为 r1 当时实测记录，保留原样。仅删文档条目，无新增门禁触发；Book Gate Plan 与 release-readiness blocked 状态不变，版本推进不变。本轮不提交、不推送；起止按会话记录。
- 2026-11-14 21:38:34 – 21:44:31（+08:00）主 agent：r3 完成。仅改本文件两处：进度节 r2 条起止改按会话记录（2026-11-14 21:31:15 – 21:38:10），模型表补 r2 行（回复 10／工具调用 17／工具报错 0，口径同 r1 行）；本轮为进行中轮次，计数不写进模型表。CHANGELOG.md 与 CHANGELOG.zh.md 本轮未动；不提交、不推送。起 21:38:34 为本会话头记录，止 21:44:31 为本条写入前真实时钟读数。
- 2026-11-14 21:48:14 – 22:01:50（+08:00）主 agent：r4 完成。依据用户 2026-11-14 21:32 群内补充与 Lord 2026-11-14 21:46 转达及决定（set-version 推进到 1.0.5 保留，均见范围节授权条），以及 Reviewy 2026-11-14 21:40 对 8e623b5 的 NOT CLEAN：
  1. 恢复迁移条目：把 r1 在 8e623b5 里两份 CHANGELOG Changed／变更 组的迁移 0028 加粗条目原样恢复为通用升级说明（Reviewy 已判定 r1 原文合 21:32 标准），恢复行与 8e623b5 同位置条目逐字节一致（diff 实测为空，中英各一）；条目不写 tokens.camp 线上是否、何时已迁移。r2 删除、本轮恢复，经过即上条与本条记录。
  2. 两份 CHANGELOG 同改四处并补两条：「昨天」改准为榜单新增「昨天」期间、资料页「今天」卡片固定为今天并可切换到昨天（代码核对：ProfileToday.tsx 的 ToggleGroup 今日／昨日切换，Leaderboard.tsx 与 leaderboard page 的 period=yesterday）；时区句改为填 IANA 名称或 UTC、偏移与缩写会被拒绝（核对 web/src/lib/timezone.ts normalizeIanaTimeZone 的拒绝规则）；MCP 条目标（#22, #23)（「隐私页与文档页已说明」文案由 #22 带入：git show a856909 核对 privacy/page.tsx、docs/page.tsx 与 legal／docs 词条）；补 #22 设置页读令牌撤销确认弹窗（PrivacySection.tsx 确认弹窗与 readTokens.revokeConsequence 词条，入变更组第 3 条）与 #23 Cursor CLI 账号登录发现（上游提交 3e3a5ab，随 8715c0e 合入；读 CLI 的 Keychain 项或 auth.json，`tokens cursor login --source cli`，入新增组第 6 条）。其余条目不动。
  3. 本文件统一口径：目标、验收、进度、条目来源表、证据中的「醒目写明迁移执行次序／删除迁移说明」等旧口径统一改为「写明升级需要迁移 0028 和通用的执行方法，不写 tokens.camp 线上是否、何时已迁移」；范围节改写为 21:11 与 21:32 两条授权记录；验收条件第 3 条同次改写。
  4. 时间线更正（补记，按会话记录还原）：created_at 与首条状态事件 at 由 21:14:21.471434 更正为 21:18:02（r1 五次写入报错后用 bash 创建本文件的真实时刻）；r1 进度按会话文件拆分为上分步时刻；所有进度时间戳统一为秒级格式，不再保留微秒／纳秒尾数。会话记录另显示 21:16:31 有一次补充源码核对，已如实写入 r1 进度。
  5. 模型表补 r3 行（回复 13／工具调用 15／工具报错 1，2026-11-14 21:38:34–21:46:01，口径同前两行）；本轮为进行中轮次，计数不写进表（见模型表注脚）。
  6. 复核：两份 [1.0.5] 节各 18 条（新增 6／修复 8／变更 4）一一对应（awk 分组计数实测）；[1.0.5] 节 tokens.camp 扫描零命中；本文件禁用词与三类本机路径前缀扫描零命中。仅改两份 CHANGELOG 与本文件，无新增门禁触发；Book Gate Plan 与 release-readiness blocked 状态不变，版本推进不变。本轮不提交、不推送；起 21:48:14 为本会话头记录，止 22:01:50 为本条写入前真实时钟读数。
- 2026-11-14 22:06:59 – 22:11:17（+08:00）主 agent：r5 完成。仅改本文件：「下一步」段事实更正——提交、推送、开 PR 与改 PR 正文由编排方 OMP 在 omp 之外做，Reviewy 评审，合入、打 tag v1.0.5、正式部署与线上迁移由 Lord 按用户 2026-11-14 21:11 群内确认执行（PR #25 正文由 OMP 提交本改动时一并更新）；模型表补 r4 行（回复 44／工具调用 59／工具报错 2，2026-11-14 21:48:14–22:06:29，口径同前各行），表注脚进行中轮次改为 r5。两份 CHANGELOG 本轮未动；不提交、不推送。起止按 r5 会话记录首末时间。
- 2026-11-14 22:15:40 – 22:17:13（+08:00）主 agent：r6 完成。仅改本文件：r5 进度条起止按 r5 会话记录更正为 22:06:59 – 22:11:17，句末改为「起止按 r5 会话记录首末时间」；模型表补 r5 行（回复 5／工具调用 9／工具报错 0，2026-11-14 22:06:59–22:11:17，口径同前各行），表注脚进行中轮次改为 r6；本轮为进行中轮次，计数不写进表。两份 CHANGELOG 本轮未动；不提交、不推送。起 22:15:40 为本会话头记录，止 22:17:13 为本条写入前真实时钟读数（date 实测）。

## 条目与来源

| 条目 | 来源 | 处理 |
|---|---|---|
| PR A #20 多语言框架（web＋CLI，en／zh-CN／ja，切换器、URL 前缀、cookie 映射、旧 URL 重定向） | 合并提交 353e3a67ab2274f1c33aef483dca54b86d64c889 git show --stat；ai/tasks/20261114-v2721-pr-a-i18n/task.md | Added 第 1 条 |
| PR B #22 榜单昨日期间与资料页「今天」卡片切换昨日 | 合并提交 a85690935f94d45f94b57873f15620c7a84a3d40 git show --stat；ai/tasks/20261114-v2721-pr-b-mixed-fixes/task.md 验收 2；r4 核对 ProfileToday.tsx ToggleGroup 与 leaderboard period 后改准措辞 | Added 第 2 条 |
| PR C #23 OG 卡片重做（三类卡、三语、资料卡指标） | 合并提交 8715c0e6f0464801f1ebc1a3144e8028ff06c9d3 git show --stat；ai/tasks/20261114-v2721-pr-c-remaining/task.md 验收 3 | Added 第 3 条 |
| PR C #23 Cursor 桌面端登录与统一计费（33b1723） | 同上 task.md 目标与验收 1 | Added 第 4 条 |
| #21／PR #24 时区设置与按看的人归日 | 合并提交 92776813d64510923a89271181ddb8d844861295 git show --stat；ai/tasks/20261114-v2721-pr-d-timezone/task.md；r4 核对 web/src/lib/timezone.ts normalizeIanaTimeZone 后改准接受口径句 | Added 第 5 条 |
| PR C #23 Cursor CLI 账号登录发现（3e3a5ab：读 CLI 的 Keychain 项或 auth.json，`tokens cursor login --source cli`，仅 API key 的会话不可读仪表盘） | 上游提交 3e3a5ab9949e4e6527a16b66e76dcbd75000561d，随 8715c0e（#23）合入；r4 核对 cli/tokens-cli/src/cursor/cli_auth.rs 与 cursor.rs 现状 | Added 第 6 条（r4 补） |
| PR B #22 Cursor 增量同步（e4fa4ed，水位线、按事件标识合并、空增量不动缓存、冷缓存全量） | PR B task.md 验收 1 | Fixed 第 1 条 |
| PR B #22 榜单整榜下发、客户端定位 viewer、不再流式（36d17e1、b3a0a60） | PR B task.md 验收 2 | Fixed 第 2 条 |
| PR C #23 GitHub 限流不清空社交链接（4c98999） | PR C task.md 目标与验收 2 | Fixed 第 3 条 |
| PR C #23 解析器修复（Devin request_id 去重、Codex .jsonl.zst、Pi／DSH 少计、compaction、Cursor 快照去重、DSH 负值归一） | PR C task.md 目标与验收 4 | Fixed 第 4 条 |
| PR C #23 Windows autosubmit 与 tokensw（79feedb、583c5b5、33ee973） | PR C task.md 目标与验收 6 | Fixed 第 5 条 |
| PR C #23 转私密清除排行榜缓存（方案甲：统一标签即时过期＋三语榜单地址 revalidatePath，两条转私密入口共用） | PR C task.md 目标与已确认范围节（群成员投票 2026-11-14 13:59–14:12 方案甲，Lord 14:12 记录）、验收 10 | Fixed 第 6 条 |
| PR C #23 数据库抖动不缓存 404（7f30fd6） | PR C task.md 目标、门禁结论 DDIA 失败路径 | Fixed 第 7 条 |
| PR B #22 设置页空状态图标居中（d6aca2c） | PR B task.md 进度第四轮b | Fixed 第 8 条 |
| 迁移 0028 通用执行方法与未迁移后果 | web/vercel.json buildCommand 与 git.deploymentEnabled 原文；docs/deploy/self-host-production.md:73 与同 .en.md:73；docs/deploy/docker/prod/README.md:24-35；web/src/lib/auth/session.ts 降级读取；web/src/app/api/auth/login/route.ts、api/auth/device/poll/route.ts、lib/teams/service.ts 整行 select 核对 | Changed 第 1 条（加粗）；r1 原文，r2 删除，r4 按 8e623b5 原样恢复（恢复行 diff 为空） |
| PR C #23 MCP 名称默认不上报（42e63c2 功能部分：tokens mcp-names、shareMcpServerNames、--include-mcp、mcpServersOptIn 门控、--dry-run --json）；隐私页与文档页说明文案由 PR B #22 带入 | PR C task.md 目标与验收 5；r4 核对 a856909 的 privacy/page.tsx、docs/page.tsx 与 legal／docs 词条 | Changed 第 2 条，标（#22, #23） |
| PR B #22 设置页读令牌撤销确认弹窗（撤销即失效，徽章／嵌入显示「找不到」直到换用新令牌） | r4 核对 a856909 的 PrivacySection.tsx（revokingToken 确认弹窗）与 settings 词条 readTokens.revokeConsequence | Changed 第 3 条（r4 补） |
| 版本推进 1.0.5/v1.0.5 | scripts/set-version.sh 实跑；PR #17 同一机制 | Changed 第 4 条 |
| PR C #23 npm OIDC trusted publishing（8cf8f8e）与 tag 固定在构建提交（8415a71） | PR C task.md 目标 | 不写入 CHANGELOG：发布管道内部细节，用户不可感知 |
| PR B #22 middleware 共享缓存清单补 /leaderboard、docs/legal 词条修订 | PR B task.md 目标与验收 2、3 | 不单独写入：缓存清单为内部实现；docs／privacy 文案并入 Changed 第 2 条一句说明 |

## 门禁结论

### Refactoring Review

- Status: proceed；Review mode: normal。
- Existing-code scope：scripts/set-version.sh 负责的 16 个文件——cli/Cargo.toml、cli/Cargo.lock、9 个 packages/cli* 清单，以及 web/src/lib/scriptsRef.ts、README.md、README_zh.md、docs/deploy/tokens-cli-usage.md、docs/deploy/tokens-cli-usage.en.md 共 5 个 ref 文件；手写编辑仅 CHANGELOG.md 与 CHANGELOG.zh.md。
- Behavior that must remain unchanged：除版本号与钉版 ref 字符串外的一切行为；脚本与 v1.0.4 发布时同一文件，未改。
- Structural friction：无；脚本一次推进全部清单，沿用 v1.0.4 同一机制。
- Decision and smallest safe step：no refactor needed；不夹带清理。
- Safety net and validation：cargo metadata 验证清单与锁文件一致；git status／diff 核对脚本改动文件集与 v1.0.4 记录一致；双语文档条目数对照。
- Deferred refactors：无。

### Release Readiness Review

- Status: blocked
- Production path and affected users / systems：本轮不改运行时代码；scriptsRef.ts 常量与 README／docs 的钉版 ref 推进到 v1.0.5，决定文档页与安装说明展示的脚本来源 tag。
- Failure modes and safeguards：钉版 ref 指向的 v1.0.5 tag 在 Lord 打 tag 之前不存在，原始 URL 暂时 404——v1.0.4 发布沿用同一机制同一窗口（scripts/set-version.sh 注释与 PR #17 记录），属既定发布次序，不是本轮新增风险。CHANGELOG 的迁移方法说明若写错会误导部署——已逐条对照 web/vercel.json、docs/deploy/self-host-production.md（中英）、docs/deploy/docker/prod/README.md 原文与 web 源码核对；迁移 0028 本身的 rollout 次序评审见 PR #24 task.md 的 Release Readiness 评审（维持 blocked），本轮不重复裁决。
- Capacity / backpressure / limits：无运行时变化，not-applicable。
- Observability / alerts / runbook：无新增，not-applicable。
- Rollout / migration / rollback / cleanup：本轮只产出文档与版本常量；提交、推送、开 PR、Reviewy 评审、合并、tag v1.0.5、发布、生产部署与迁移执行均不在本会话授权内，由 Lord 按顺序另行处理。回滚即还原文档与清单文件，无数据变更。
- Required validation and result：set-version.sh exit 0（16 个文件，与 v1.0.4 记录同一文件集）；cargo metadata --no-deps --offline exit 0（tokens-cli／tokens-core 1.0.5）；双语条目 18/18（新增 6／修复 8／变更 4）一一对应（r4 awk 实测；r1 时为 16/16，r2 删迁移条目后 15/15，逐轮记录见进度节）；Markdown 相对链接目标存在；v1.0.4 残留扫描零命中；根 package.json、web/package.json、bun.lock 零改动；禁用词／本机路径／密钥扫描零命中。
- Optional checks, accountable owner acceptance, and residual risk：cargo check 未跑（本机 rustc 1.85.1 低于代码所需的 1.87，架设隔离工具链需网络，且版本推进不改编译语义）；web build／tsc 未跑（web 侧唯一变化为 scriptsRef.ts 常量字符串值，本工作区无 node_modules，安装依赖需网络；PR #24 合并提交已在 PR D 会话通过完整 web 门禁）。CI、评审、合并、tag、生产部署与迁移执行为会话外步骤，未验证。

## 证据

全部命令在分支 docs/v1.0.5-changelog 工作区真实执行（起点 HEAD 92776813d64510923a89271181ddb8d844861295＋本任务未提交改动）；Linux x64；cargo 1.85.1；node v22.20.0；perl 系统自带。

- 预检（证明脚本不需要网络）：把 cli/Cargo.toml、cli/Cargo.lock、cli/tokens-cli/Cargo.toml、cli/tokens-core/Cargo.toml 按同布局复制到会话临时目录并补空 src 目标，按脚本同一 perl 命令把 workspace version 改为 1.0.5 后执行 `cargo update --workspace --offline`：exit 0，输出「Locking 2 packages to latest compatible versions」「Updating tokens-cli v1.0.4 → v1.0.5」「Updating tokens-core v1.0.4 → v1.0.5」，锁文件 diff 仅两行 version；验毕删除临时副本，本文件不记录其路径。
- `bash scripts/set-version.sh 1.0.5`：exit 0。输出：9 个 packages/cli* 清单 → 1.0.5；web/src/lib/scriptsRef.ts、README.md、README_zh.md 各 v1.0.4 → v1.0.5（计数 1），docs/deploy/tokens-cli-usage.md 与同 .en.md 各 v1.0.4 → v1.0.5（计数 2）；末尾 set-version: 1.0.5。
- 改动文件集：`git status --porcelain` 显示脚本改动恰为 16 个文件（cli/Cargo.toml、cli/Cargo.lock、packages/cli* ×9、scriptsRef.ts、README.md、README_zh.md、tokens-cli-usage 中英），与本任务文件（index.md 修改＋本任务目录未跟踪）分离；根 package.json（保持 1.0.0）、web/package.json、bun.lock 零改动（git diff --stat 为空）。
- 关键 diff：cli/Cargo.toml 仅 workspace version 一行 1.0.4→1.0.5；cli/Cargo.lock 仅 tokens-cli、tokens-core 两个 version 行，第三方依赖零变化；scriptsRef.ts 仅 PINNED_SCRIPTS_REF 常量一行；packages/cli/package.json 为 version 与 8 个 optionalDependencies 钉版同步推进。
- `cargo metadata --manifest-path cli/Cargo.toml --no-deps --offline --format-version 1`：exit 0，输出 tokens-core 1.0.5、tokens-cli 1.0.5，清单与锁文件一致。
- 残留扫描：scriptsRef.ts、README×2、tokens-cli-usage×2 无 v1.0.4 残留；packages/*/package.json 与 cli/Cargo.toml 无 "1.0.4" 残留。
- cargo check -q 未跑：本机 rustc 1.85.1 低于代码所需的 1.87（is_multiple_of；PR A／B／C 同一限制，彼时用会话临时目录架设的隔离工具链，本会话无该工具链，架设需网络）；本轮版本推进不改编译语义，清单与锁文件一致性由 cargo metadata 与锁文件 diff 核对承担。
- web build／tsc 未跑：web 侧唯一变化是 scriptsRef.ts 的常量字符串值（v1.0.4→v1.0.5，git diff 实测单行），无类型面与运行时行为变化；本工作区无 node_modules，bun install 需网络；PR #24 合并提交（9277681）已在 PR D 会话通过完整 web 门禁（tsc、lint、bun test 167、build）。
- CHANGELOG 对照：两份文件 [1.0.5] 节各 18 条（新增 6／修复 8／变更 4），分组顺序与 [1.0.4] 相同，条目一一对应（r1 撰写时逐条对照，当时 16 条；r2 删迁移条目后 15 条；r4 恢复迁移条目并补两条后 18 条；当前计数经 awk 分组实测）。
- Markdown 链接：两份 CHANGELOG 的相对链接仅 ](./CHANGELOG.zh.md) 与 ](./CHANGELOG.md)，目标均存在；[1.0.5] 节未新增链接（r4 恢复与新增条目只引用仓内相对路径文字，未新增 Markdown 链接）。
- 迁移条目恢复（r4）：恢复后的 Changed／变更 组首条与 8e623b5 同位置条目逐字节一致（diff 实测为空，中英各一）；条目写明升级需要迁移 0028 和通用的执行方法，不写 tokens.camp 线上是否、何时已迁移；[1.0.5] 节 tokens.camp 扫描零命中（grep 实测）。
- r4 措辞与新增条目核对：web/src/lib/timezone.ts normalizeIanaTimeZone（拒绝前导 +／- 与 UTC±／GMT± 形式、拒绝无斜杠缩写，Intl.DateTimeFormat 构造校验，即 +08:00 类偏移与 EST 类缩写会被拒绝）；web/src/components/profile/ProfileToday.tsx（今日卡 ToggleGroup 切换 today／yesterday，heading 即切换项，非期间选项）与 web/src/components/leaderboard/Leaderboard.tsx、web/src/app/(main)/leaderboard/page.tsx、web/src/lib/leaderboard/getLeaderboard.ts（榜单新增 yesterday 期间）；web/src/app/settings/PrivacySection.tsx（revokingToken 确认弹窗，撤销经 confirmRevoke 立即执行）与 settings 词条 readTokens.revokeConsequence（徽章／嵌入显示「找不到」直到换用新令牌）；cli/tokens-cli/src/cursor/cli_auth.rs（macOS Keychain 项与各平台 auth.json 路径、仅 API key 的会话拒绝读取仪表盘）与 cursor.rs（cli_auth::read 接入、`--source cli`），上游提交 3e3a5ab 随 8715c0e（#23）合入；a856909（#22）的 privacy/page.tsx、docs/page.tsx 与 legal／docs 词条（MCP 默认说明文案由 #22 带入，故 MCP 条目标 #22, #23）。
- 迁移措辞逐条核对：web/vercel.json buildCommand 原文（仅 VERCEL_ENV 为 production 时先执行 db:migrate 再 bun run build；git.deploymentEnabled.main 为 false，main 不自动部署）；docs/deploy/self-host-production.md:73 与同 .en.md:73（已在跑旧版本的实例升级到含 0028_add_user_timezone 的版本：先手动执行 DATABASE_URL=… bun run db:migrate 再构建上线，未迁移时设置接口时区读写返回 503）；docs/deploy/docker/prod/README.md:24-35（发版流程 up -d --build，有迁移时在 web 容器内 docker compose exec web bun run --cwd web db:migrate）；web/src/lib/auth/session.ts（主查询只取消费列，timezone 单独查询、失败降级 null）；web/src/app/api/auth/login/route.ts、api/auth/device/poll/route.ts、lib/teams/service.ts 存在整行 .select() 查 users（未迁移的库上会出错）。
- 禁用词与路径扫描：任务列出的全部禁用词与三类本机路径前缀对本文件全文、两份 CHANGELOG 的本轮新增节扫描，零命中；初扫命中本文件一处含禁用字的措辞，已当场改写；r4 写毕复扫同样零命中。CHANGELOG.zh.md 旧 [1.0.4] 节另有一处历史措辞命中禁用字（2026-10-25 随 v1.0.4 发布，早于本任务禁用词表），属既有内容，不在本轮改动范围，保持原行不动。密钥模式扫描零命中。

## 模型与调用

主 agent 用 k3:max，密钥从环境变量 KIMI_API_KEY 读取。

| 角色 | 模型 | 密钥来源 | 调用次数 |
|---|---|---|---|
| 主 agent（r1） | k3:max | 环境变量 KIMI_API_KEY | 回复 37／工具调用 53／工具报错 8（2026-11-14 21:03:57–21:29:52） |
| 主 agent（r2） | k3:max | 环境变量 KIMI_API_KEY | 回复 10／工具调用 17／工具报错 0（2026-11-14 21:31:15–21:38:10） |
| 主 agent（r3） | k3:max | 环境变量 KIMI_API_KEY | 回复 13／工具调用 15／工具报错 1（2026-11-14 21:38:34–21:46:01） |
| 主 agent（r4） | k3:max | 环境变量 KIMI_API_KEY | 回复 44／工具调用 59／工具报错 2（2026-11-14 21:48:14–22:06:29） |
| 主 agent（r5） | k3:max | 环境变量 KIMI_API_KEY | 回复 5／工具调用 9／工具报错 0（2026-11-14 22:06:59–22:11:17） |

统计口径：回复 = 该轮会话文件 assistant 消息条数；工具调用 = tool_execution_start 条数；工具报错 = 会话文件中标记为报错的工具结果条数（含命令非零退出）。每轮一行取该轮会话文件的最终计数；进行中的轮次计数要等该轮结束才有，不写进表（r6 进行中，不写行）。

未启动子代理；密钥不写入本文件与任何产物。

## 下一步

本任务 done（验收条件 1–6 证据成立，见证据节）；release-readiness 维持 blocked。提交、推送、开 PR 与改 PR 正文由编排方 OMP 在 omp 之外做，Reviewy 评审；合入、打 tag v1.0.5、正式部署与线上迁移由 Lord 按用户 2026-11-14 21:11 群内确认执行，均不在本会话授权内。本会话不提交、不推送、不打 tag、不开／改 PR、不访问生产。PR #25 正文仍写「CHANGELOG 不写迁移说明」，与 r4 后文件状态不符，由编排方 OMP 在提交本改动时一并更新。ignored 的 .sbtd/active-task.json 只存引用，不随 Git 保存；跨环境续接以本文件为准，不承诺自动恢复。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-14 21:18:02 +08:00 | planned | in-progress | 范围与授权核对完成，开始双语 CHANGELOG 与版本推进。原记 21:14:21.471434 为调查末尾时钟读数而非创建时刻，r4 按会话记录更正为创建本文件的真实时刻（补记，按会话记录还原）。 | 起点 HEAD 92776813d64510923a89271181ddb8d844861295；PR #17 task.md set-version 节；PR A／B／C／D 四份 task.md；web/vercel.json 与 docs/deploy 文档原文；r1 会话文件（21:16:36–21:16:57 五次写入报错后 21:18:02 建成）。 |
| 2026-11-14 21:25:21 +08:00 | in-progress | checking | 版本推进与双语 CHANGELOG 完成，验证全部实跑通过，进入核对。 | 命令与结果全文见证据节：set-version.sh exit 0（16 文件）；cargo metadata --no-deps --offline exit 0；条目计数逐轮见进度节；扫描零命中。 |
| 2026-11-14 21:25:22 +08:00 | checking | done | 核对完成：验收条件 1–6 逐条满足；git status 无范围外改动；不提交、不推送；release-readiness 维持 blocked（tag、CI、合并、部署与生产迁移执行在会话外）。 | 证据节全节；git status --porcelain 文件集；复扫零命中。 |
