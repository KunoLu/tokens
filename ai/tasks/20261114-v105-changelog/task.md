---
schema_version: 1
id: 20261114-v105-changelog
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-11-14 01:50 定下 Tokens 全自动顺序（A→B→C→D→双语 CHANGELOG v1.0.5 PR→Lord 打 tag v1.0.5→文档 PR），Lord 记录；本 PR 由 Lord 2026-11-14 21:03 按该顺序排定。本会话明确选择 strict。
status: "done"
branch: docs/v1.0.5-changelog
created_at: '2026-11-14T21:14:21.471434+08:00'
updated_at: '2026-11-14T21:25:22.278717000+08:00'
completed_at: '2026-11-14T21:25:22.278717000+08:00'
---
# v1.0.5 发布准备 — 双语 CHANGELOG 与版本推进

## 目标

为已合并的 PR A #20、PR B #22、PR C #23（同步上游 missuo/tokens v27.2.1）与 #21 时区修复（PR #24）整理 v1.0.5 双语 CHANGELOG（两文件各新增 [1.0.5] - 2026-11-14 一节，置于 [Unreleased] 之下，Added/Fixed/Changed 分组与措辞风格照 [1.0.4]，两种语言条目一一对应），并醒目写明迁移 0028 的执行次序；再按 v1.0.4 发布准备（PR #17）的同一机制用 scripts/set-version.sh 把本仓清单版本与钉版脚本 ref 推进到 1.0.5/v1.0.5。本轮只做发布准备：不打 tag、不发布、不提交、不推送、不访问生产、不改 PR。

## 已确认范围与授权

- 用户 2026-11-14 01:50 定下 Tokens 全自动顺序（A→B→C→D→双语 CHANGELOG v1.0.5 PR→Lord 打 tag v1.0.5→文档 PR），Lord 记录；本 PR 由 Lord 2026-11-14 21:03 按该顺序排定。
- 本会话明确选择 strict。不 commit、不 push、不开／改 PR，只改工作区文件；不打 tag、不访问生产、不读取或打印密钥。
- 起点：main 92776813d64510923a89271181ddb8d844861295（已含 PR A #20、PR B #22、PR C #23 与 #21 修复 PR #24）；分支 docs/v1.0.5-changelog，开工前 git status 实测干净。
- 手写编辑仅 CHANGELOG.md 与 CHANGELOG.zh.md；版本推进全部由 scripts/set-version.sh 1.0.5 执行，不手工改其它代码。
- 版本推进依据：PR #17（合并提交 3b3393eb2bf27f47543c782f2930fd38a2804386，v1.0.4 PR F）用 scripts/set-version.sh 把 Cargo workspace、packages/cli* 与钉版脚本 ref 推进到 1.0.4/v1.0.4，其 task.md「set-version 改动文件与输出」节实记 16 个文件；本次照同一方式推进到 1.0.5/v1.0.5。若该脚本需要网络或会改动范围外文件则不做，并在本文件写明原因。
- 迁移 0028 的措辞以 web/vercel.json 与 docs/deploy 文档原文为准，逐条核对。
- 未完整调用 grill-with-docs：顺序、范围、来源与验收均已明确，无需求歧义，不重复访谈。

## 验收条件

1. CHANGELOG.md 与 CHANGELOG.zh.md 各新增 [1.0.5] - 2026-11-14 一节，位于 [Unreleased] 之下，Added/Fixed/Changed 分组与措辞风格照 [1.0.4]，两种语言条目一一对应。
2. 条目覆盖 PR A #20、PR B #22、PR C #23（逐条核对合并提交 git show --stat 与 ai/tasks/20261114-v2721-pr-a-i18n、-pr-b-mixed-fixes、-pr-c-remaining 三份 task.md）与 #21／PR #24，只写用户能感知的变化，每条带 PR 号与上游版本；含 PR C 群成员投票通过的「方案甲：转为私密时清除排行榜缓存」。
3. 单独醒目写明迁移 0028（users 表新增可空 timezone 列）：Vercel Production 构建先自动执行 db:migrate；自部署（含 Docker）升级前手动执行 db:migrate；未迁移的库上会话解析不报错，但密码登录、设备轮询、团队相关查询与时区设置会出错，必须先迁移再上线；措辞与 web/vercel.json、docs/deploy 文档逐条核对一致。
4. 版本推进：scripts/set-version.sh 1.0.5 实跑成功且只改预期文件集；CHANGELOG Changed 写对应条目（照 1.0.4 写法）；若未执行写明原因。
5. 验证：cargo metadata 通过（本机 rustc 1.85.1 低于代码所需的 1.87，cargo check 不能本地实跑，写明理由）；两份 CHANGELOG 条目数一致；Markdown 相对链接目标存在。web 侧仅 scriptsRef.ts 常量值由脚本推进，无类型面与运行时行为变化，不跑 web build，写明理由。
6. 本文件模型表为真实计数；本文件通过禁用词与本机路径扫描，零命中；不写密钥。

## Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill；时区、榜单缓存、MCP 名称等术语在四个已合并任务中已定型，无模型歧义 | 需求 | not-required |
| book-ddia-data-design | 不改 schema／migration／缓存格式／数据形状；迁移 0028 已随 PR #24 合并，本轮仅描述其执行次序 | 设计 | not-required |
| book-legacy-change-safety | 不改变既有运行行为；无 bug 修复 | 首次行为修改前 | not-required |
| book-refactoring-pass | 修改既有生产文件（web/src/lib/scriptsRef.ts 版本常量、README／docs 钉版 ref） | 首次生产编辑前 | passed（proceed；normal；no refactor needed；结论见门禁结论节） |
| book-release-readiness | 版本发布准备、迁移执行次序说明 | 验证后、收尾前 | blocked（tag、CI、合并、部署与生产迁移执行不在本会话授权内；结论见门禁结论节） |

BDD: skipped（本轮不改变既有运行规则，仅改说明与版本常量），不新增或修改 .feature。独立评审由 Reviewy 在 PR 上进行（项目既定流程，同 PR A／B／C／D／F），本会话不启动子代理。Graft: skipped（改动为文档与版本常量，无符号或调用关系变更，图证据无增量）。验证均用原生命令，不经 rtk。

## 进度

- 2026-11-14 21:14:21 +08:00 主 agent：开始任务。核对根、分支、授权与范围；起点 HEAD 92776813d64510923a89271181ddb8d844861295，分支 docs/v1.0.5-changelog，工作区干净（git status 实测）；创建本文件，随后在 ai/tasks/index.md 登记，再改文件。created_at 与首条状态事件取创建命令发出前最后一次真实时钟读数（21:14:21.471434），创建在其后数秒内完成；初稿的微秒读数在最终定稿覆盖时未留存，不重造。
- 2026-11-14T21:25:21.275093092+08:00 主 agent：完成全部实施与验证（本条目在验证完成后写入；下列步骤按序完成于 21:14 至本条目写入时刻之间，逐条命令输出为证，结果全文见证据节）：
  1. 完成调查：通读四个合并提交（353e3a6、a856909、8715c0e、9277681）的 git show --stat 与 PR A／B／C／D 四份 task.md、PR #17 的 task.md set-version 节；核对 web/vercel.json、docs/deploy/self-host-production.md（中英）、docs/deploy/docker/prod/README.md、scripts/set-version.sh 原文；源码核对 session.ts 降级读取与 login／device poll／teams 的整行 select。
  2. 完成门禁前置：Book Gate Plan 登记，Refactoring Review 结论 proceed（no refactor needed）。
  3. 完成版本推进：先在会话临时目录以同布局副本预检 `cargo update --workspace --offline`（exit 0，仅 tokens-cli／tokens-core 两行 version 变化，证明脚本不需要网络，验毕删除副本），再实跑 `bash scripts/set-version.sh 1.0.5`（exit 0）；改动文件集经 git status 核对恰为 v1.0.4 记录的同一 16 个文件。
  4. 完成双语 CHANGELOG：两文件各新增 [1.0.5] - 2026-11-14 一节（新增 5／修复 8／变更 3，合计 16 条，两种语言一一对应），置于 [Unreleased] 之下，迁移 0028 为变更组首条并加粗。
  5. 完成验证：cargo metadata --no-deps --offline exit 0（tokens-cli 1.0.5、tokens-core 1.0.5）；条目计数、Markdown 链接、残留版本号、禁用词／本机路径／密钥扫描、文件集边界核对全部通过（数字见证据节）。
  6. ponytail-review：not-needed（生产 diff 仅脚本产出的版本常量与一个字符串 ref，无手写生产逻辑、无投机抽象）。Code Readability Review：手写部分为两份 CHANGELOG 文档，逐条对照一一对应；脚本产出文件不属手写代码，未改 vendor／generated。

## 条目与来源

| 条目 | 来源 | 处理 |
|---|---|---|
| PR A #20 多语言框架（web＋CLI，en／zh-CN／ja，切换器、URL 前缀、cookie 映射、旧 URL 重定向） | 合并提交 353e3a67ab2274f1c33aef483dca54b86d64c889 git show --stat；ai/tasks/20261114-v2721-pr-a-i18n/task.md | Added 第 1 条 |
| PR B #22 昨日期间（榜单与资料页） | 合并提交 a85690935f94d45f94b57873f15620c7a84a3d40 git show --stat；ai/tasks/20261114-v2721-pr-b-mixed-fixes/task.md 验收 2 | Added 第 2 条 |
| PR C #23 OG 卡片重做（三类卡、三语、资料卡指标） | 合并提交 8715c0e6f0464801f1ebc1a3144e8028ff06c9d3 git show --stat；ai/tasks/20261114-v2721-pr-c-remaining/task.md 验收 3 | Added 第 3 条 |
| PR C #23 Cursor 桌面端登录与统一计费（33b1723） | 同上 task.md 目标与验收 1 | Added 第 4 条 |
| #21／PR #24 时区设置与按看的人归日 | 合并提交 92776813d64510923a89271181ddb8d844861295 git show --stat；ai/tasks/20261114-v2721-pr-d-timezone/task.md | Added 第 5 条 |
| PR B #22 Cursor 增量同步（e4fa4ed，水位线、按事件标识合并、空增量不动缓存、冷缓存全量） | PR B task.md 验收 1 | Fixed 第 1 条 |
| PR B #22 榜单整榜下发、客户端定位 viewer、不再流式（36d17e1、b3a0a60） | PR B task.md 验收 2 | Fixed 第 2 条 |
| PR C #23 GitHub 限流不清空社交链接（4c98999） | PR C task.md 目标与验收 2 | Fixed 第 3 条 |
| PR C #23 解析器修复（Devin request_id 去重、Codex .jsonl.zst、Pi／DSH 少计、compaction、Cursor 快照去重、DSH 负值归一） | PR C task.md 目标与验收 4 | Fixed 第 4 条 |
| PR C #23 Windows autosubmit 与 tokensw（79feedb、583c5b5、33ee973） | PR C task.md 目标与验收 6 | Fixed 第 5 条 |
| PR C #23 转私密清除排行榜缓存（方案甲：统一标签即时过期＋三语榜单地址 revalidatePath，两条转私密入口共用） | PR C task.md 目标与已确认范围节（群成员投票 2026-11-14 13:59–14:12 方案甲，Lord 14:12 记录）、验收 10 | Fixed 第 6 条 |
| PR C #23 数据库抖动不缓存 404（7f30fd6） | PR C task.md 目标、门禁结论 DDIA 失败路径 | Fixed 第 7 条 |
| PR B #22 设置页空状态图标居中（d6aca2c） | PR B task.md 进度第四轮b | Fixed 第 8 条 |
| 迁移 0028 执行次序与未迁移后果 | web/vercel.json buildCommand 与 git.deploymentEnabled 原文；docs/deploy/self-host-production.md:73 与同 .en.md:73；docs/deploy/docker/prod/README.md:24-35；web/src/lib/auth/session.ts 降级读取；web/src/app/api/auth/login/route.ts、api/auth/device/poll/route.ts、lib/teams/service.ts 整行 select 核对 | Changed 第 1 条（醒目加粗） |
| PR C #23 MCP 名称默认不上报（42e63c2 功能部分：tokens mcp-names、shareMcpServerNames、--include-mcp、mcpServersOptIn 门控、--dry-run --json） | PR C task.md 目标与验收 5 | Changed 第 2 条 |
| 版本推进 1.0.5/v1.0.5 | scripts/set-version.sh 实跑；PR #17 同一机制 | Changed 第 3 条 |
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
- Failure modes and safeguards：钉版 ref 指向的 v1.0.5 tag 在 Lord 打 tag 之前不存在，原始 URL 暂时 404——v1.0.4 发布沿用同一机制同一窗口（scripts/set-version.sh 注释与 PR #17 记录），属既定发布次序，不是本轮新增风险。CHANGELOG 的迁移次序若写错会误导部署——已逐条对照 web/vercel.json、docs/deploy/self-host-production.md（中英）、docs/deploy/docker/prod/README.md 原文与 web 源码核对；迁移 0028 本身的 rollout 次序评审见 PR #24 task.md 的 Release Readiness 评审（维持 blocked），本轮不重复裁决。
- Capacity / backpressure / limits：无运行时变化，not-applicable。
- Observability / alerts / runbook：无新增，not-applicable。
- Rollout / migration / rollback / cleanup：本轮只产出文档与版本常量；提交、推送、开 PR、Reviewy 评审、合并、tag v1.0.5、发布、生产部署与迁移执行均不在本会话授权内，由 Lord 按顺序另行处理。回滚即还原文档与清单文件，无数据变更。
- Required validation and result：set-version.sh exit 0（16 个文件，与 v1.0.4 记录同一文件集）；cargo metadata --no-deps --offline exit 0（tokens-cli／tokens-core 1.0.5）；双语条目 16/16（新增 5／修复 8／变更 3）一一对应；Markdown 相对链接目标存在；v1.0.4 残留扫描零命中；根 package.json、web/package.json、bun.lock 零改动；禁用词／本机路径／密钥扫描零命中。
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
- CHANGELOG 对照：两份文件 [1.0.5] 节各 16 条（新增 5／修复 8／变更 3），分组顺序与 [1.0.4] 相同，条目一一对应（撰写时逐条对照，计数经 awk 实测）。
- Markdown 链接：两份 CHANGELOG 的相对链接仅 ](./CHANGELOG.zh.md) 与 ](./CHANGELOG.md)，目标均存在；[1.0.5] 节未新增链接。
- 迁移措辞逐条核对：web/vercel.json buildCommand 原文（仅 VERCEL_ENV 为 production 时先执行 db:migrate 再 bun run build；git.deploymentEnabled.main 为 false，main 不自动部署）；docs/deploy/self-host-production.md:73 与同 .en.md:73（已在跑旧版本的实例升级到含 0028_add_user_timezone 的版本：先手动执行 DATABASE_URL=… bun run db:migrate 再构建上线，未迁移时设置接口时区读写返回 503）；docs/deploy/docker/prod/README.md:24-35（发版流程 up -d --build，有迁移时在 web 容器内 docker compose exec web bun run --cwd web db:migrate）；web/src/lib/auth/session.ts（主查询只取消费列，timezone 单独查询、失败降级 null）；web/src/app/api/auth/login/route.ts、api/auth/device/poll/route.ts、lib/teams/service.ts 存在整行 .select() 查 users（未迁移的库上会出错）。
- 禁用词与路径扫描：任务列出的全部禁用词与三类本机路径前缀对本文件全文、两份 CHANGELOG 的本轮新增节扫描，零命中；初扫命中本文件一处含禁用字的措辞，已当场改写。CHANGELOG.zh.md 旧 [1.0.4] 节另有一处历史措辞命中禁用字（2026-10-25 随 v1.0.4 发布，早于本任务禁用词表），属既有内容，不在本轮改动范围，保持原行不动。密钥模式扫描零命中。

## 模型与调用

主 agent 用 k3:max，密钥从环境变量 KIMI_API_KEY 读取。

| 角色 | 模型 | 密钥来源 | 调用次数 |
|---|---|---|---|

统计口径：回复 = 该轮会话文件 assistant 消息条数；工具调用 = tool_execution_start 条数；工具报错 = 会话文件中标记为报错的工具结果条数（含命令非零退出）。每轮一行取该轮会话文件的最终计数；第一轮计数要等本轮结束才有，不写进表。

未启动子代理；密钥不写入本文件与任何产物。

## 下一步

本任务 done（验收条件 1–6 证据成立，见证据节）；release-readiness 维持 blocked。提交、推送、开 PR 与 Reviewy 评审由 Lord 按顺序在 omp 之外处理；tag v1.0.5、发布、生产部署与迁移执行不在本会话授权内。本会话不提交、不推送、不打 tag、不开／改 PR、不访问生产。ignored 的 .sbtd/active-task.json 只存引用，不随 Git 保存；跨环境续接以本文件为准，不承诺自动恢复。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-14T21:14:21.471434+08:00 | planned | in-progress | 范围与授权核对完成，开始双语 CHANGELOG 与版本推进。 | 起点 HEAD 92776813d64510923a89271181ddb8d844861295；PR #17 task.md set-version 节；PR A／B／C／D 四份 task.md；web/vercel.json 与 docs/deploy 文档原文。 |
| 2026-11-14T21:25:21.275093092+08:00 | in-progress | checking | 版本推进与双语 CHANGELOG 完成，验证全部实跑通过，进入核对。 | 命令与结果全文见证据节：set-version.sh exit 0（16 文件）；cargo metadata --no-deps --offline exit 0；条目 16/16；扫描零命中。 |
| 2026-11-14T21:25:22.278717000+08:00 | checking | done | 核对完成：验收条件 1–6 逐条满足；git status 无范围外改动；不提交、不推送；release-readiness 维持 blocked（tag、CI、合并、部署与生产迁移执行在会话外）。 | 证据节全节；git status --porcelain 文件集；复扫零命中。 |
