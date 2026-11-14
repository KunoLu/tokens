---
schema_version: 1
id: 20261114-v2721-pr-b-mixed-fixes
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-11-13 10:32 群内选定多语言方案甲、10:33 确认 A/B/C 拆法；2026-11-14 01:50 用户在群内开启全自动模式直到
  v1.0.5 按模板发布且文档补全，Lord 记录；本会话明确选择 strict。
status: "done"
branch: sync/upstream-v27.2.1-pr-b
created_at: '2026-11-14T07:01:21.131576+08:00'
updated_at: "2026-11-14T11:28:42.000000+08:00"
completed_at: "2026-11-14T10:05:04.229193+08:00"
---
# 同步上游 v27.2.1 拆分 PR B — 混合文件的非多语言修复

## 目标

把 PR A「碰到但未做的文件」登记的混合文件中、上游的非多语言修复带进本仓：

- `cli/tokens-cli/src/cursor.rs` 的 sync_start 增量同步及其测试模块（上游 e4fa4ed，上游 #114 的 CLI 侧）；
- 整榜下发（36d17e1）、榜单流式修复（b3a0a60）、昨日卡片（ba3ad78）；
- 上游 MCP opt-in（42e63c2）在 cbf1ecd 之后对 docs/privacy/legal 词条的修订，以及 v27.2.1 相对 cbf1ecd 多出的 23 条 CLI 词条（commands.mcpNames.* 11 条、cli.mcp-names.* 5 条、cli.submit/cli.cursor 相关 7 条），只取与上述混合文件相关的部分；
- 上游 16724c8 版设置页：评估后按本仓自有隐私模式实现保留，只取不冲突的修复，取舍逐条记录。

保持本仓差异：/settings 未登录跳 /login、无 shame 页、无 GitHub 登录与 verified 徽章、品牌色、邮箱登录、teams/teamboard。新增与改动文案 en、zh-CN、ja 三语齐备。f114057..e39110d 共 149 个上游提交逐个归类（见「上游提交归类」），C 类清单可直接作为下一 PR 输入。

## 已确认范围与授权

- 用户 2026-11-13 10:32 群内选定多语言方案甲、10:33 确认 A/B/C 拆法；2026-11-14 01:50 用户在群内开启全自动模式直到 v1.0.5 按模板发布且文档补全，Lord 记录。
- 本会话明确选择 strict 模式。不 commit、不 push，只改工作区文件。
- 起点：`main` 353e3a67ab2274f1c33aef483dca54b86d64c889（已含 PR A #20）；分支 `sync/upstream-v27.2.1-pr-b`，开工前 `git status` 实测干净。上游 `missuo/tokens` tag v27.2.1 = e39110ddc5ca9b2b8b25abcf57738b23931755d1，merge-base f114057。
- 范围锚点：PR A task.md「碰到但未做的文件」一节登记的混合文件，逐个核对。OG 链路（api/og/route.tsx、lib/og、scripts/generate-og-fonts.ts、og-cjk-hanzi.txt）在该清单中登记为随 OG 重做另行处理，不在本 PR；worker.ts、VerifiedBadge、shame 页登记为不适用本仓，维持该结论。
- 不在本 PR（归 PR C）：与上述混合文件无关的上游其它功能与修复提交（codex compaction、devin/pi/dsh 解析、cursor 登录与计费、Windows autosubmit、缓存类修复、GitHub 限流清空社交链接的 4c98999、OG 重做、publish 工作流修订等），完整清单见「上游提交归类」C 类。
- 未完整调用 grill-with-docs：范围、来源、验收与授权均已明确，需求无歧义，不重复访谈。

## 验收条件

1. cursor.rs 增量同步落地：缓存水位线减修正窗与钟差得到 startDate；合并为按事件标识的并集，新拉取覆盖同标识缓存行，只覆盖近段的增量不丢历史；空增量不动缓存；冷缓存仍全量。e4fa4ed 的测试模块随代码一并移植并通过。
2. 36d17e1/b3a0a60/ba3ad78 落地：榜单整榜下发、客户端在整榜中定位 viewer；榜单不再流式渲染，失败可成 500；昨日数据在榜单与资料页可用；middleware 共享缓存清单按本仓实际补 /leaderboard（本仓无 /shame，不加）；teams/team 列等本仓自有差异保留。
3. 42e63c2 词条：docs/legal 命名空间按本仓自有结构补齐三语并在页面渲染；23 条 CLI 新词条三语齐备；既有 coverage 测试继续通过。
4. 16724c8 设置页评估完成，取舍逐条写入「取舍说明」；本仓自有隐私模式（api/settings/privacy、api/me/private、settings/read-tokens、lib/privacy.ts）保留。
5. 验证全部真实执行并记录：`cd web && bunx tsc --noEmit`、`bun run lint`、`bun test`、`bun run build`；`cargo test --manifest-path cli/Cargo.toml`、`cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`、`cargo build --release --manifest-path cli/Cargo.toml -p tokens-cli`。本机预装 rustc 1.85.1 低于代码所需的 1.87，CLI 验证使用会话临时目录内的隔离工具链（PR A 架设的 rustup stable 与手工解包 make 4.4.1 仍在，不改 HOME、不动仓内配置）。
6. 149 个提交归类表完整，C 类清单可直接作为下一 PR 输入。
7. 本文件模型表为真实计数；本文件通过禁用词与本机路径扫描，零命中。

## 模型与调用

主 agent 用 k3:max，密钥从环境变量 KIMI_API_KEY 读取。

| 角色 | 模型 | 密钥来源 | 调用次数 |
|---|---|---|---|
| 主 agent | k3:max | 环境变量 KIMI_API_KEY | 工具报错 = 会话文件中标记为报错的工具结果条数（含命令非零退出）；第一至五轮的工具报错均为会话结束后按会话文件统计的终值。第一轮：回复 191、工具调用 264、工具报错 9；第二轮：回复 15、工具调用 25、工具报错 0；第三轮：回复 7、工具调用 8、工具报错 1；第四轮：回复 12、工具调用 17、工具报错 2（启动后未改文件即中止）；第四轮b：回复 32、工具调用 61、工具报错 2；第五轮：回复 7、工具调用 18、工具报错 0；第六轮（本轮）：回复 4、工具调用 11、工具报错 0（截至 2026-11-14 11:28 +08:00 本次写入时） |

未启动子代理；密钥不写入本文件与任何产物。

## Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill；viewer／昨日卡片／隐私模式等术语在本仓代码与既有任务中已定型，无模型歧义 | 需求 | not-required |
| book-ddia-data-design | cursor 本地缓存合并策略与水位线、榜单整榜数据形状与共享缓存清单、昨日数据边界 | 设计稳定前 | passed（confirmed） |
| book-legacy-change-safety | 改变 cursor 同步与榜单渲染的既有行为，现有行为需刻画 | 首次行为修改前 | passed（characterized；normal） |
| book-refactoring-pass | 修改既有生产代码（cursor.rs、榜单与资料页组件、middleware、词条文件） | 首次生产编辑前 | passed（proceed；normal；no refactor needed） |
| book-release-readiness | 榜单路由与 middleware 缓存行为、CLI 同步行为变更 | 验证通过后、收尾前 | blocked（本地门禁全绿；CI、合并、部署不在本会话授权内） |

TDD：cursor 增量同步的测试模块随 e4fa4ed 一并移植，先落地测试确认其在本仓因缺实现而失败，再补实现转绿；web 侧沿用既有测试与 coverage 测试兜底。

## 进度

- 2026-11-14 06:58 +08:00 主 agent：开始调查与归类（PR A 清单、上游 149 提交、本仓现状核对）。
- 2026-11-14 06:58 +08:00 主 agent：完成调查与归类。结论：上游 v27.2.1 的 cursor.rs 相对本仓的剩余差异＝e4fa4ed 增量同步（本 PR）＋33b1723/3e3a5ab/d1cc44c 登录与计费（归 C）＋59f31ae 多语言（已在 A）；42e63c2 的 docs/legal 词条需按本仓自有结构改写（本仓 legal.privacy 为 h./short./collect./never./public./choices. 结构，docs 为扁平键），CLI 23 条新词条三语上游已备可直接采用；本仓已具备 16724c8 的等价自有实现（api/settings/privacy、api/me/private、settings/read-tokens、lib/privacy.ts、getLeaderboard 私密过滤均存在）。逐条归类见下节。
- 2026-11-14 07:14 +08:00 主 agent：开始基线刻画（legacy-safety 证据）。
- 2026-11-14 07:14 +08:00 主 agent：完成基线刻画。发现工作区 node_modules 缺失（6 条测试报 Cannot find module），以 bun install --frozen-lockfile 恢复后基线全绿：bun test 100 pass／0 fail、tsc exit 0、lint 0 errors＋1 条既有 warning、cargo test 2467 pass／0 fail／4 ignored（会话临时目录的隔离工具链）。另有两处范围修正写入取舍说明：6201946 是 ba3ad78 的结构前提（同在两个混合文件内），随本 PR 一并移植；23 条 CLI 新词条中 8 条 .about/.help 受 main.rs 目录树完整性测试限制，须随 C 的命令落地，本 PR 先落 15 条。
- 2026-11-14 07:46 +08:00 主 agent：开始 cursor.rs 移植（e4fa4ed，先红后绿）。
- 2026-11-14 07:52 +08:00 主 agent：完成 cursor.rs 移植。13 条增量同步测试先落编译失败（E0425）为红，实现落地后 tokens-cli 228 pass／0 fail。
- 2026-11-14 08:35 +08:00 主 agent：开始 web 移植（整榜下发、流式修复、昨日卡片、middleware、词条）。
- 2026-11-14 09:10 +08:00 主 agent：完成 web 移植与词条落地。期间一次模糊匹配误伤 getLeaderboard.ts 邻近注释，tsc 当场暴露并已修复（记入门禁结论）。lint 2 条 warning：base 既有 docs/page.tsx img 1 条，另 1 条为本轮改动使 ProfilePageClient 一行 eslint-disable 多余（unused-disable）。
- 2026-11-14 09:23 +08:00 主 agent：完成全部验证与冒烟（结果见证据节）、ponytail-review（Lean already）、可读性复核、release-readiness（blocked：CI/合并/部署不在本会话授权内）。开始最终核对与收尾。
- 2026-11-14 09:33 +08:00 主 agent：开始第二轮恢复核对与验证重跑。恢复核对：git status 与第一轮记录一致（39 个修改、1 个新增，均未提交）；node_modules 在位（485 个顶层条目，bun.lock 未动）；PR A 架设的会话临时目录隔离工具链在位（rustup stable：rustc 1.100.0、cargo 1.100.0、clippy 0.1.100，另有手工解包 make 4.4.1），本机预装 rustc 1.85.1 低于代码所需的 1.87，CLI 验证继续走隔离工具链；无第一轮遗留的 next start 等后台进程。第一轮已记 done，但其验证证据未经实跑确认即触会话上限，按重开流程回 planned 再进 in-progress，一切以本轮实跑为准。
- 2026-11-14 10:05 +08:00 主 agent：完成全部验证重跑与冒烟，七条命令全绿且数字与第一轮记录一致（逐项见证据节「第二轮复跑」）；生产构建 curl 冒烟 18 项全过，进程已关闭；CLI 产物冒烟正常（tokens --version 输出 1.0.4，zh-CN 与 ja 帮助输出正常）。完成 task.md 本轮补齐（进度、复跑证据、模型表两轮计数、状态事件）。
- 2026-11-14 10:08 +08:00 主 agent：第三轮只改计数：模型表换用第一、二轮会话结束后的最终计数（191/264/1、15/25/0），补记第三轮截至写入时的计数（回复 5、工具调用 7、工具报错 0），模型表上方补一句模型与密钥来源说明；禁用词与本机路径扫描零命中，无其他改动。
- 2026-11-14 10:52 +08:00 主 agent：第四轮b 按审查修正（此前第四轮启动后未改文件即中止，本轮接续）：d6aca2c 由不适用改归本 PR——设置页空状态图标按上游改为 mb-3 flex justify-center opacity-50（原 mx-auto 写法因 svg 为块级而实际偏左）；4c98999 由不适用改归 C——资料页路径 u/[username]/page.tsx → githubSocials.ts 的 syncGitHubSocialLinks 不看 complete 就写库，限流会把社交链接写成空；删除 ProfilePageClient 多余的 react-hooks/set-state-in-effect 禁用行，lint 回到 0 errors、1 warning（只剩 docs/page.tsx 的 img）；归类统计改 A 9／B 8／C 20／不适用 112；e4fa4ed 测试数 12 改 13；b624e57 理由改正为 generateMetadata 读 headers 使 /device 成动态页；取舍说明补 merge_cursor_usage_events 早退分支、榜单共享缓存 90 秒窗口、MCP 文案与 v1.0.5 发布依赖三条；leaderboard page.tsx 注释中 deploy layer 一说改正为本仓实际。验证全部实跑（tsc、lint、bun test、build、cargo test -p tokens-cli），结果见证据节「第四轮b 复跑」。
- 2026-11-14 11:12 +08:00 主 agent：第五轮按复审意见只改本文件文字与数字，不改代码、不 commit、不 push：不适用小类改合并提交 55／上游版本线 12／本仓已落地 26／结构或体系不同 19（合计 112 不变）；模型表第四轮改 12／17／2、第四轮b 改 32／61／2（均为会话结束后的最终计数）并补第五轮截至写入时计数；lint 基线说法改正——base 353e3a6 只有 docs/page.tsx 的 img 1 条 warning，本 PR 第一轮改动使 ProfilePageClient 一行 eslint-disable 多余而多出 1 条，第四轮b 已删除；删去「首次输出不全」「HEAD 原样重跑同为 2 条」说法，进度、证据、门禁结论与状态事件同步；第四轮b 复跑时段改 10:42–10:55 +08:00；取舍 10 空缓存措辞严谨化；状态事件补本轮更正行。禁用词与本机路径扫描零命中（grep 实测）。
- 2026-11-14 11:28 +08:00 主 agent：第六轮只改本文件文字与数字，不改代码、不 commit、不 push：统一工具报错口径——工具报错 = 会话文件中标记为报错的工具结果条数（含命令非零退出）；第一至五轮工具报错按会话结束后会话文件统计的终值改正为第一轮 9、第二轮 0、第三轮 1、第四轮 2、第四轮b 2、第五轮 0；第五轮回复与工具调用改终值 7／18；模型表补第六轮截至写入时计数（回复 4、工具调用 11、工具报错 0，截至 2026-11-14 11:28 +08:00）；11:12 进度条目与 11:12:34 状态事件行内的旧工具报错数（第四轮 1、第四轮b 0）同步改正；10:08 进度条目为截至当时写入时的旧记录，保留原样；状态事件补本轮更正行。禁用词与本机路径扫描零命中（grep 实测）。

## 上游提交归类（f114057..e39110d，共 149 个）

统计：已在 A 9、本 PR B 8、留给 C 20、不适用本仓 112（其中合并提交 55、上游版本线 12、本仓已落地 26、结构或体系不同 19）。42e63c2 与 16724c8 跨类：词条与设置页评估在 B，功能代码与冲突部分归 C 或不适用，表内注明。

| 提交 | 归类 | 依据 |
|---|---|---|
| e39110d chore(release): 27.2.1 | 不适用本仓 | 上游版本号提交；本仓自有 v1.0.x 版本线与发布流程 |
| 633874f..b8da52d 共 55 个合并提交（633874f 4676e4a 76cb59f ef219a7 0783fc9 f46d2d2 a0b15c9 3cf250f c70b081 f2f054e f3767c9 c2f3857 24c8329 2a5f1ba 390df73 0fc8784 5d25f78 65e050a 3665ef3 496e117 9f3f9ae 349a057 4f53dca b3c0a9b ed70137 f50410e b7e5935 d466784 94502e5 73ae3e6 81c275f 92845c0 4361fa7 a7faf94 2ba729e 98d3443 637deaa 67ea7c1 09736a9 d3df8c5 0334745 030653f 1e547ef 3715155 e1bc1a5 36a55d7 986e61d 21c1d02 3ee911c dc13438 69c1ce9 cc67eb7 4bcbe47 b0b8114 b8da52d） | 不适用本仓 | 合并提交，无独立 diff；内容随对应内容提交归类（cbf1ecd、d5a9f8b 两个合并提交已在 A，单列下行） |
| 42e63c2 fix(privacy): MCP server names opt-in | 本 PR B（部分） | B 取 docs/legal 词条修订与 CLI 词条；功能代码（mcp_names.rs、tokens-core/mcp.rs、settings.rs、main.rs、api/submit、validation、docs/privacy 页面结构）归 C |
| cba9c22 fix(parsers): Devin 去重＋Codex .jsonl.zst | 留给 C | 解析器修订：sessions/devin.rs、codex.rs、scanner.rs、message_cache.rs、clients.rs、lib.rs、main.rs、validation/submission.ts（本仓 devin.rs 仍按 row_id 去重，无 request_id） |
| 38b0a15 fix(parsers): Pi/DSH 少计 | 留给 C | pi.rs entry_identity_first 与 dsh.rs 流式读取重做，本仓均无 |
| 2ac6cff docs(ios): 私有资料 read tokens | 不适用本仓 | 本仓 docs 页无 iOS 章节，docs 文案自有 |
| f612b32 chore(release): 27.2.0 | 不适用本仓 | 上游版本号提交 |
| 79feedb style(cli): schtasks UTF-16 as_chunks | 留给 C | Windows autosubmit 相关（commands/autosubmit.rs） |
| cbf1ecd Merge PR #117 feat/i18n | 已在 A | PR A 已取该合并的多语言内容（cbf1ecd 时点） |
| 583c5b5 fix(cli): Windows 任务迁移无窗启动器 | 留给 C | Windows autosubmit 相关（autosubmit.rs、tokensw.rs、main.rs、Cargo.toml、README） |
| 1ef5a3c test(cursor): 解析测试缓存目录按平台 | 留给 C | tokens-core lib.rs 测试辅助，随 cursor 核心解析修订一并处理 |
| 7ed4f29 test(i18n): 语言切换测试串行 | 已在 A | PR A 内容 |
| d1cc44c fix(cursor): 本地登录门控与跨端点去重 | 留给 C | cursor.rs/cursor/billing.rs/cursor/cli_auth.rs/main.rs 的非 sync_start 部分 |
| 6011333 fix(i18n): 评审修订 | 已在 A | PR A 内容 |
| d5a9f8b chore(merge): main 合入 feat/i18n | 已在 A | #117 分支内合并，随 cbf1ecd 生效 |
| ba3ad78 feat(ui): 昨日卡片 | 本 PR B | leaderboard/page.tsx、api/leaderboard/route.ts、ProfilePageClient、Leaderboard.tsx、ProfileToday、getLeaderboard、types |
| 3e3a5ab feat(cursor): CLI 账号登录发现 | 留给 C | cursor.rs、cursor/cli_auth.rs、main.rs、clients.rs |
| 33b1723 fix(cursor): desktop 登录与统一计费 | 留给 C | cursor.rs、cursor/billing.rs、cursor/desktop.rs、main.rs、clients.rs |
| 59f31ae feat: CLI 与分享卡多语言 | 已在 A | PR A 内容（OG 链路文件除外，登记随 OG 重做处理） |
| 6249779 feat(web): 界面三语化 | 已在 A | PR A 内容 |
| f367951 fix(codex): compaction 跨元数据保留 | 留给 C | sessions/codex.rs（本仓无 pending_compaction 机制） |
| 6554b27 fix(codex): compaction 遗漏计数 | 留给 C | sessions/codex.rs、message_cache.rs |
| e4fa4ed fix(cursor): 只同步缓存未覆盖部分 | 本 PR B | cursor.rs 的 sync_start 增量同步及其测试模块 |
| d9a213a fix(cursor): 快照跨事件只计一次 | 留给 C | sessions/cursor.rs、message_cache.rs；与 e4fa4ed 同属上游 #114，拆分解析侧去重，风险记入取舍说明 |
| e17eeb3 fix(deploy): 静态资源跨部署可用 | 不适用本仓 | 本仓无 Dockerfile/compose.yaml/docker-entrypoint.sh 部署结构 |
| b624e57 fix(web): device 页退出共享缓存 | 不适用本仓 | 本仓 /device 的 generateMetadata 经 getTranslator() 读 headers 做语言检测，页面因此是动态渲染（本轮构建实测为 ƒ dynamic），不会被静态化后遭共享缓存长期持有，上游要修的问题在本仓不成立；middleware 清单也不含 /device |
| e2fd454 fix(core): DSH 负值归一 | 留给 C | sessions/dsh.rs、message_cache.rs（本仓无 clamp(0, output) 归一） |
| 33ee973 fix(cli): Windows autosubmit 无控制台窗口 | 留给 C | Windows autosubmit 相关（publish-cli.yml、README、Cargo.toml、tokensw.rs、autosubmit.rs） |
| c10bc0e chore(release): 27.1.5 | 不适用本仓 | 上游版本号提交 |
| 2d906e1 feat(web): 周期模型用量表速度分层 | 不适用本仓 | 已在本仓：ProfileModels 含 getSpeedTierRows/SpeedTierLabel（v1.0.4 系列） |
| bfac6f6 feat(web): 资料页模型速度分层 | 不适用本仓 | 已在本仓：SpeedTier 组件与 speedUsage.ts 在仓 |
| 00579f7 refactor: codex 速度状态命名 | 不适用本仓 | 已在本仓：随 v1.0.4 speed 后端落地 |
| 1216cdd refactor: 速度命名简化 | 不适用本仓 | 已在本仓：speed.rs、speedUsage.ts 在仓 |
| 511c710 chore(release): 27.1.4 | 不适用本仓 | 上游版本号提交 |
| 412195b ci: 非镜像路径不重建生产 | 不适用本仓 | 本仓无 deploy-self-hosted 工作流 |
| 42d7185 feat: reported/selected 速度保留 | 不适用本仓 | 已在本仓：speed.rs 区分 provider-reported 与 selected；web 侧本仓结构为 speedUsage.ts（无 speedBreakdown.ts，结构差异已核对） |
| 73e413d ci: 跳过任务的汇报名修正 | 不适用本仓 | 本仓 ci.yml 明确不采用按路径跳过（文件内注释已述理由） |
| 27e99cf ci: 跳过未触及任务族 | 不适用本仓 | 同上 |
| c0002de feat(web): 成分图占比 | 不适用本仓 | 已在本仓：TokenBreakdown 以 formatPercent 渲染占比 |
| 427b514 feat: Paseo 托管会话 | 不适用本仓 | 已在本仓：tokens-core/paseo.rs、docs.paseoUsage 词条在仓 |
| 31d7688 feat(web): 设置页云端会话分组 | 不适用本仓 | 已在本仓：SettingsClient 使用 groupCloudDevices |
| b39ac7d fix(cloud): 云端会话设备评审修订 | 不适用本仓 | 已在本仓：lib/devices/cloud.ts、cloud.test.ts 等在仓 |
| c5dfbff fix(web): OMP 官方图标 | 不适用本仓 | 已在本仓：constants.ts 引用 client-omp.png |
| 119c98d fix(antigravity): 单会话刷新失败不拖垮 sync | 不适用本仓 | 已在本仓：随 v1.0.4 antigravity 落地（partial 字段在仓） |
| 2a22dc6 fix(antigravity): 测试辅助 cfg(test) 门控 | 不适用本仓 | 已在本仓：antigravity.rs 含 cfg(test) 门控 |
| 797b4a4 fix(antigravity): 隐式同步限界与部分结果 | 不适用本仓 | 已在本仓：antigravity.rs 含 partial/partial_error |
| 848849e feat(cli): Claude Code 云端会话上报 | 不适用本仓 | 已在本仓：tokens-cli/src/cloud.rs 在仓 |
| 2dc82fe fix(antigravity): 提交前自动同步 | 不适用本仓 | 已在本仓：main.rs 含 include_antigravity 与 has_antigravity_cache 逻辑 |
| 059cb69 chore(release): 27.1.3 | 不适用本仓 | 上游版本号提交 |
| 392053b fix(claude): 1 小时缓存写定价 | 不适用本仓 | 已在本仓：pricing/mod.rs 有 1 小时费率推导 |
| 651e757 feat(cli): 模型 ID 前缀归一 | 不适用本仓 | 已在本仓：model_normalization.rs 在仓并被 lib.rs 引用 |
| c281983 chore(release): 27.1.2 | 不适用本仓 | 上游版本号提交 |
| d6aca2c fix(ui): 设置空状态图标居中 | 本 PR B | 上游修复在本仓成立：preflight 使 svg 为 display:block，父级 text-center 移不动它，mx-auto 只居中本已通栏的容器，图标实际偏左；已按上游把 EmptyIcon 改为 mb-3 flex justify-center opacity-50（含注释） |
| 4e3d492 fix(web): 用量图表路径瘦身 | 不适用本仓 | 已在本仓：usageChartPaths.ts 在仓 |
| 7b83ac0 feat(og): 分享卡重做 | 留给 C | OG 链路（api/og、lib/og、next.config.ts 等），与 PR A 登记的 OG 多语言部分同批处理 |
| 16724c8 feat: 私有模式、read tokens、设备打码 | 本 PR B（部分） | B 只评估设置页并取不冲突修复；私有模式主体本仓自有实现已存在（api/settings/privacy、api/me/private、read-tokens、lib/privacy.ts、getLeaderboard 私密过滤），冲突部分保留本仓侧 |
| 8ca2931 docs(web): Nowdex 背书 | 不适用本仓 | 本仓 docs 页无 Nowdex/客户背书章节，文案自有 |
| 33090f4 docs(web): Nowdex 致谢 | 不适用本仓 | 同上 |
| b728ee2 fix(pi): 归集到实际服务模型 | 不适用本仓 | 已在本仓：pi.rs 的 served-model 注释与实现已在仓 |
| 7e8830f chore(release): 27.1.1 | 不适用本仓 | 上游版本号提交 |
| b4f6459 fix(codex): headless 重复计数 | 留给 C | sessions/codex.rs、message_cache.rs、main.rs、validation/submission.ts |
| ce0817e chore(release): 27.1.0 | 不适用本仓 | 上游版本号提交 |
| 3db0468 fix(codex): submit 解析版本升至 3 | 留给 C | main.rs submit_parser_revision 与 validation/submission.ts 上限 |
| 9a6b630 fix(antigravity): Windows 下门控受限 RPC | 不适用本仓 | 已在本仓：残留 8 行差异为本仓 clippy 等价改写（方向相反、更简），cfg 门控已在 |
| ae163d7 fix(cli): usage 助手仅 macOS | 不适用本仓 | 已在本仓：usage/helpers.rs 与上游逐行一致（diff 为 0） |
| 0532b9d feat(clients): Craft Agent 解析 | 不适用本仓 | 已在本仓：sessions/craft_agent.rs 与 clients.rs 条目在仓 |
| 5df7d1f feat(workbuddy): WORKBUDDY_CONFIG_DIR | 不适用本仓 | 已在本仓：scanner.rs 已实现该环境变量 |
| ab4b279 fix(commandcode): v3 缓存 token 只计一次 | 不适用本仓 | 已在本仓：commandcode 测试 fixtures 在仓 |
| 2b5aafa fix(grok): 单日志取 span | 不适用本仓 | 已在本仓：grok.rs 与上游逐行一致（diff 为 0） |
| 53082d7 feat(core): tokscale 4.17.0 同步 | 不适用本仓 | 已在本仓：sessions 全清单、mcp.rs、paseo.rs 等随 v1.0.4 系列在仓 |
| 9ecae99 ci: actions 升 Node 24 主版本 | 不适用本仓 | 本仓 ci/publish 已为 checkout@v6、cache@v5、upload-artifact@v7、setup-node@v6（即 Node 24 主版本） |
| 6fb6af5 chore(release): 27.0.5 | 不适用本仓 | 上游版本号提交 |
| 8cf8f8e ci(publish): npm OIDC trusted publishing | 留给 C | publish-cli.yml 现仅有 contents: write，无 id-token 配置 |
| 6f724b7 chore(release): 27.0.4 | 不适用本仓 | 上游版本号提交 |
| 76279f5 chore(release): 27.0.3 | 不适用本仓 | 上游版本号提交 |
| 09b1800 docs: iOS 链接指 App Store | 不适用本仓 | 本仓 docs 页无 iOS 章节，README 本仓自有 |
| 86ca4c8 chore(release): 27.0.2 | 不适用本仓 | 上游版本号提交 |
| 8415a71 fix(ci): tag 固定在构建提交 | 留给 C | publish-cli.yml 发布流程修订 |
| bfc87bb feat(api): 公开资料端点暴露 verified | 不适用本仓 | 本仓已移除 GitHub verified 徽章体系（publicProfileData 无 verified） |
| 6201946 fix(web): 资料页同日不重复 | 本 PR B | ba3ad78 的结构前提（ProfilePageClient.tsx、ProfileToday.tsx 同属混合文件），随昨日卡片一并移植 |
| 15e3a66 docs(web): 数据处理者文案 | 不适用本仓 | 本仓隐私页文案自有（share 节已列 Resend/Neon/Cloudflare） |
| 4c98999 fix(web): GitHub 限流不删徽章 | 留给 C | 本仓同样中招：资料页路径 u/[username]/page.tsx → githubSocials.ts 的 syncGitHubSocialLinks 不看 complete 就写库，GitHub 限流时会把用户的社交链接写成空，所以 C 必须带上它 |
| ae37b79 feat(deploy): 徽章定时刷新 | 不适用本仓 | 本仓无 Dockerfile/compose/web/scripts/cron.mjs |
| c3d9d47 chore(deploy): Caddyfile 入库 | 不适用本仓 | 本仓无 Caddy 部署件 |
| b3a0a60 fix(web): 榜单不再流式，失败可 500 | 本 PR B | leaderboard/page.tsx、LeaderboardSkeleton.tsx |
| 7f30fd6 fix(web): 数据库抖动不缓存 404 | 留给 C | u/[username]/page.tsx、publicProfileData.ts（缓存类修复） |
| 00c7731 perf(web): 缩短边缘窗口 | 已在 A | 机制随 PR A middleware 适配落地，CACHE_HEADER 同为 s-maxage=30、stale-while-revalidate=60 |
| 53fd5b6 docs(web): 赞助商署名 | 不适用本仓 | 本仓页脚与 docs 不含赞助商章节（ServiceFooter 注释明确） |
| 36d17e1 perf(web): 整榜下发 | 本 PR B | leaderboard/page.tsx、Leaderboard.tsx、LeaderboardSkeleton.tsx、getLeaderboard.ts、searchDirectives.ts；middleware 清单补 /leaderboard |
| bdfc273 docs(web): 赞助商独立章节 | 不适用本仓 | 同 53fd5b6 |
| c741831 perf(web): 无读者页面共享缓存 | 已在 A | 机制随 PR A middleware 适配落地，清单按本仓裁剪 |
| 2c6cbc9 perf(web): 公开资料页进共享缓存 | 已在 A | 本仓清单含 /u/:username |
| 6e4283b fix(deploy): pg 18 卷路径 | 不适用本仓 | 本仓无该 Dockerfile/compose 部署结构 |
| 77675a4 fix(deploy): Node 构建不依赖 wrangler 类型 | 不适用本仓 | 本仓 2fc41cd 已移除 worker 层 |
| 0def29d feat(deploy): 单机 Caddy 部署 | 不适用本仓 | 本仓部署为自有结构，无该批文件 |

## 取舍说明

1. **16724c8 设置页评估结论**：本仓隐私模式为独立组件 `PrivacySection.tsx`（加载失败可重试、`role="switch"`＋`aria-checked`、创建后明文一次性展示、错误经 `aria-describedby` 关联），功能等价上游 16724c8 的设置页部分。上游的 PrivacySwitch 视觉开关与 fetch 封装属纯结构差异，无行为收益，不取。取到的不冲突修复一条：**read token 撤销改为确认弹窗＋后果说明**——上游 16724c8 为 read token 撤销加了 RevokeTokenModal consequence，本仓 API token 撤销本就有确认弹窗而 read token 此前一键直撤，两处自此一致；复用 SettingsClient 的 ModalShell 等原语（六个组件加了 export），新增 `settings.readTokens.revokeConsequence` 三语词条（按本仓无 iOS 客户端的实际改写上游文案）。16724c8 其余部分本仓自有等价实现逐个核对已在仓：api/settings/privacy、api/me/private、settings/read-tokens 路由、lib/privacy.ts（含 revalidateTag 失效链路）、lib/auth/readTokens.ts、lib/devices/shared.ts 打码、PrivateProfileOwnerView、getLeaderboard 私密过滤、CLI `login --private`；schema/migration 0024 与上游 API 形状不适用本仓自建实现，settings 命名空间结构不同，上游 settings.* 18 条维持 PR A 的未引入结论。
2. **e4fa4ed（本 PR）与 d9a213a（归 C）的拆分风险**：合并函数对无 `usageEventId` 的行按含 token 数的投影键控，同一事件的两次快照会并存；上游把两个提交放在同一个 #114 合入正是为了让解析侧去重（dedupe_cursor_event_snapshots）坍缩并存行。本仓解析器在 C 落地前对并存快照各计一次；usageEventId 在上游报告路径中正常存在，无 id 只是兜底分支，风险窗口限于「增量同步启用且事件无 id」的组合。
3. **42e63c2 词条先行、功能在 C 的时序**：隐私政策与 docs 页现在描述「MCP 名称默认关闭、opt-in 才发送」，而 CLI 的 opt-in 开关（mcp-names 命令、settings.json 的 shareMcpServerNames、submit --include-mcp、服务端 mcpServersOptIn）随 C 落地。当前本仓 CLI 行为为默认提交 MCP 名称（与 42e63c2 之前上游一致），文案与行为的差在 v1.0.5 发布前由 C 闭合；若 C 推迟，本批文案需要回退。隐私页 UPDATED 日期未动，随 v1.0.5 发布模板一并处理；PR A 登记的 cookie 条文不准问题仍挂起，不在本 PR。发布提醒：MCP「默认关闭」文案已进隐私页和文档，而 CLI 当前默认仍上报 MCP 名称，v1.0.5 必须等 C 合入才能发布。
4. **23 条 CLI 新词条只落 15 条**：commands.mcpNames.* 11 条＋cli.submit.dryRunJsonHint／mcpServers／mcpServersNoneFound／mcpServersNotShared 4 条已落地（三语）。其余 8 条（cli.mcp-names.* 5 条、cli.submit.include_mcp.help、cli.submit.json.help、cli.cursor.login.source.help）受 main.rs 的 english_catalogue_matches_the_clap_tree 测试限制——该测试要求每个 .about/.help 键对应真实 clap 元素，而这些键描述的命令与开关在 C 才存在，现在加入会变 stale entries 挂测。三条既有词条值的上游修订（cli.cursor.login.about、cli.cursor.sync.about、cli.submit.dryRun）同归 C：前两条描述 C 的登录来源与 JSON 缓存改动，dryRun 的换行调整与 dryRunJsonHint 的打印顺序绑定。
5. **/api/leaderboard 保留 force-dynamic**：上游 ba3ad78 带去的 revalidate=60 不取。本仓 b39a875 的隐私修复要求账号转私密后公开响应立即失效；路由响应缓存不挂 leaderboard 标签，force-dynamic 是本仓的正确方向。页面数据层 unstable_cache 的 leaderboard 标签失效链路（privacy.ts）不动。
6. **Leaderboard.tsx 保留的本仓差异**：MembershipCells／team、group 列、avatarUrlFor、FormatToggle 的 titles 参数、PERIODS／PERIOD_LABEL_KEYS 导出形状（Teamboard 复用）。改用上游词条键 leaderboard.title／userHandle／stats.tokens／stats.cost／stats.yourRankValue／table.rank（三语新增）；FormatToggle 维持本仓 titles 方案，不引入上游 exactAria／abbreviateAria 两键。开发者链接改用 localizeHref（PR A 合并该文件时留下的无前缀链接，本次随上游状态对齐修正）。
7. **ProfileToday 的本仓差异**：日期 chip 保留 formatDate 本地化（上游显示原始 ISO 串）；成分百分比保留 formatWholePercents。词条 profile.today.dayFallback 随「固定今日」下线（三语删除），新增 yesterdayHeading／dayAria／emptyYesterday（三语采用上游译文）。
8. **Teamboard 不出现昨日选项**：其 API 无昨日通道（其它周期值会静默回落 all），共享 PERIODS 在 Teamboard 渲染处过滤并注释原因。
9. **getUserRank 保留不删**：页面不再调用它，上游 v27.2.1 同样保留该导出，维持对齐；删除属 C 之后的独立清理。
10. **merge_cursor_usage_events 未带 e4fa4ed 的早退分支**：上游 e4fa4ed 原版开头有 `if cached.is_empty() { return incoming; }`，本仓移植未带；缓存为空时走正常合并，拉回的数据里若有重复行会被合并掉（merge_on_an_empty_cache_is_the_incoming_history 测试锁定该行为）；但这个函数只在缓存非空时被调用，运行时无影响，且上游后来在 33b1723（归 C）里删掉了该分支。
11. **榜单共享缓存与转私密的失效窗口**：榜单页进共享缓存（s-maxage=30、stale-while-revalidate=60）后，privacy.ts 的 revalidatePath("/leaderboard") 不一定清掉 /zh-CN、/ja 前缀与带查询参数的副本，转私密的用户最长还会在榜上停留 90 秒；是否接受需 640 决定。

## 证据

全部命令在分支 sync/upstream-v27.2.1-pr-b 工作区真实执行（HEAD 353e3a6＋本任务未提交改动）。CLI 验证使用会话临时目录内的隔离工具链（PR A 架设的 rustup stable 与 make 4.4.1，RUSTUP_HOME／CARGO_HOME 指向该目录，不改 HOME、不动仓内配置）。

### 基线（改动前）

- 开工时工作区 node_modules 缺失：bun test 49 pass／6 fail（均为 Cannot find module），以 `bun install --frozen-lockfile` 恢复，lockfile 未变。
- 恢复后基线：`bun test` 100 pass／0 fail（13 文件）；`bunx tsc --noEmit` exit 0；`bun run lint` 0 errors；`cargo test` 2467 pass／0 fail／4 ignored。

### 改动后验证

- cursor.rs TDD：先落 e4fa4ed 的 13 条测试，编译失败 E0425（缺 merge_cursor_usage_events 等 6 个函数与 2 个常量）为红；再落实现，`cargo test -p tokens-cli` 228 pass／0 fail（215 基线＋13 新）。
- `cd web && bunx tsc --noEmit`：exit 0（全部改动后）。
- `cd web && bun run lint`：0 errors，2 warnings（base 353e3a6 只有 docs/page.tsx 的 img 1 条；本 PR 第一轮改动使 ProfilePageClient 一行 eslint-disable 变得多余，多出 1 条 unused-disable）。第四轮b 经审查确认该 react-hooks/set-state-in-effect 禁用行多余并已删除，lint 回到 0 errors、1 warning（只剩 docs/page.tsx 的 img），见「第四轮b 复跑」。
- `cd web && bun test`：105 pass／0 fail（13 文件）＝基线 100＋middleware 5 条新测试（榜单进共享缓存清单、前缀一致、白名单查询参数、杂散参数 307 规范化、teamboard 不在清单内）。middleware 测试为实现后补的行为刻画（移植性质），cursor 侧为真先红后绿，如实区分。
- `cd web && bun run build`：成功（全部页面生成，Proxy (Middleware) 正常）。
- `bunx next start` 生产构建 curl 实测：`/leaderboard` 200 且 `cache-control: public, max-age=0, s-maxage=30, stale-while-revalidate=60`；`/zh-CN/leaderboard` 同头且中文渲染；`/leaderboard?period=yesterday` 200；`/leaderboard?utm_source=x` 307 归一到 `/leaderboard`；`?sortBy=cost&page=2` 保持共享缓存头；`/teamboard` 仍 private no-store；`/settings` 307→`/login?returnTo=%2Fsettings`，`/zh-CN/settings` 307→`/login?returnTo=%2Fzh-CN%2Fsettings`（既有行为不变）；`/privacy`、`/zh-CN/privacy`、`/ja/privacy` 渲染全部 MCP 新条款（collect／never.5／public.p4／choices.stopMcp）；`/docs`、`/zh-CN/docs`、`/ja/docs` 渲染 privateMcp 段与四个 code 命令；榜单页 SSR 含「昨天／昨日／Yesterday」切换；`/api/leaderboard?period=yesterday` 在无 DATABASE_URL 环境 500，与 today／all 行为一致（缺库所致，非回归）。
- `cargo test --manifest-path cli/Cargo.toml`：2480 pass／0 fail／4 ignored（2467 基线＋13 新增量同步测试）。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`：exit 0。既有 cargo 级清单 warning（tokens-cli 未使用 csv 依赖）在 HEAD 同样存在，不属本 diff；33b1723（C）落地时自然消解，此处不夹带删除。
- `cargo build --release --manifest-path cli/Cargo.toml -p tokens-cli`：exit 0。产物冒烟：`tokens --version` 输出 1.0.4；`--lang zh-CN cursor sync --help` 中文、`--lang ja submit --help` 日文正常。
- Playwright e2e：未运行——需数据库与 fixture 环境，CI 也不跑此套件；本 PR 未改 e2e 规格文件，榜单行为变化（整榜下发、昨日）对应的 e2e 断言由 C 之后的维护窗口评估，如实标注不冒充通过。

### 第二轮复跑（2026-11-14 09:33–10:05 +08:00，同分支同工作区，结论与第一轮记录一致）

- `cd web && bunx tsc --noEmit`：exit 0。
- `cd web && bun run lint`：0 errors、2 warnings（docs/page.tsx 既有 img warning＋ProfilePageClient 的 unused-disable，与第一轮记录一致；该 unused-disable 为本 PR 第一轮改动致该禁用行多余，非 base 既有）。第四轮b 已删除该多余禁用行，当前 lint 为 0 errors、1 warning（只剩 docs/page.tsx 的 img），见「第四轮b 复跑」。
- `cd web && bun test`：105 pass／0 fail（13 文件）。
- `cd web && bun run build`：exit 0，全部页面生成，Proxy (Middleware) 正常。
- `cargo test --manifest-path cli/Cargo.toml`（隔离工具链）：2480 pass／0 fail／4 ignored；其中 tokens-cli 228 pass，含 13 条新增量同步测试。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`：exit 0；tokens-cli 清单级 csv 未使用依赖 warning 为 HEAD 既有，与本 diff 无关。
- `cargo build --release --manifest-path cli/Cargo.toml -p tokens-cli`：exit 0。产物冒烟：`tokens --version` 输出 1.0.4；`--lang zh-CN cursor sync --help` 中文、`--lang ja submit --help` 日文正常。
- 生产构建 `bunx next start`（端口 3107）curl 冒烟 18 项全过：`/leaderboard` 与 `/zh-CN/leaderboard` 200 且 `cache-control: public, max-age=0, s-maxage=30, stale-while-revalidate=60`；`/leaderboard?period=yesterday` 200；`/leaderboard?utm_source=x` 307 归一到 `/leaderboard`；`?sortBy=cost&page=2` 保持共享缓存头；`/teamboard` 仍 private no-store；`/settings` 307→`/login?returnTo=%2Fsettings`、`/zh-CN/settings` 307→`/login?returnTo=%2Fzh-CN%2Fsettings`；三语 /privacy 均渲染 MCP 条款（各 43 处）、三语 /docs 均渲染 privateMcp 段（各 35 处）；榜单 SSR 含 Yesterday／昨天／昨日切换；`/api/leaderboard?period=yesterday` 在无 DATABASE_URL 环境 500，与 today／all 行为一致（缺库所致，非回归）。冒烟结束已关闭该进程，无遗留。

### 第四轮b 复跑（2026-11-14 10:42–10:55 +08:00，同分支同工作区，HEAD 892b765＋本轮未提交改动）

本轮代码改动：SettingsClient 空状态图标按 d6aca2c 改 flex 居中、ProfilePageClient 删除多余禁用行、leaderboard page.tsx 注释改正；其余为 task.md 文字。CLI 代码未动。

- `cd web && bunx tsc --noEmit`：exit 0。
- `cd web && bun run lint`：0 errors、1 warning（docs/page.tsx 既有 img），多余禁用行删除后 unused-disable 消失，与审查预期一致。
- `cd web && bun test`：105 pass／0 fail（13 文件）。
- `cd web && bun run build`：exit 0，全部页面生成，/device 实测为 ƒ（Dynamic）server-rendered on demand（generateMetadata 经 getTranslator 读 headers），Proxy (Middleware) 正常。
- `cargo test -p tokens-cli --manifest-path cli/Cargo.toml`（隔离工具链，rustc 1.100.0）：228 pass／0 fail；CLI 本轮无代码变化，跑一次确认。

## 门禁结论

### Legacy Change Safety Review

- Status: characterized；Review mode: normal。
- 变更行为：cursor sync 由全量拉取改为水位线增量＋按事件标识合并；榜单由服务端分页＋会话排名改为整榜下发＋客户端定位 viewer；榜单数据库不可用时由流式 200 空榜改为失败前置成 500；资料页顶卡由「图选日」改为固定今日并可切昨日；/api/leaderboard 与页面接受 yesterday 周期。
- 保留行为：冷缓存仍全量拉取；空增量不动缓存；零事件保护（已有缓存时拉到空结果不覆盖缓存、记录错误）；teams/group 列与 MembershipCells；publicUserCondition 私密过滤；/api/leaderboard 保持 force-dynamic（本仓 b39a875 隐私修复）；tt_session 其他用途不动；Teamboard 不出现昨日选项（其 API 无昨日，详见取舍说明）。
- 现状证据（改动前基线，本分支 HEAD 353e3a6）：`cd web && bun test` 100 pass／0 fail（13 个文件）；`bunx tsc --noEmit` exit 0；`bun run lint` 0 errors、1 warning（base 353e3a6 只有 docs/page.tsx 的 img 一条；本 PR 第一轮改动使 ProfilePageClient 一行 eslint-disable 多余而多出 1 条 unused-disable，第四轮b 删除该禁用行后回到 img 1 条）；`cargo test --manifest-path cli/Cargo.toml` 2467 pass／0 fail／4 ignored。开工时工作区 node_modules 缺失导致 6 条测试报模块解析失败，以 `bun install --frozen-lockfile` 恢复后全绿，lockfile 未变。
- 安全网：上述既有全部测试＋e4fa4ed 自带 13 条增量同步测试（先红后绿）＋middleware 19 条＋coverage 2 条。
- 隐藏依赖／seam：Leaderboard 的 PERIODS／PERIOD_LABEL_KEYS／DeveloperRow／FormatToggle 被 Teamboard 复用；useSettings 的 setLeaderboardSort 会写 sort cookie；/api/auth/session 返回 { user }；无新增 seam。
- Validation plan：验收条件 5 的全量命令。

### DDIA Data Design Review

- Status: confirmed。
- Data owner and source of truth：cursor 用量缓存归 CLI 本机 cursor-cache/usage*.json，服务端无涉；榜单数据归 Postgres，读取经 getLeaderboardData 的 unstable_cache（tags：leaderboard、leaderboard:{period}、user-rank），本仓 privacy.ts 在隐私切换时 revalidateTag("leaderboard") 使数据层失效。
- Write／read／failure paths：cursor 写路径＝按水位线增量拉取→与缓存按事件标识并集合并（incoming 覆盖同标识行）→原子写；读路径＝submit/usage 解析既有缓存，JSON 形状不变（totalUsageEventsCount＋usageEventsDisplay）；失败路径＝缓存损坏视为无缓存转全量；空增量＝不动缓存；全量拉空且有缓存＝保留缓存并记录错误。榜单：整榜单次查询上限 5000 行截断保护；指令搜索（client:／model:）仍服务端过滤并参与缓存键；纯文本搜索、排序、分页在客户端数组上完成。
- Consistency model：合并对重复 sync 冪等收敛；同事件新快照覆盖旧快照；无 id 行按不可变投影键控，宁留重复不丢行（解析侧快照去重 d9a213a 归 C，见取舍说明）。
- Schema／migration／backfill／rollback：无 schema 与迁移；缓存文件格式前后兼容；回滚即还原代码。
- Observability and repair：零事件保护与错误列表维持原语义；榜单数据库故障在首字节前抛错，可被中间件层识别为 500。
- Required tests：e4fa4ed 自带 13 条（合并保留窗口外历史、快照替换、缓存重复坍缩、无 id 投影键、冷缓存、水位线兼容字符串与数字时间戳、忽略坏行、回退窗口与钟差、无缓存无水位线、坏 JSON 视为空、请求体仅增量携带 startDate、事件标识优先于投影键、空白 id 被忽略）。

### Refactoring Review

- Status: proceed；Review mode: normal；Decision: no refactor needed。
- 既有代码范围：cursor.rs、榜单五件套（page.tsx、Leaderboard.tsx、getLeaderboard.ts、searchDirectives.ts、route.ts）、ProfileToday.tsx、ProfilePageClient.tsx、middleware.ts、词条文件。
- 不变行为：见 Legacy 节保留行为。
- 结构判断：移植上游既定实现，不做先行重构；Leaderboard.tsx 以本仓文件为基叠加 36d17e1／ba3ad78 行为，保留本仓 MembershipCells／avatarUrlFor／FormatToggle 导出形状。
- 延期重构：无新增；PR A 记录的 lib/utils.ts 与 lib/format.ts 双格式化器维持延期。

### Ponytail-review（对本次 diff 的复杂度审查）

- 结论：Lean already。新增代码为上游既定实现的移植与本仓适配层（PrivacySection 确认弹窗复用既有 ModalShell 原语、Teamboard 一行过滤、middleware 清单条目），无投机抽象、无新增依赖、无死代码。两条记录：getUserRank 页面侧已无调用，因上游 v27.2.1 同样保留该导出而保留（对齐，见取舍 9）；tokens-cli 的 csv 依赖未使用 warning 为 HEAD 既有，归 C 的 33b1723 自然消解。上游镜像文件不评审删改。

### Code Readability Review

- 本仓手写改动（PrivacySection 弹窗接线、Teamboard 过滤注释、middleware 清单注释、ProfileToday 切换、ProfilePageClient 今日/昨日推导、词条文案）逐段复审通过：命名与既有文件风格一致，注释说明为什么而非做什么；移植段保留上游注释原貌。一处自审查修复：PrivacySection 弹窗标题 id 与错误提示 id 撞名，已拆为独立 useId。
- 一处编辑事故记录：getLeaderboard.ts 删除 matchesLeaderboardSearch 时模糊匹配误伤邻近注释，造成临时语法错误，tsc 当场暴露并已修复；最终 diff 逐 hunk 复审无残留。

### Release Readiness Review

- Status: blocked（本地必需验证全绿；CI、合并、部署不在本会话授权内，同 PR A 的处理）。
- Production path：/leaderboard 页面与其共享缓存头、/api/leaderboard（新增 yesterday）、资料页日卡、设置页 read token 撤销确认、CLI cursor sync 增量拉取。影响面：全部站点访客（榜单文档进入边缘缓存）与 cursor 用户。
- Failure modes/safeguards：数据库不可用时榜单页在首字节前失败成 500（不再流式 200 空榜，b3a0a60）；缺 DATABASE_URL 时页面回落空榜的既有行为保留；/api/leaderboard 保持 force-dynamic（取舍 5）；隐私切换经 privacy.ts revalidateTag("leaderboard") 使数据层失效；middleware 查询串白名单 307 规范化防缓存分裂；cursor 空增量不动缓存、零事件保护保留、冷缓存全量。
- Capacity/limits：整榜单次上限 5000 行截断保护；榜单文档随行数线性增长（上游实测 69B/行 gzip）；cursor 增量窗口 1 小时＋5 分钟钟差，请求体仅增量携带 startDate。
- Observability/runbook：上线后抽查 /leaderboard 与 /zh-CN/leaderboard 的 cache-control 头、杂散参数 307、?period=yesterday 渲染、cursor sync 耗时下降。
- Rollout/rollback：纯代码变更，回滚即还原；无 schema／数据迁移；CLI 缓存 JSON 形状前后兼容。
- Required validation：上文「改动后验证」全部通过；未运行：Playwright e2e（需数据库与 fixture，CI 亦不跑）、CI（本会话不 commit/push）、真实部署。残余风险：取舍 2 的 id-less 快照并存窗口、取舍 3 的文案先行窗口，由编排方在 PR 流程处置（Reviewy 评审＋CI），本文不记录任何豁免。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-14T07:01:22.895609+08:00 | planned | in-progress | 开工：范围、验收条件与 Book Gate Plan 已确认，调查与归类完成 | task.md 已确认范围与授权节及上游提交归类表；分支 sync/upstream-v27.2.1-pr-b，开工前 git status 实测干净 |
| 2026-11-14T09:23:04.210809+08:00 | in-progress | checking | 实现与移植完成（cursor 增量同步、整榜下发、流式修复、昨日卡片、MCP 词条、设置页取舍、CLI 词条 15 条），进入收尾核对 | web: tsc exit 0、lint 0 errors（2 条 warning：base 既有 docs/page.tsx img＋本 PR 第一轮改动致多余的 ProfilePageClient 禁用行）、bun test 105 pass/0 fail、next build 成功、next start curl 冒烟全过；CLI: cargo test 2480 pass/0 fail/4 ignored、clippy -D warnings exit 0、release build exit 0、三语 help 实测正常 |
| 2026-11-14T09:29:03.797052+08:00 | checking | done | 验收条件 1-7 全部满足：增量同步与 13 条测试落地、整榜下发/流式修复/昨日卡片移植完成、MCP 词条三语落地并渲染、设置页评估与取舍记录完成、149 个提交归类齐备、验证命令全绿、禁用词与本机路径扫描零命中 | task.md 证据节与取舍说明；web: tsc exit 0、lint 0 errors（2 条 warning：base 既有 docs/page.tsx img＋本 PR 第一轮改动致多余的 ProfilePageClient 禁用行）、bun test 105 pass/0 fail、next build 成功、生产构建 curl 冒烟全过；CLI: cargo test 2480 pass/0 fail/4 ignored、clippy -D warnings exit 0、release build exit 0；release-readiness 记 blocked：CI/合并/部署不在本会话授权内 |
| 2026-11-14T09:33:15.000000+08:00 | done | planned | 重开：第一轮会话在最终验证阶段触上限被终止，09:29 的 done 所依据的验证证据未经实跑确认；保留该行与证据节作为旧完成证据，completed_at 先随重开失效、本轮重新完成后写入新值，全部验证由第二轮实跑 | 本轮开工说明；git status 与第一轮记录一致（39 个修改、1 个新增，均未提交） |
| 2026-11-14T09:33:16.000000+08:00 | planned | in-progress | 恢复执行：环境核对完成（node_modules 与 PR A 隔离工具链在位、无遗留后台进程），开始重跑验收条件 5 的全部命令 | 第二轮进度日志 09:33 条目 |
| 2026-11-14T10:03:16.000000+08:00 | in-progress | checking | 七条验证命令与生产 curl 冒烟全部实跑通过，数字与第一轮记录一致 | web: tsc exit 0、lint 0 errors（2 条 warning：base 既有 docs/page.tsx img＋第一轮改动致多余的 ProfilePageClient 禁用行）、bun test 105 pass/0 fail、build exit 0；CLI: cargo test 2480 pass/0 fail/4 ignored、clippy -D warnings exit 0、release build exit 0、产物三语冒烟正常；next start 冒烟 18 项全过且进程已关闭 |
| 2026-11-14T10:05:04.229193+08:00 | checking | done | 验收条件 1-7 经第二轮实跑确认全部满足；本轮进度日志、证据节复跑记录与模型表两轮计数补齐，禁用词与本机路径扫描零命中 | task.md 第二轮进度日志、证据节「第二轮复跑」与模型表；grep -nE 禁用词与本机路径扫描 exit 1 零命中 |
| 2026-11-14T10:52:30.000000+08:00 | done | done | 更正：按审查意见修正——d6aca2c 改归本 PR（设置空状态图标改 flex 居中，原 mx-auto 写法因 svg 块级实际偏左）、4c98999 改归 C 并改正理由（syncGitHubSocialLinks 不看 complete 写库，限流会清空社交链接）、删除 ProfilePageClient 多余 eslint-disable（lint 回 0 errors、1 warning）、归类统计改 A 9／B 8／C 20／不适用 112、e4fa4ed 测试数改 13、b624e57 理由改为 generateMetadata 读 headers 成动态页、取舍说明补早退分支与榜单 90 秒窗口与 v1.0.5 发布依赖三条、leaderboard 注释改正为无 deploy layer 组件的本仓实际；状态保持 done，不 commit、不 push | 证据节「第四轮b 复跑」：tsc exit 0、lint 0 errors 1 warning（docs/page.tsx img）、bun test 105 pass/0 fail、build exit 0（/device 为 ƒ dynamic）、cargo test -p tokens-cli 228 pass/0 fail（隔离工具链 rustc 1.100.0） |
| 2026-11-14T11:12:34.000000+08:00 | done | done | 更正：按复审意见修正 task.md 文字与数字——不适用小类改合并提交 55／上游版本线 12／本仓已落地 26／结构或体系不同 19（合计 112）、模型表第四轮改 12/17/2 与第四轮b 改 32/61/2 终值并补第五轮截至写入时计数、lint 基线说法改正（base 353e3a6 只有 docs/page.tsx 的 img 1 条 warning，本 PR 第一轮改动致 ProfilePageClient 一行 eslint-disable 多余多 1 条，第四轮b 已删，删去「首次输出不全」「HEAD 原样重跑同为 2 条」说法）、第四轮b 复跑时段改 10:42–10:55 +08:00、取舍 10 空缓存措辞严谨化；只改本文件，不改代码，不 commit、不 push，状态保持 done | 本轮 task.md 改动；grep -nE 禁用词与本机路径扫描零命中 |
| 2026-11-14T11:28:42.000000+08:00 | done | done | 更正：统一工具报错口径——工具报错 = 会话文件中标记为报错的工具结果条数（含命令非零退出），第一至五轮工具报错按会话结束后会话文件统计的终值改正为第一轮 9、第二轮 0、第三轮 1、第四轮 2、第四轮b 2、第五轮 0；第五轮回复与工具调用改终值 7/18；模型表补第六轮截至写入时计数（回复 4、工具调用 11、工具报错 0，截至 2026-11-14 11:28 +08:00）；11:12 进度条目与本表 11:12:34 行内旧数（12/17/1、32/61/0）同步改正；10:08 进度条目保留原样；只改本文件，不改代码，不 commit、不 push，状态保持 done | 本轮 task.md 改动；grep -nE 禁用词与本机路径扫描零命中 |
