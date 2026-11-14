---
schema_version: 1
id: 20261114-v2721-pr-c-remaining
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-11-13 10:32 群内选定多语言方案甲、10:33 确认 A/B/C 拆法；2026-11-14 01:50 用户在群内开启全自动模式直到
  v1.0.5 按模板发布且文档补全，Lord 记录；本会话明确选择 strict。
status: "done"
branch: sync/upstream-v27.2.1-pr-c
created_at: '2026-11-14T13:08:12.000000+08:00'
updated_at: "2026-11-14T16:18:13.000000+08:00"
completed_at: "2026-11-14T15:24:28.000000+08:00"
---
# 同步上游 v27.2.1 拆分 PR C — 其余提交（解析器、cursor 登录计费、MCP opt-in、Windows autosubmit、OG 重做、缓存与 publish 修复）

## 目标

把 PR B task.md「上游提交归类」表标记为「留给 C」的 20 个上游提交逐个带进本仓（或在「上游提交去向」写明不带的原因与改归），外加 42e63c2 在 B 未取的功能部分（CLI 开关、tokens-core/mcp.rs、api/submit、validation 与剩余 CLI 词条）：

- 4c98999：GitHub 限流不清空社交链接——本仓资料页路径 `getGitHubSocialLinks → syncGitHubSocialLinks` 不看 complete 就写库，会丢用户数据；移植并补测试；
- 33b1723：cursor desktop 登录与统一计费（含上游删除 `merge_cursor_usage_events` 的 `cached.is_empty` 早退分支）；
- 7b83ac0：OG 渲染器重做，og 词条三语（en、zh-CN、ja）随 PR A 登记一并接回；
- 42e63c2 功能部分：MCP 名称默认不上报（`tokens mcp-names` 命令、settings.json `shareMcpServerNames`、`submit --include-mcp`、服务端 `mcpServersOptIn` 门控、`submit --dry-run --json` 打印真实载荷），使 CLI 默认行为与已上线的隐私页、docs「默认关闭」文案一致；CLI 三语补齐 8 条 .about/.help 新键（cli.mcp-names.* 5 条、cli.submit.include_mcp.help、cli.submit.json.help、cli.cursor.login.source.help）与 3 条既有词条值修订（cli.cursor.login.about、cli.cursor.sync.about、cli.submit.dryRun）；
- 解析器：cba9c22（Devin 按 request_id 去重＋Codex .jsonl.zst）、38b0a15（Pi/DSH 少计）、f367951＋6554b27（codex compaction）、d9a213a（cursor 快照去重）、e2fd454（DSH 负值归一）、1ef5a3c（cursor 测试缓存目录按平台）；
- Windows autosubmit：79feedb、583c5b5、33ee973（含 tokensw 无窗启动器与 publish 打包）；
- 缓存与发布：7f30fd6（数据库抖动不缓存 404）、8cf8f8e（npm OIDC trusted publishing）、8415a71（tag 固定在构建提交）。

保持本仓差异：隐私模式、无 shame 页、无 GitHub 登录与 verified 徽章、/settings 未登录跳 /login、品牌色、邮箱登录、teams/teamboard、SITE_URL 派生用户可见 URL。新增与改动文案 en、zh-CN、ja 三语齐备。榜单转私密共享缓存清除：2026-11-14 13:59–14:12 群成员投票（甲），Lord 14:12 记录，第三轮落地——统一标签 leaderboard＋revalidateTag { expire: 0 } 清数据层三处 unstable_cache，对 en、zh-CN、ja 三个榜单地址逐个 revalidatePath，/api/me/private 与 /api/settings/privacy 两条转私密入口共用同一清除，页面层无 ISR 可清，新增 cache.test.ts 5 条（取舍说明 12）。

## 已确认范围与授权

- 用户 2026-11-13 10:32 群内选定多语言方案甲、10:33 确认 A/B/C 拆法；2026-11-14 01:50 用户在群内开启全自动模式直到 v1.0.5 按模板发布且文档补全，Lord 记录。
- 2026-11-14 13:59–14:12 群成员投票（甲），Lord 14:12 记录：榜单转私密共享缓存清除并进 PR C（第三轮落地），不再保留最长 90 秒的失效窗口。
- 本会话明确选择 strict 模式。不 commit、不 push，只改工作区文件。
- 起点：`main` a85690935f94d45f94b57873f15620c7a84a3d40（已含 PR A #20 与 PR B #22）；分支 `sync/upstream-v27.2.1-pr-c`，开工前 `git status` 实测干净，与 origin/main 一致。上游 `missuo/tokens` tag v27.2.1 = e39110ddc5ca9b2b8b25abcf57738b23931755d1，merge-base f114057。
- 范围锚点：`ai/tasks/20261114-v2721-pr-b-mixed-fixes/task.md`「上游提交归类」表 C 类 20 行＋42e63c2 行内注明归 C 的功能代码部分；OG 多语言部分按 PR A task.md 登记随 OG 重做一并处理。
- 未完整调用 grill-with-docs：范围、来源、验收与授权均已明确，需求无歧义，不重复访谈。

## 验收条件

1. 20 个 C 类提交逐个有去向：带进（列出文件）或写明不带的原因；42e63c2 功能部分落地后 CLI 默认不上报 MCP 名称，`tokens mcp-names status` 可读状态，`submit --dry-run --json` 打印的载荷与真实提交同源。
2. 4c98999 回归测试落地：GitHub 两路请求任一失败（限流/5xx/超时）时 `syncGitHubSocialLinks` 不写库、已有快照保留；404（账号不存在）仍视为可用的空快照（本仓既有语义）。
3. OG：/api/og 站点卡、页面卡、资料卡三类渲染；资料卡带 tokens/spend/rank/活跃天数/top clients/一年活跃条；`locale` 参数三语；`og` 词条三语注册进 messages 目录；品牌色与 SITE_HOST 用本仓值；next.config.ts 增加 htmlLimitedBots；layout、docs、teamboard、u 页 OG 图片地址改走 ogImageUrl（带 v 版本参数）。
4. 解析器行为按上游：Devin request_id 去重、Codex .jsonl.zst 扫描解码、Pi/DSH 计数修正、compaction 保留与遗漏计数、cursor 快照跨事件只计一次、DSH 负值归一；上游测试随代码一并移植并通过。
5. MCP opt-in：默认不扫描不上报；`shareMcpServerNames` 或 `--include-mcp` 才附带所选客户端的 MCP 名称；服务端只认 `mcpServersOptIn: true` 的名称，旧 CLI 的名称被丢弃并在下一次本地提交清空已存列表。
6. Windows autosubmit：tokensw 启动器、任务迁移与 schtasks UTF-16 解码按上游；publish-cli.yml 打包 tokensw；非 Windows 平台编译行为不变。
7. 验证全部真实执行并记录：`cd web && bunx tsc --noEmit`、`bun run lint`、`bun test`、`bun run build`；`cargo test --manifest-path cli/Cargo.toml`、`cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`、`cargo build --release --manifest-path cli/Cargo.toml -p tokens-cli`。本机预装 rustc 1.85.1 低于代码所需的 1.87，CLI 验证使用会话临时目录内的隔离工具链（PR A 架设、PR B 沿用的 rustup stable 1.100.0 与手工解包 make 4.4.1 仍在，不改 HOME、不动仓内配置）。lint 结果与 base a856909 对比，写清新增 warning 数。
8. 本文件模型表为真实计数；本文件通过禁用词与本机路径扫描，零命中。
9. 跑完验证后关掉本会话启动的 next start 等进程。
10. 第三轮（方案甲）：转私密时统一标签 LEADERBOARD_CACHE_TAG 以 { expire: 0 } 立即过期、三语榜单地址全部 revalidatePath；cache.test.ts 5 条覆盖两条转私密端点、helper 直调与参数变体挂标，TDD 先红（3 条失败、2 条通过）后绿；目标节与取舍说明 7 写为落地后的表述，不留与现状矛盾旧文。

## 模型与调用

主 agent 用 k3:max，密钥从环境变量 KIMI_API_KEY 读取。

| 角色 | 模型 | 密钥来源 | 调用次数 |
|---|---|---|---|
| 主 agent | k3:max | 环境变量 KIMI_API_KEY | 第一轮：回复 71、工具调用 90、工具报错 4（会话 01a1d839 因主机暂停被看门狗结束，代码零改动） |
| 主 agent | k3:max | 环境变量 KIMI_API_KEY | 第二轮：回复 136、工具调用 168、工具报错 3 |
| 主 agent | k3:max | 环境变量 KIMI_API_KEY | 第三轮：回复 96、工具调用 126、工具报错 3 |

统计口径：工具报错 = 会话文件中标记为报错的工具结果条数（含命令非零退出）。每轮一行，取该轮会话文件的最终计数；第四轮起只改记录，不进本表计数。

未启动子代理；密钥不写入本文件与任何产物。

## Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill；MCP 名称、cursor 登录来源、社交链接快照等术语在 PR A/B 与本仓代码中已定型，无模型歧义 | 需求 | not-required |
| book-ddia-data-design | cursor 本地缓存与统一计费合并、submission 解析版本帽与 mcpServersOptIn 服务端语义、OG 一年 immutable 缓存键、社交链接快照写库条件 | 设计稳定前 | passed（confirmed；结论见门禁结论节） |
| book-legacy-change-safety | 改变解析器、cursor 同步、社交链接刷新、资料页 404 语义的既有行为 | 首次行为修改前 | passed（characterized；normal；结论见门禁结论节） |
| book-refactoring-pass | 修改既有生产代码（tokens-core 解析器、cursor.rs、web 页面与库） | 首次生产编辑前 | passed（proceed；normal；no refactor needed；结论见门禁结论节） |
| book-release-readiness | publish-cli.yml 发布流程、api/submit 与 api/og 行为、CLI 提交语义变更 | 验证通过后、收尾前 | blocked（本地门禁全绿；CI、合并、部署与 npm 侧 trusted publisher 登记不在本会话授权内；结论见门禁结论节） |

TDD：4c98999 先写失败测试再改实现；上游自带测试（解析器、cursor、billing、autosubmit、submission）随代码一并移植，能先红的先红。

## 门禁结论

### DDIA Data Design Review

- Status: confirmed
- Data owner and source of truth：CLI 本地缓存（tokens-core message_cache、cursor-cache）归 CLI 所有；服务端 submissions/users 表归 web api 所有；社交链接快照在 users 行，GitHub 为上游来源；OG 卡片是查询串的纯函数，无持久状态。
- Write / read / async / failure paths：cursor 事件按 identity 合并、跨事件快照由 dedupe_cursor_event_snapshots 只计一次；本地设备的 mcpServers 列表整列替换，仅当载荷带 mcpServersOptIn: true 才存名称（旧 CLI 不带信号 → 名称被丢弃，下次本地提交清空已存列表）；社交快照仅在 complete（本仓语义：404 视为可用的空快照）时写库，失败路径不写；资料页 5xx 在 unstable_cache 内抛出，不入缓存。
- Consistency model：best-effort 最终一致（60 秒 revalidate）；服务端门控确定性。
- Idempotency / ordering / retry / deduplication：devin 按 request_id 去重、cursor 跨端点去重与快照去重、codex compaction 保留与遗漏计数；解析版本帽（devin-cli=2）保证修正后的更低总数能越过同版本回归护栏替换旧计数。
- Schema / migration / backfill / rollback / replay：无 schema 变更；版本帽即数据兼容闸。
- Observability and repair：cron 路径 failed 计数转 500（本仓既有 strict 变体保留）；`submit --dry-run --json` 打印与真实提交同源的载荷供自查。
- Required tests：上游随代码移植的解析器/cursor/billing/autosubmit 单测＋submission.test.ts 10 条＋githubSocials.test.ts 6 条，全部实跑通过（cargo test 2533 pass、bun test 121 pass）。

### Legacy Change Safety Review

- Status: characterized
- Behavior to change：syncGitHubSocialLinks 非 strict 路径不看 complete 就写库（限流清空快照）；资料页把 5xx 折叠成 404 并被边缘缓存；解析器少计/重复计数；cursor 缺 desktop/CLI 登录来源与统一计费。
- Behavior to preserve：404 视为可用的空快照并照常写库（本仓既有语义）；strict cron 变体与 failed→500；隐私模式、无 shame 页、无 GitHub 登录、membership、locale 缓存键、品牌色与 SITE_URL 派生。
- Current reproduction evidence：4c98999 新测试先行实跑，4 条失败（双路 403 写库、半失败写库、传输失败写库，及一处测试自身 URL 归一化预期错误已修正）；PR B 记录的 base 基线（bun test 100 pass、tsc exit 0、lint 0 errors 1 warning、cargo test 2467 pass）。
- Safety net：上游随代码移植的单测＋githubSocials.test.ts 6 条＋i18n coverage 测试＋七条验证命令全绿。
- Hidden dependencies / seam：OG 路由的 tokens.ci 字面量与 BRAND 色值（已换 SITE_HOST 与 #4f8ff0）；无新 seam。
- Validation plan：focused 测试先红后绿，再跑项目全量验证（结果见证据节）。
- Review mode: normal

### Refactoring Review

- Status: proceed
- Review mode: normal
- Existing-code scope：tokens-core 解析器与 message_cache、tokens-cli cursor/main/settings/autosubmit、web 的 githubSocials/publicProfileData/u 页/layout/docs/teamboard/next.config。
- Behavior that must remain unchanged：见 Legacy 节 preserve 清单。
- Structural friction：无——目标文件与上游历史 blob 逐字节一致（第一轮核对，第二轮复核），整取上游版本即最小 diff；本仓差异以定点 edit 回贴。
- Decision and smallest safe step：no refactor needed；不夹带清理。
- Safety net and validation：同 Legacy 节。
- Deferred refactors：无。

### Release Readiness Review

- Status: blocked
- Production path and affected users / systems：api/submit（MCP 名称门控）、api/og（新缓存头与渲染）、api/cron/refresh-social-links（沿用 strict）、u 页 5xx 行为、publish-cli.yml 发布流。
- Failure modes and safeguards：GitHub 限流不再清空快照（有测试）；数据库抖动不再缓存成 404（5xx 抛出，缓存不存）；旧 CLI 的 MCP 名称被服务端丢弃（隐私修复的本意）；OG 渲染失败不影响页面（图片路由独立）。
- Capacity / backpressure / limits：OG 一年 immutable 缓存键带 OG_VERSION；cron 分批并发 4（既有）。
- Observability / alerts / runbook：cron failed→500 供调度器重试；publish workflow 自带 npm≥11.5.1 自检。
- Rollout / migration / rollback / cleanup：web 须先于 CLI 部署（上游 42e63c2 提交信息同样注明）；8cf8f8e 的 npm OIDC 需要维护者在 npmjs.com 为每个包登记本仓＋workflow 文件名为 trusted publisher，未登记前 publish 步会失败（取舍说明 10）。
- Required validation and result：七条验证命令与生产冒烟全部实跑通过（证据节）。
- Optional checks, accountable owner acceptance, and residual risk：CI 运行、合并、部署与 npm 侧登记不在本会话授权内；Windows autosubmit 的实机行为未在本仓 Linux 环境验证，靠上游测试与 cfg 门控保障（取舍说明 11）。

### 第三轮门禁补评（榜单转私密共享缓存清除）

- Refactoring（proceed）：新建 cache.ts 为单一职责模块；privacy.ts 与 getLeaderboard.ts 只换常量引用与调用点，无结构性重构；安全网为 cache.test.ts 5 条与既有 121 条测试。
- Legacy Change Safety（characterized）：改变的行为——转私密后榜单三语与查询变体副本由「最长 90 秒」变为立即失效；保留的行为——提交刷新仍走 60 秒数据层 revalidate 与既有共享缓存头、embed/u 页逐用户 purge、teamboard 路径 purge、账号删除与删数据路径的 "max" 档次 purge。
- DDIA（confirmed）：标签分类不变（leaderboard 统一标签＋period/user-rank 窄标签），清除覆盖面从单一路径扩到三语路径；一致性模型不变（数据层 expire:0 立即过期，页面层按路径清除）。
- Release Readiness（维持 blocked）：运行时行为变化并入第二轮同节结论；残余边界见取舍说明 12 末段。

## 进度

- 2026-11-14 12:04:02 +08:00 主 agent（补记，按会话记录还原）：第一轮开始（首条用户消息），开始调查与血缘核对（PR B 归类表、上游 21 个提交 diff、本仓逐文件 blob 等价测试）。
- 2026-11-14 13:03:43 +08:00 主 agent（补记，按会话记录还原）：完成调查。结论：(1) tokens-core 与 tokens-cli 多数目标文件与上游某一历史 blob 完全一致（mcp.rs==53082d7、clients.rs==0532b9d、lib.rs==1216cdd、message_cache.rs==00579f7、scanner.rs==427b514、sessions/{devin,dsh,cursor}.rs==53082d7、sessions/pi.rs==b728ee2、sessions/utils.rs==42d7185、settings.rs/autosubmit.rs==59f31ae、Cargo.toml==53082d7、api/submit/route.ts==42d7185、publish-cli.yml==160c24d），匹配点之后仅剩 C 提交，可整取 e39110d 版本；(2) b4f6459 与 3db0468 的内容已在本仓（本仓 70b8245/dd2dfda 与上游同作者同 diff，PR B 归类表按标题把它们留在 C，本会话更正为「已在仓」）；(3) main.rs==cbf1ecd＋1 处本仓 help 文本、cursor.rs==cbf1ecd＋4 处小 delta（早退分支删除、注释措辞、2 处格式化），整取 e39110d 后本仓 delta 或被上游覆盖或等价保留，唯一需回贴的是 main.rs 的 login --private help 文本；(4) 本仓 4c98999 等价物只覆盖 cron strict 路径，资料页非 strict 路径仍中招，确认必须带；(5) codex.rs 与上游 00579f7 仅差 21 行 rustfmt 风格，整取上游版只损失该格式细节。逐条去向见「上游提交去向」。
- 2026-11-14 13:08:12 +08:00 主 agent：创建本文件与 index 行；基线环境核对——web/node_modules 缺失，已以 bun install --frozen-lockfile 恢复（658 包）；会话临时目录隔离工具链在位（rustc/cargo/clippy 1.100.0、make 4.4.1）；本机预装 rustc 1.85.1 不满足代码所需 1.87。
- 2026-11-14 13:58:23 +08:00 主 agent（补记，按会话记录还原）：第二轮开始（首条用户消息；会话 01a1d839 第一轮因主机暂停中断、代码零改动，本轮接续）。13:59:52–14:12:27 先复核第一轮 blob 等价结论：mcp/clients/lib/message_cache/scanner/sessions/settings/autosubmit/tokens-cli Cargo.toml/api submit 路由/publish-cli.yml 与所记上游历史 blob 逐一 diff 为 0，cursor.rs 四处 delta、main.rs 一处 help 文本 delta、codex.rs 21 行格式差均与第一轮记录一致；cli/Cargo.toml 与上游的差异仅为 workspace version（本仓 1.0.4 线，保留）。
- 2026-11-14 14:12:51–14:14:05 +08:00 主 agent（补记，按会话记录还原）：完成 CLI 移植。14:12:51 整取 e39110d：tokens-core 11 文件与 tokens-cli 9 文件（含新增 mcp_names.rs、cursor/billing.rs、cursor/cli_auth.rs、cursor/desktop.rs、bin/tokensw.rs）；14:13:47 main.rs 回贴本仓 login --private help 文本；14:13:47–14:14:05 locales 三语 cli.json 取上游 8 条新键（cli.mcp-names.* 5 条、cli.submit.include_mcp.help、cli.submit.json.help、cli.cursor.login.source.help）与 3 条修订值（cli.cursor.login.about、cli.cursor.sync.about、cli.submit.dryRun），cli.login.private.help 保留本仓三语值（与 main.rs 回贴一致）。
- 2026-11-14 14:14:27–14:19:07 +08:00 主 agent（补记，按会话记录还原）：完成 web 移植。14:14:27 整取 api/submit/route.ts、validation/submission.ts、submission.test.ts、lib/og/ogImage.ts、lib/og/fonts.generated.ts、scripts/generate-og-fonts.ts、scripts/og-cjk-hanzi.txt、api/og/route.tsx 与 i18n messages og 三语；14:14:48–14:15:37 OG 路由适配——BRAND 与 HEAT 末档换 #4f8ff0、Background 径向渐变换 rgba(79,143,240,…)、三处 tokens.ci 字样换 SITE_HOST；14:15:49 next.config.ts 只加 htmlLimitedBots；14:16:04–14:16:21 layout 保留 SITE_URL metadataBase 与 favicon v3、OG 图改 ogImageUrl({locale})；14:16:39–14:17:52 u 页加 7f30fd6 的 5xx 抛出、ogImageUrl 全参数（avatar/rank/tokens/cost/days/clients/activity/locale）、规范 handle 与 twitter description（保留 profile.meta.* 键、SITE_URL/localizeHref、membership 与 locale 参数），publicProfileData 只在 cached 函数内加 5xx 抛出（保留 locale 缓存键与本仓访问检查次序）；14:18:17 docs 与 teamboard 页 OG 地址改走 ogImageUrl；14:18:34–14:19:07 og 三语注册进 messages/index.ts（保留本仓 auth/teams/teamboard 命名空间，不建 shame）。
- 2026-11-14 14:19:52–14:20:49 +08:00 主 agent（补记，按会话记录还原）：完成 4c98999 TDD。14:19:52 写 githubSocials.test.ts 6 条，14:20:07 实跑：4 条失败（双路 403 写库、半失败写库、传输失败写库，另 1 条为测试自身 URL 归一化预期错误，14:20:16–14:20:36 修正测试后仍 3 条红在写库断言）；14:20:45 把 syncGitHubSocialLinks 改为仅在 complete 时写库，strict 变体与本仓 404 语义不动；14:20:49 复跑 6 条全绿。
- 2026-11-14 14:20:54–14:21:45 +08:00 主 agent（补记，按会话记录还原）：完成 publish-cli.yml 移植：取 33ee973（matrix launcher 字段、Collect 步、ship/  artifact、双成员打包）、8cf8f8e（id-token: write、node-version 24、去 registry-url 与 NODE_AUTH_TOKEN、npm≥11.5.1 自检）、8415a71（gh release create --target 固定在构建提交）；actions 主版本（checkout v6、cache v5、setup-node v6）保持不动（取舍说明 6）。
- 2026-11-14 14:22:48–14:30:48 +08:00 主 agent（补记，按会话记录还原）：完成七条验证命令实跑与生产冒烟，全部通过，数字见证据节。cargo test 14:22:48 起后台跑；tsc 14:24:59 exit 0；bun test 14:25:25（121 pass）；lint 14:26:26，base a856909 临时 worktree 对比 14:26:54–14:27:46（同为 0 errors、1 warning，对比用 worktree 随即移除）；build 14:27:54–14:28:42 exit 0；next start 14:29:01 起，冒烟 14:29:31–14:30:34：/api/og 站点卡、页面卡（ja title）、资料卡（zh-CN，tokens/cost/rank/days/clients/activity 全参数）均 200 image/png 1200×630 且 Cache-Control 为一年 immutable，/、/zh-CN/leaderboard、/docs 头部 og:image 带 locale 与 v=2；14:30:48 关 next start。clippy 14:30:48–14:31:41 exit 0。
- 2026-11-14 14:33:24–14:35:07 +08:00 主 agent（补记，按会话记录还原）：14:33:24 趁 release 构建期间开始写 task.md；14:34:10 release 构建完成、产出 tokens 与 tokensw；14:34:10–14:34:47 CLI 冒烟：mcp-names enable/status/disable 闭环（默认关闭、status 可读、disable 提示下次提交清空）、submit --dry-run --json 走通扫描路径（隔离 HOME 无数据，止于空数据提示）、三语 help 新键渲染正常；14:35:02–14:35:07 复跑 cargo test 取总计数（2533 passed、0 failed、4 ignored）。
- 2026-11-14 14:37:21–14:46:56 +08:00 主 agent（补记，按会话记录还原）：第二轮收尾——14:37:21–14:39:24 写入四项门禁结论、进度与证据节；14:39:43 清理临时产物（便签 HOME、下载的 PNG 已删，基线对比用临时 worktree 已移除）；14:44:07–14:44:57 按会话记录统计第二轮最终计数；14:45:12–14:45:42 写入模型表、状态事件与 frontmatter；14:45:49 禁用词与本机路径扫描零命中；14:46:02 确认无遗留进程与杂散文件；14:46:56 交付。
- 2026-11-14 14:48:28 +08:00 主 agent（补记，按会话记录还原）：第三轮开始，处理榜单转私密缓存清除（方案甲）。
- 2026-11-14 15:03:57–15:04:35 +08:00 主 agent（补记，按会话记录还原）：完成第三轮调查。结论：(1) 榜单缓存共三层——getLeaderboard.ts 三处 unstable_cache 数据缓存（getPeriodRanking 的 period-ranking 键覆盖时间段/日期/排序变体，getLeaderboardData 的 leaderboard:* 键覆盖 all 时间段与 client:/model: 指令搜索的分页/排序/搜索/日期变体，getUserRank 的 user-rank 键覆盖 all 时间段个人名次；三处 tags 均含字面量 leaderboard，revalidate 60，键不含语言，三语共用一份数据）；页面层无 ISR（leaderboard 页读 searchParams 且 getTranslator 读 headers 为动态渲染，无 Full Route Cache；/api/leaderboard 为 force-dynamic）；middleware 给 /leaderboard 及 /zh-CN、/ja 前缀与 period/from/to/sortBy/page/search 查询变体盖 `public, max-age=0, s-maxage=30, stale-while-revalidate=60` 共享缓存头（90 秒窗口所在，边缘按完整 URL 建副本）。(2) teamboard 的三处 unstable_cache 同样只挂 leaderboard 标签，统一标签清除一并覆盖其数据层。(3) 转私密路径恰两条——/api/me/private POST（tokens login --private，只设私密）与 /api/settings/privacy PATCH（双向），均汇于 privacy.ts 的 revalidateUserVisibility。(4) 现状缺口与 PR B 取舍 11 记录一致：revalidateUserVisibility 只 revalidatePath("/leaderboard")，覆盖不到 /zh-CN/leaderboard、/ja/leaderboard 两个语言前缀副本。(5) 榜单查询经 publicUserCondition() 排除私密与封禁账号，清除后新渲染即不含该用户。
- 2026-11-14 15:04:42–15:06:31 +08:00 主 agent（补记，按会话记录还原）：重开任务——状态 done→planned→in-progress，旧完成证据保留在状态事件表，completed_at 清空待第三轮验收后重记；目标节、已确认范围与授权第二条与取舍说明同步改写为方案甲表述（授权见「已确认范围与授权」第二条）。
- 2026-11-14 15:08:02 +08:00 主 agent（补记，按会话记录还原）：建 lib/leaderboard/cache.ts——集中统一标签 LEADERBOARD_CACHE_TAG（值仍为 leaderboard）与 revalidateLeaderboard()。
- 2026-11-14 15:08:51–15:09:59 +08:00 主 agent（补记，按会话记录还原）：写 cache.test.ts 5 条并跑红。15:08:51 成文，15:09:13 调整 mock；15:09:26 首跑因测试自身 mock 缺 READ_TOKEN_PREFIX 导出而未执行（1 error），15:09:55 补齐 mock 导出；15:09:59 跑红：5 条里 3 条失败——PATCH /api/settings/privacy、POST /api/me/private、revalidateUserVisibility 直调均缺 /zh-CN/leaderboard 与 /ja/leaderboard 的 revalidatePath（统一标签清除断言本已成立），2 条通过（10 个参数变体条目全挂统一标签、helper 自身语义）。
- 2026-11-14 15:10:17–15:10:44 +08:00 主 agent（补记，按会话记录还原）：落地实现并转绿。15:10:17–15:10:39 privacy.ts 的 revalidateUserVisibility 改走 revalidateLeaderboard()、getLeaderboard.ts 三处 tags 改引用常量；15:10:44 复跑 5 条全绿。
- 2026-11-14 15:11:11–15:15:51 +08:00 主 agent（补记，按会话记录还原）：完成第三轮七条验证与冒烟，全部通过，数字见证据节。tsc 15:11:11 exit 0；lint 15:11:31，base a856909 临时 worktree 对比 15:11:58–15:13:11（同为 0 errors、1 warning，对比用 worktree 随后移除）；bun test 15:13:27（126 pass）；build 15:13:41 exit 0；cargo test、clippy、release build 15:14:03–15:15:01 全绿。冒烟 15:15:28–15:15:40（next start 端口 3103）实测 /leaderboard、/zh-CN/leaderboard、/ja/leaderboard 及 period/sortBy/page/search 与 custom 日期查询变体均 200 且共享缓存头在位（即本轮清除所命名的副本形态）；本地无 DATABASE_URL，页面走空板兜底路径，清除行为由 cache.test.ts 覆盖；15:15:46–15:15:51 next 进程全部关闭。
- 2026-11-14 15:16:22–15:26:06 +08:00 主 agent（补记，按会话记录还原）：第三轮收尾——15:21:55–15:22:38 写入验收条件 10、取舍说明 7 与 12、进度与证据节、第三轮门禁补评；15:22:49–15:24:28 按会话记录统计第三轮最终计数；15:25:21 写入模型表、frontmatter 与状态事件；15:25:37 禁用词与本机路径扫描零命中；15:25:43 确认无遗留 next 进程；15:26:06 交付。
- 2026-11-14 15:42:07 +08:00 主 agent：第四轮只改记录：按 Reviewy 对 12d7cd89 的 NOT CLEAN 意见修订本文件——90 秒缓存窗三处表述统一为「2026-11-14 13:59–14:12 群成员投票（甲），Lord 14:12 记录」并直接写落地后现状；进度日志各条时间按会话文件逐条还原，事后补写的条目加标「补记，按会话记录还原」；created_at 改为第一轮实际建文时刻 13:08:12；模型表改为每轮一行最终计数并统一工具报错口径；取舍说明 10 补 npm 包名与 repository.url 现状。不改代码，不跑构建，不提交不推送。
- 2026-11-14 16:18:13 +08:00 主 agent：第六轮只改记录：按 Reviewy 15:41 对 b9b15293 的剩余意见修订本文件（第五轮 453f9ebf 已改一部分）——投票表述全文统一为「2026-11-14 13:59–14:12 群成员投票（甲），Lord 14:12 记录」（目标节、授权第二条、取舍说明 7、第四轮条目引文、状态事件表两处）；进度节各条时间按三轮会话文件再逐条核对改正：第一轮开始改 12:04:02（首条用户消息）、第二轮开始改 13:58:23，第二轮各条改为区间（CLI 移植 14:12:51–14:14:05 含 main.rs 回贴 14:13:47、web 移植 14:14:27–14:19:07、4c98999 测试 14:19:52–14:20:49、publish-cli.yml 14:20:54–14:21:45、14:30:48 关 next start、14:33:24 开始写本文件），并补第二轮收尾条（14:37:21–14:46:56）；第三轮拆出「建 cache.ts（15:08:02）」「写测试并跑红（15:08:51–15:09:59）」「落地实现并转绿（15:10:17–15:10:44）」三条并补第三轮收尾条（15:16:22–15:26:06），全节按时间升序；第三轮 TDD 表述统一为实情 5 条里 3 条先红、2 条先绿（验收条件 10、取舍说明 7、状态事件表两处）；模型表维持三轮最终计数每轮一行，口径注脚改为「第四轮起只改记录」；取舍说明编号 1–12 连续、文中引用核对一致。不改代码，不跑构建，不提交不推送。

## 上游提交去向（f114057..e39110d 中 PR B 表 C 类 20 个＋42e63c2 功能部分）

| 提交 | 去向 | 文件与说明 |
|---|---|---|
| 42e63c2 fix(privacy): MCP server names opt-in（功能部分） | 带进 | mcp_names.rs（新）、commands/mod.rs、settings.rs、tokens-core/mcp.rs、main.rs（McpNames 命令、--include-mcp/--json、SubmitPreview、resolve_submit_mcp_servers、统一载荷构建）、api/submit/route.ts（optedInMcpServers 门控）、submission.ts/test（mcpServersOptIn schema＋optedInMcpServers＋6 条测试）、CLI 三语 8 新键＋3 修订值；docs/privacy 页面与词条 PR B 已按本仓结构落地，README 段不带（README 本仓自有） |
| cba9c22 fix(parsers): Devin 去重＋Codex .jsonl.zst | 带进 | sessions/devin.rs、sessions/codex.rs、sessions/utils.rs、scanner.rs、message_cache.rs、lib.rs、clients.rs、main.rs（submit_parser_revision 改 match，devin-cli=2）、submission.ts/test（devin-cli 版本帽 2 与 per-client 上限表） |
| 38b0a15 fix(parsers): Pi/DSH 少计 | 带进 | sessions/pi.rs（entry_identity_first）、sessions/dsh.rs（流式读取重做）、message_cache.rs、lib.rs |
| 79feedb style(cli): schtasks UTF-16 as_chunks | 带进 | commands/autosubmit.rs |
| 583c5b5 fix(cli): Windows 任务迁移无窗启动器 | 带进 | commands/autosubmit.rs、bin/tokensw.rs（新）、main.rs、Cargo.toml；README 段不带 |
| 1ef5a3c test(cursor): 解析测试缓存目录按平台 | 带进 | tokens-core lib.rs 测试辅助 |
| d1cc44c fix(cursor): 本地登录门控与跨端点去重 | 带进 | cursor.rs、cursor/billing.rs（新）、cursor/cli_auth.rs（新）、main.rs；README 段不带 |
| 3e3a5ab feat(cursor): CLI 账号登录发现 | 带进 | cursor.rs、cursor/cli_auth.rs、main.rs、clients.rs；README 段不带 |
| 33b1723 fix(cursor): desktop 登录与统一计费 | 带进 | cursor.rs、cursor/billing.rs、cursor/desktop.rs（新，macOS cfg）、main.rs、clients.rs；含上游对 merge_cursor_usage_events 早退分支的删除。csv 依赖核对：上游 e39110d 的 tokens-cli 仍含 csv = "1.3"，无任何 C 提交移除它，本仓与上游保持一致，PR B 进度条中「33b1723 自然消解 csv warning」的预判不实，如实更正 |
| f367951 fix(codex): compaction 跨元数据保留 | 带进 | sessions/codex.rs（pending_compaction 机制） |
| 6554b27 fix(codex): compaction 遗漏计数 | 带进 | sessions/codex.rs、message_cache.rs |
| d9a213a fix(cursor): 快照跨事件只计一次 | 带进 | sessions/cursor.rs、message_cache.rs（dedupe_cursor_event_snapshots；闭合 PR B 取舍 2 记录的风险窗口） |
| e2fd454 fix(core): DSH 负值归一 | 带进 | sessions/dsh.rs、message_cache.rs |
| 33ee973 fix(cli): Windows autosubmit 无控制台窗口 | 带进 | bin/tokensw.rs、commands/autosubmit.rs、Cargo.toml、publish-cli.yml（tokensw 打包与 collect 段）；README 段不带 |
| 7b83ac0 feat(og): 分享卡重做 | 带进并适配 | api/og/route.tsx（整取上游重做版，BRAND 换本仓 #4f8ff0、tokens.ci 字样换 SITE_HOST）、lib/og/ogImage.ts、lib/og/fonts.generated.ts（上游已生成的内嵌字体数据整取）、scripts/generate-og-fonts.ts、scripts/og-cjk-hanzi.txt、next.config.ts（htmlLimitedBots 段）、layout.tsx（ogImageUrl({locale})，保留 SITE_URL metadataBase 与 favicon v3）、u 页 generateMetadata（ogImageUrl 全参数＋canonical handle＋twitter description，保留本仓 profile.meta.* 键与 SITE_URL/localizeHref）、docs/page.tsx 与 teamboard/page.tsx 的 OG 地址改走 ogImageUrl；og 词条三语新建并注册；shame/page.tsx 段不带（本仓无 shame 页） |
| b4f6459 fix(codex): headless 重复计数 | 已在仓 | 本仓 70b8245 与上游同作者（Vincent Yang）同标题同 diff（main.rs、lib.rs、message_cache.rs、sessions/codex.rs、submission.ts 五件齐全，逐个 diff 核对一致）；PR B 归类表按标题把它留在 C，更正为「已在仓」，无需移植 |
| 3db0468 fix(codex): submit 解析版本升至 3 | 已在仓 | 本仓 dd2dfda 同理已在仓（main.rs submit_parser_revision codex=3 与 submission.ts 上限；后续 70b8245 已升到 4）；更正为「已在仓」 |
| 8cf8f8e ci(publish): npm OIDC trusted publishing | 带进 | publish-cli.yml：id-token: write、setup-node 去 registry-url、发布步去 NODE_AUTH_TOKEN、npm≥11.5.1 自检。npm 侧需把本仓＋该 workflow 登记为 trusted publisher，属部署前提，记入取舍说明 |
| 8415a71 fix(ci): tag 固定在构建提交 | 带进 | publish-cli.yml：gh release create 加 --target（版本提交） |
| 4c98999 fix(web): GitHub 限流不删徽章 | 带进并适配＋补测试 | githubSocials.ts：非 strict 的 syncGitHubSocialLinks 只在 complete 时写库（本仓已有 complete 信号与 strict 变体，资料页路径补上同一道闸）；refreshSocialLinks.ts 与 api/cron/refresh-social-links/route.ts 的本仓等价实现（strict＋failed 计数→500）保留，不换上上游 skipped 版；worker.ts 段不适用（本仓无 worker 层，2fc41cd 已移除） |
| 7f30fd6 fix(web): 数据库抖动不缓存 404 | 带进 | u/[username]/page.tsx（result.kind==="error" 且 status≥500 抛出）、publicProfileData.ts（cached 函数内 5xx 抛出）；本仓 locale 参数、membership、verified 移除等自有差异保留 |

## 取舍说明

1. **b4f6459/3db0468 实为已在仓**：PR B 归类表按标题把它们留给 C；本会话逐文件 diff 核对发现本仓 70b8245/dd2dfda（v1.0.x 同步路线）与上游两提交同作者同内容。C 实际移植 18 个提交＋42e63c2 功能部分，另 2 个核对后关闭。
2. **README 本仓自有**：583c5b5、d1cc44c、3e3a5ab、33b1723、33ee973、42e63c2 六个提交含 README 段；本仓 README 为自有文档（PR B 归类表已述），不带入；相关功能的用户可见说明由 docs 页与 CLI --help 承担。
3. **OG 品牌适配**：上游卡片 BRAND #2F6FDB 换本仓品牌色 #4f8ff0（本仓旧 OG 渲染器与站点 primary 同色），HEAT 热力末档与 Background 径向渐变同步换色；卡片上的 tokens.ci 字样换 SITE_HOST（本仓惯例：用户可见 URL 一律从部署 origin 派生，lib/site.ts）。OG_VERSION 随上游为 "2"。
4. **u 页元数据**：取 7b83ac0 的 ogImageUrl 全参数（avatar/rank/tokens/cost/days/clients/activity/locale）、canonical handle（卡与 og:url 跟随规范大小写）与 twitter description；shareTitle/shareDescription 保留本仓 profile.meta.* 键位（PR A 已修正上游 meta.* 指向错误），SITE_URL/localizeHref 保留。
5. **publicProfileData 只取 7f30fd6 抛出段**：verified 暴露属 bfc87bb（PR B 归类不适用：本仓已移除 GitHub verified 徽章体系）；16724c8 的访问检查次序与 visibilityHeaders 写法保留本仓侧（PR B 取舍 1 已决）；`NOT u.is_private` 与 `u.is_private = false` 等价，保留本仓写法。
6. **9ecae99 维持不适用**：publish-cli.yml 只取 33ee973/8cf8f8e/8415a71 三个 C 提交的 hunk；actions 主版本升级（checkout v7、cache v6、setup-node v7、node 24）不在 C 清单，本仓 ci.yml 同样未做该升级，保持一致。
7. **榜单转私密共享缓存清除（方案甲）**：2026-11-14 13:59–14:12 群成员投票（甲），Lord 14:12 记录；第三轮落地，PR B 取舍 11 挂起项就此关闭——统一标签 leaderboard＋revalidateTag { expire: 0 } 清数据层三处 unstable_cache，对 /leaderboard、/zh-CN/leaderboard、/ja/leaderboard 三个榜单地址逐个 revalidatePath，/api/me/private 与 /api/settings/privacy 两条转私密入口共用同一清除，页面层无 ISR 可清，cache.test.ts 5 条中 3 条先红、2 条先绿，实现后 5 条全绿；实现细节与残余边界见取舍说明 12。
8. **cursor.rs 整取 e39110d**：本仓相对 cbf1ecd 的 4 处 delta 中——早退分支删除上游 33b1723 同款、注释措辞上游版更准（dedupe_cursor_event_snapshots 现已存在）、2 处 rustfmt 风格差异无关紧要——无本仓独有内容损失。sessions/codex.rs 同理整取（仅损失 21 行格式化细节）。
9. **worker.ts 不适用**：4c98999 的 worker.ts hunk 属上游 Workers cron 入口；本仓 2fc41cd 已移除 worker 层，定时刷新走 api/cron 路由。
10. **8cf8f8e 的部署前提**：npm trusted publishing 要求 npmjs.com 上每个包把本仓＋workflow 文件名登记为 trusted publisher，且 package.json repository.url 指本仓；packages 下 npm 包的 repository.url 和包名仍是上游的（missuo/tokens、tokens-cli*），trusted publishing 在本仓跑不通，登记前需改包名和仓库地址。未登记前该 workflow 的 publish 步会失败。此为本仓维护者的 npm 侧操作，不在本会话授权内，如实记录。
11. **tokensw 与 Windows 段在非 Windows 平台**：tokensw.rs 全目标编译（非 Windows 为纯转发），autosubmit.rs 的迁移逻辑 cfg(windows) 门控、含 cfg_attr(not(windows), allow(dead_code)) 与跨平台单测；本仓 Linux 验证覆盖编译与单测，Windows 行为靠上游测试与代码审查保障。
12. **第三轮：榜单转私密共享缓存清除（方案甲）**：新建 lib/leaderboard/cache.ts 集中统一标签 LEADERBOARD_CACHE_TAG（值仍为 "leaderboard"，submit、teams/service、settings 各既有 purge 点不受影响）与 revalidateLeaderboard()；getLeaderboard.ts 三处 unstable_cache 的 tags 改引用该常量。清除语义：revalidateTag(tag, { expire: 0 }) 立即过期全部数据层条目——缓存键不含语言（译文在渲染层产生），三语共用一份数据，一次标签清除覆盖三语数据；时间段/日期/排序/分页/搜索各变体条目同挂该标签一并失效；teamboard 三处聚合也只挂该标签，随同清除。页面层：leaderboard 页为动态渲染，无 ISR 可清；边缘共享缓存按完整 URL 存副本，逐个 revalidatePath 三个语言榜单地址（/leaderboard、/zh-CN/leaderboard、/ja/leaderboard，由 LOCALES 派生，新增语言自动纳入），每个地址的清除覆盖其全部查询变体。privacy.ts 的 revalidateUserVisibility 改走该 helper，/api/me/private 与 /api/settings/privacy 两条转私密路径共用同一清除。共享缓存头（s-maxage=30、stale-while-revalidate=60）本身不动：提交数字的短时延迟是 middleware 注释明记的既定设计，本轮只让转私密立即生效。账号删除与删数据路径沿用各自既有 "max" 档次 purge，语义不借本轮改变。残余边界：若部署在平台之外再架一层不理会 revalidatePath 的共享缓存，该层副本仍受其缓存头时限（最长 90 秒），仓库代码够不到该层；embed svg 路由的 private,no-store 注释已记录同类边界。

## 证据

第二轮实跑（全部在分支 sync/upstream-v27.2.1-pr-c 工作区，未提交）：

- `cd web && bunx tsc --noEmit`：exit 0，无输出。
- `cd web && bun run lint`：0 errors、1 warning（docs/page.tsx 的 `@next/next/no-img-element` img 提示）。base 对比：在 a856909 的临时 git worktree 上以同一依赖树实跑 `bun run lint`，同为 0 errors、1 warning（同一文件同一规则）。新增 warning 数：0。对比用 worktree 已移除。
- `cd web && bun test`：121 pass、0 fail、5438 expect()，15 个测试文件——含本轮新增 githubSocials.test.ts 6 条与移植的 submission.test.ts 10 条（单跑复核：submission 10 pass/0 fail）；i18n coverage 测试继续通过。
- `cd web && bun run build`：exit 0（✓ Compiled successfully、42/42 静态页生成）。
- `cargo test --manifest-path cli/Cargo.toml`（隔离工具链 rustup stable 1.100.0＋手工解包 make 4.4.1，不改 HOME、不动仓内配置）：2533 passed、0 failed、4 ignored（按各套件 test result 行求和；PR B 记录 PR B 末态为 2480，C 的移植测试净增 53）。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`：exit 0；仅 1 条 cargo manifest warning（tokens-cli 的 csv 未用依赖）——base 同款依赖同款提示，上游 e39110d 仍保留 csv = "1.3"，本仓与上游保持一致（去向表 33b1723 行已更正 PR B 的预判）。
- `cargo build --release --manifest-path cli/Cargo.toml -p tokens-cli`：exit 0，产出 tokens 与 tokensw 两个二进制（tokensw 在非 Windows 为纯转发，编译行为不变）。
- 生产冒烟（`next start` 于本会话端口 3102，验毕已关闭）：`/api/og`、`/api/og?locale=zh-CN`、`/api/og?locale=ja&title=…`、`/api/og?handle=…&tokens=…&cost=…&rank=…&days=…&clients=…&activity=…&locale=zh-CN` 四路均 HTTP 200、content-type image/png、PNG 头 1200×630、Cache-Control `public, max-age=3600, s-maxage=31536000, stale-while-revalidate=604800, immutable`；`/`、`/zh-CN/leaderboard`、`/docs` 响应头部 og:image 分别为 `…/api/og?locale=en&v=2`、`…?locale=zh-CN&v=2`、`…?title=Docs&locale=en&v=2`（htmlLimitedBots 生效，元数据在 head）。
- CLI 冒烟（release 产物，隔离 HOME 便签目录）：`tokens mcp-names status` 默认 not shared、列出各客户端配置读取位置；`--json` 输出 enabled:false 与 locations；enable→status→disable 闭环文案正常；`tokens submit --dry-run --json` 走通扫描（无本地数据止于「No usage data found」，载荷内容断言由 cargo test 内上游移植测试覆盖）；三语 help 实测——en 的 mcp-names 条目与 --include-mcp/--json、zh-CN 的 mcp-names 子命令与 submit 选项、ja 的 cursor login --source 与修订后 about 均正常渲染。
- 4c98999 TDD 红绿记录：githubSocials.test.ts 先红（3 条写库断言失败＋1 条测试自身预期错误）→ 实现仅在 complete 时写库 → 6 条全绿。
- 冒烟与验证结束后：next start 已关闭，无本会话遗留服务进程；临时 worktree、便签 HOME 与下载的 PNG 产物均已清理。

第三轮实跑（分支 sync/upstream-v27.2.1-pr-c 工作区，未提交；CLI 用同一会话临时目录隔离工具链 rustup stable 1.100.0＋make 4.4.1，不改 HOME、不动仓内配置）：

- `cd web && bunx tsc --noEmit`：exit 0，无输出。
- `cd web && bun run lint`：0 errors、1 warning（docs/page.tsx 的 `@next/next/no-img-element` img 提示，340 行）。base 对比：在 a856909 的临时 git worktree 上以同一依赖树实跑 `bun run lint`，同为 0 errors、1 warning（同文件同规则，339 行；行号差为 PR C 已合入改动所致）。新增 warning 数：0。对比用 worktree 已移除。
- `cd web && bun test`：126 pass、0 fail、5465 expect()，16 个测试文件——含本轮新增 cache.test.ts 5 条（第二轮基线 121 → 126）；i18n coverage 测试继续通过。
- `cd web && bun run build`：exit 0，/leaderboard 维持 ƒ 动态渲染。
- `cargo test --manifest-path cli/Cargo.toml`：2533 passed、0 failed、4 ignored（CLI 本轮无改动，与第二轮持平）。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`：exit 0；仅 base 同款 tokens-cli csv 未用依赖 manifest warning。
- `cargo build --release --manifest-path cli/Cargo.toml -p tokens-cli`：exit 0，产出 tokens 与 tokensw 两个二进制。
- TDD 红绿记录：cache.test.ts 先红 3 条（PATCH /api/settings/privacy、POST /api/me/private、revalidateUserVisibility 直调均缺 /zh-CN/leaderboard 与 /ja/leaderboard 的 revalidatePath）→ 实现后复跑 5 条全绿。
- 冒烟（next start 于本会话端口 3103，验毕已关闭）：/leaderboard、/zh-CN/leaderboard、/ja/leaderboard、/zh-CN/leaderboard?period=week&sortBy=cost&page=2&search=model:kimi、/leaderboard?period=custom&from=2026-10-01&to=2026-10-31 五路均 HTTP 200 且 Cache-Control 为 `public, max-age=0, s-maxage=30, stale-while-revalidate=60`——即本轮清除所命名的副本形态；本地无 DATABASE_URL，页面按 isMissingDatabaseUrl 空板兜底渲染，属预期。
- 冒烟与验证结束后：无本会话遗留 next 进程；lint 对比用临时 worktree 已移除。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-14T13:10:00.000000+08:00 | planned | in-progress | 开工：范围、验收与 Book Gate Plan 已确认，调查完成 | task.md 已确认范围与授权节；分支 sync/upstream-v27.2.1-pr-c，工作区开工前干净，与 origin/main(a856909) 一致；血缘核对结论见进度节 |
| 2026-11-14T14:45:00.000000+08:00 | in-progress | checking | 第二轮完成全部移植与取舍回贴：18 个 C 提交落地＋2 个核对为已在仓、42e63c2 功能部分落地、4c98999 测试先红后绿、七条验证命令与生产/CLI 冒烟全绿，进入收尾核对 | task.md 进度节第二轮各条与证据节；web: tsc exit 0、lint 0 errors/1 warning（与 base a856909 实测相同，新增 0）、bun test 121 pass/0 fail、build exit 0；CLI: cargo test 2533 pass/0 fail/4 ignored、clippy -D warnings exit 0、release build exit 0；OG 三类卡与三语 help 冒烟通过，next start 已关闭 |
| 2026-11-14T14:45:00.000000+08:00 | checking | done | 验收条件 1-9 全部满足：20 个 C 类提交逐条有去向、4c98999 回归测试落地、OG 三卡三语与缓存头正确、解析器行为随上游并测试通过、MCP 默认关闭且服务端门控生效、tokensw 与打包段落地且非 Windows 编译不变、七条验证命令真实执行通过、模型表两轮计数真实、本文件禁用词与本机路径扫描零命中、next start 等进程已关闭；不 commit、不 push | 证据节全部实跑记录；门禁结论节四项评审（release-readiness 记 blocked：CI、合并、部署与 npm trusted publisher 登记不在本会话授权内）；grep 禁用词与本机路径扫描零命中 |
| 2026-11-14T15:05:00.000000+08:00 | done | planned | 重开：2026-11-14 13:59–14:12 群成员投票（甲），Lord 14:12 记录，榜单转私密共享缓存清除并进 PR C（第二轮完成证据保留于上行与证据节，本轮完成后另行记录） | 已确认范围与授权节第二条：2026-11-14 13:59–14:12 群成员投票（甲），Lord 14:12 记录 |
| 2026-11-14T15:05:00.000000+08:00 | planned | in-progress | 第三轮开工：缓存层调查完成，统一标签与三语路径清除方案确定 | 进度节 15:03:57 调查结论（1)-(5) |
| 2026-11-14T15:24:28.000000+08:00 | in-progress | checking | 第三轮完成：lib/leaderboard/cache.ts 落地统一标签与 revalidateLeaderboard()、privacy.ts 两条转私密路径共用该 helper、getLeaderboard.ts 三处 tags 改引用常量、cache.test.ts 5 条中 3 条先红、2 条先绿、实现后全绿，七条验证命令与冒烟全绿，进入收尾核对 | 进度节 15:10:44 与 15:15:51 各条、证据节第三轮块；web: tsc exit 0、lint 0 errors/1 warning（与 base a856909 实测相同，新增 0）、bun test 126 pass/0 fail、build exit 0；CLI: cargo test 2533 pass/0 fail/4 ignored、clippy exit 0、release build exit 0；冒烟三语榜单与查询变体均 200 且共享缓存头在位，next 进程已关闭 |
| 2026-11-14T15:24:28.000000+08:00 | checking | done | 验收条件 1-10 全部满足：20 个 C 类提交去向与第三轮方案甲落地（统一标签立即过期＋三语榜单地址全路径清除，5 条测试 3 条先红、2 条先绿、实现后全绿）、七条验证命令真实执行通过、模型表三轮计数真实、本文件禁用词与本机路径扫描零命中、next start 等进程已关闭；不 commit、不 push | 证据节第二轮与第三轮实跑记录；门禁结论节含第三轮补评（release-readiness 维持 blocked）；grep 禁用词与本机路径扫描零命中 |
