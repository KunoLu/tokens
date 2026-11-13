---
schema_version: 1
id: 20261114-v2721-pr-a-i18n
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-11-13 10:32 群内选定多语言方案甲、10:33 确认 A/B/C 拆法；2026-11-14 01:33 群内确认恢复开工，Lord
  记录；本会话明确选择 strict。
status: "done"
branch: sync/upstream-v27.2.1-pr-a-i18n
created_at: '2026-11-14T01:46:00.696645+08:00'
updated_at: "2026-11-14T04:31:42.388367+08:00"
completed_at: "2026-11-14T04:31:42.388367+08:00"
---
# 同步上游 v27.2.1 拆分 PR A — 多语言（方案甲）

## 目标

web 与 CLI 切换到上游 v27.2.1 的多语言框架：en／zh-CN／ja 词条文件、语言切换器、URL 语言前缀、middleware；停用本仓自制实现（`web/src/lib/i18n` 旧五件套）。本仓独有词条（teams、teamboard／团队榜、邮箱登录、隐私模式、导航、页脚等）迁入上游格式，三种语言齐备，ja 按上游既有译法补齐。

## 已确认范围与授权

- 用户 2026-11-13 10:32 群内选定多语言方案甲、10:33 确认 A/B/C 拆法；2026-11-14 01:33 群内确认恢复开工，Lord 记录。
- 本会话明确选择 strict 模式。不 commit、不 push，只改工作区文件。
- 起点：`main` 3277189e9e784c870b1b2c37ec1027518b32675b；分支 `sync/upstream-v27.2.1-pr-a-i18n`。上游 `missuo/tokens` tag v27.2.1 = e39110ddc5ca9b2b8b25abcf57738b23931755d1，双方共同祖先 f114057。
- PR A 内容：上游 cbf1ecd（Merge PR #117 feat/i18n，含 6249779、59f31ae、6011333、7ed4f29）引入的多语言改动，覆盖 web 104 个文件与 cli 27 个文件。
- 不在本 PR：上游其它功能与修复提交（cursor、codex compaction、devin／pi／dsh 解析、MCP opt-in、Windows autosubmit、speed tiers、yesterday、private mode、部署与缓存 perf 等）；混合文件只取多语言部分，未取部分登记入「碰到但未做的文件」。
- 兼容承诺：旧 cookie `tt_locale=zh` 映射为 zh-CN；旧不带语言前缀的中文 URL 能打开或重定向，不 404。
- 未完整调用 grill-with-docs：范围、来源、验收与授权均已明确，无需求歧义，不重复访谈。

## 验收条件

1. web 与 CLI 均由上游框架渲染 en／zh-CN／ja；旧实现文件删除，全仓无对旧 API（`useI18n`／旧 `t`／`parseLocale` 等）的残留引用。
2. 本仓独有词条三语齐备并接入上游 messages 目录。
3. middleware：`tt_locale=zh` 按 zh-CN 处理；旧无前缀 URL 返回 200 或重定向，无 404；为这两点新增测试并通过。
4. 验证命令全部通过或写明失败原因：`cd web && bun run lint`、`bunx tsc --noEmit`、`bun test`、`bun run build`；`cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`、`cargo test --manifest-path cli/Cargo.toml`、`cargo build --release --manifest-path cli/Cargo.toml -p tokens-cli`。
5. task.md 记录词条三计数（迁移／复用上游／丢弃，附统计方法）、引入的上游文件清单（路径＋上游来源）、真实验证命令与结果。

## 模型与调用

| 角色 | 模型 | 密钥来源 | 调用次数 |
|---|---|---|---|
| 主 agent | k3:max | 环境变量 KIMI_API_KEY | 408（截至 2026-11-14 04:30 +08:00，按本会话转录的 assistant 轮次逐轮计数，含本次写入；计数证据为会话转录 `## assistant` 块数） |

未启动子代理；密钥不写入本文件与任何产物。

## Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill；teams／teamboard 等术语在本仓代码与既有任务中已定型，无模型歧义 | 需求 | not-required |
| book-ddia-data-design | 不改 schema／migration／持久缓存／跨进程数据流；`tt_locale` 取值映射属客户端 cookie 兼容，无数据格式变更 | 设计 | not-required |
| book-legacy-change-safety | 替换既有自制多语言与路由行为，高回归风险且现有行为需刻画 | 首次行为修改前 | passed（characterized；normal） |
| book-refactoring-pass | 修改既有生产代码（web 组件、CLI Rust 源） | 首次生产编辑前 | passed（proceed；normal；no refactor needed） |
| book-release-readiness | middleware／URL 路由行为变更 | 验证通过后、收尾前 | blocked（本地门禁全绿；CI、合并、部署不在本会话授权内） |

TDD：middleware 的 `tt_locale=zh` 映射与无前缀 URL 重定向先写失败测试再实现。

## 进度

- 2026-11-14 01:43 +08:00 主 agent：开始基线与行为刻画。
- 2026-11-14 01:52 +08:00 主 agent：完成基线与行为刻画。验证：`cd web && bun test` 51 pass／0 fail；`bun run lint` 0 errors（1 个既有 img warning）；`bunx tsc --noEmit` 通过。CLI 基线：本机缺 `make` 且 rustc 1.85.1 低于代码所需的 1.87（`is_multiple_of`），基线编译报 E0658×3；在 `/tmp` 架设隔离 make 4.4.1 与 Rust 1.100.0（rustup minimal profile，不动 HOME 与仓内配置）后 `cargo test --manifest-path cli/Cargo.toml` 2456 pass／0 fail／4 ignored（tokens-cli 204＋tokens-core 2198＋其余 54）。现状刻画：`layout.tsx` 读 `tt_locale` cookie，`parseLocale` 仅认 `zh`；`Navigation.tsx` 经 `localeCookieValue` 写 cookie 切语言；`teams/api.ts` 经 `serverError.ts` 本地化错误；`/settings` 未登录重定向 `/login?returnTo=...`。发现：本仓 2fc41cd 已移除 Cloudflare worker 层，上游 `web/worker.ts` 改动不适用本仓；上游 middleware 的共享缓存清单依赖本仓没有的 perf 提交（36d17e1 等），移植时按本仓实际改写。
- 2026-11-14 01:52 +08:00 主 agent：开始框架文件导入（上游新增文件落地）。
- 主 agent：完成框架文件导入与 web 移植（本条于 2026-11-14 04:30 +08:00 补记时间戳；内容与验证先于 CLI 移植完成，写作当时未读时钟，不写估计值）。上游 i18n 框架文件（config／format／translate／server／rich、messages 全目录、LocaleProvider、LanguageSwitcher、not-found、CLI locales 与 i18n/mod.rs）取自 cbf1ecd 落地；58 个旧 i18n 引用点全部改写；旧模块 t.ts／locale.ts／I18nProvider.tsx／index.ts 已删除，serverError.ts 换成新 Translator 接口。middleware 移植为 `web/src/middleware.ts`（旧 `web/middleware.ts` 删除），按本仓实际调整：/settings 未登录重定向到 `/login`（邮箱登录，上游是 /api/auth/github）；`tt_locale=zh` 归一为 zh-CN；共享缓存清单只留本仓真正与读者无关的路由（/docs、/terms、/privacy、/u/[username]），/leaderboard 在本仓仍读会话故不缓存；matcher 去掉 /shame（本仓无此页）。发现上游 bug：v27.2.1 `u/[username]/page.tsx` 把四条 profile 元数据键写成了 `meta.*`（实际在 `profile.meta.*` 下），会渲染原始键名；本仓移植时用正确键，已记录。验证：新增 `web/src/middleware.test.ts`（19 条）先红后绿；新增 `web/src/lib/i18n/coverage.test.ts` 守护键存在性与三语齐备；`bunx tsc --noEmit` 通过；`bun run lint` 0 errors（2 warnings：既有 img 一条＋已清的一条已修）；`bun test` 99 pass／0 fail；`bun run build` 成功（42 页）；`next start` 后 curl 实测：`tt_locale=zh` 访问 `/` → 307 `/zh-CN/leaderboard`，`tt_locale=ja` → `/ja/leaderboard`，旧无前缀 `/leaderboard` 200 英文（不 404），`/zh-CN/leaderboard` 渲染排行榜且 `lang="zh-CN"`，`/ja/terms` 渲染日文条款，`/en/leaderboard` → 307 `/leaderboard`，`/zh-CN` → 307 `/zh-CN/leaderboard`，`/settings` 未登录 → 307 `/login?returnTo=/settings`，`/zh-CN/settings` 保留前缀。
- 主 agent：开始 CLI 移植（18 个既有 Rust 源文件的三方合并；同上，补记于 2026-11-14 04:30 +08:00）。
- 2026-11-14 04:30 +08:00 主 agent：完成 CLI 移植与全部验证（本条在 04:30 读取时钟后写入）。13 个文件零冲突直接合并；cursor.rs 两处冲突取本仓侧（sync_start 增量同步及其测试属后续 PR）；auth.rs 取本仓结构手工落键（本仓 enable_private_mode 用 serde_json::Value 与上游结构不同）；`cli.login.private.help` 目录值用回本仓既有帮助文本（三语）。发现并修复一处既有 `needless_bool`（新 clippy 才报，等价改写）。验证：cargo test 2467 pass／0 fail／4 ignored；clippy -D warnings exit 0；release build exit 0；`tokens --lang zh-CN/ja --help`、`TOKENS_LANG=zh-CN`、`--lang xx` 错误提示全部实测正常。
- 2026-11-14 04:30 +08:00 主 agent：完成词条统计、BDD 改写（`web/features/i18n.feature` 与 `tests/e2e/i18n-locale.spec.ts` 改为前缀方案，e2e 未运行已注明）、ponytail-review（Lean already）、占位符审计、污染扫描与收尾文档。开始最终核对。

## 证据

全部命令在分支 sync/upstream-v27.2.1-pr-a-i18n 工作区真实执行（HEAD 3277189e9e784c870b1b2c37ec1027518b32675b + 本任务未提交改动）。本机缺 `make` 且预装 rustc 1.85.1 低于代码所需的 1.87，CLI 验证使用 /tmp 下的隔离工具链（make 4.4.1 来自 Debian trixie 官方 deb 手工解包；Rust 1.100.0 经 rustup minimal profile 装入 /tmp/rustup，RUSTUP_HOME/CARGO_HOME 均指向 /tmp，不改 HOME、不动仓内配置）。

### 基线（改动前）

- `cd web && bun test`：51 pass／0 fail（8 个文件）。
- `cd web && bun run lint`：0 errors，1 个既有 `no-img-element` warning（docs/page.tsx）。
- `cd web && bunx tsc --noEmit`：通过。
- `cargo test --manifest-path cli/Cargo.toml`：2456 pass／0 fail／4 ignored（tokens-cli 204、tokens-core 2198+2ig、其余套件 54+2ig）。
- 预装工具链下 CLI 基线编译即失败（E0658×3：`is_multiple_of` 需要 Rust ≥1.87），属环境问题，与本次改动无关。

### 改动后验证

- `cd web && bunx tsc --noEmit`：通过（exit 0）。
- `cd web && bun run lint`：0 errors，1 warning（同一条既有 img warning）。
- `cd web && bun test`：99 pass／0 fail（13 个文件）。新增：`src/middleware.test.ts` 19 条（tt_locale=zh→zh-CN 映射、三语 cookie、无前缀旧 URL 200 不跳转不重写、前缀 rewrite＋x-tokens-locale 头、/en 归一、裸前缀重定向、/settings 未登录→/login 且前缀保留），先红（模块不存在）后绿；`src/lib/i18n/coverage.test.ts` 2 条（全部源码引用的键在 en 目录可解析；auth／teams／teamboard／docs／legal 五个本仓命名空间 zh-CN/ja 逐键齐备）。上游自带 config.test.ts／format.test.ts／messages/index.test.ts 随套件通过。
- `cd web && bun run build`：成功（42 页生成）。
- `cd web && bunx next start` 后 curl 实测（生产构建）：`tt_locale=zh` 访问 `/` → 307 `/zh-CN/leaderboard`；`tt_locale=ja` → `/ja/leaderboard`；`/leaderboard` → 200 英文（旧无前缀中文地址不 404）；`/zh-CN/leaderboard` 渲染「排行榜」且 `<html lang="zh-CN">`；`/ja/leaderboard` 渲染「リーダーボード」；`/zh-CN/terms`、`/ja/terms` 渲染对应译文且带英文版为准提示；`/zh-CN/teams`、`/zh-CN/docs` 中文渲染；`/zh-CN/login`、`/ja/login` 本地化且注册/找回链接携带 `/zh-CN` 前缀；`/en/leaderboard` → 307 `/leaderboard`；`/zh-CN` → 307 `/zh-CN/leaderboard`；`/settings` 未登录 → 307 `/login?returnTo=/settings`；`/zh-CN/settings` → 307 `/login?returnTo=%2Fzh-CN%2Fsettings`；`/zh-CN/u/不存在用户` → 404 且渲染中文「找不到这个用户」。
- `cargo test --manifest-path cli/Cargo.toml`：2467 pass／0 fail／4 ignored（tokens-cli 215＝204 基线＋11 条上游 i18n 测试；其余与基线一致）。
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings`：exit 0。过程中发现一处既有 `needless_bool`（antigravity.rs 测试辅助，1.100 clippy 才报），按 lint 建议改为等价单行，行为不变。
- `cargo build --release --manifest-path cli/Cargo.toml -p tokens-cli`：exit 0。产物冒烟：`tokens --version` 输出 1.0.4；`--lang zh-CN --help` 输出中文命令树；`--lang ja status --help` 输出日文；`TOKENS_LANG=zh-CN` 生效；`--lang xx` 报本地化错误「Unknown value for --lang」。
- 词条占位符审计（脚本展开三语目录比对 `{var}` 集合）：仅 `common.tokenGraph3d.dayWord` 命中，确认为 ICU 分支词差异（英 days/day、中 天、日 日）的误报，真实占位符一致。
- Playwright e2e（`bun run test:e2e`）：未运行——需要数据库与 fixture 环境，CI 也不跑此套件（CI 的 Web 任务为 lint＋typecheck＋build）。`tests/e2e/i18n-locale.spec.ts` 与 `web/features/i18n.feature` 已改写为前缀方案；未运行在本节如实标注，不冒充通过。

## 碰到但未做的文件（归后续 PR）

- `web/src/app/api/og/route.tsx`：上游多语言版 OG 渲染器依赖 `lib/og/ogImage`、`lib/og/fonts.generated`（另有非多语言 PR 的 OG 重做），本 PR 不引入；本仓 OG 保持既有英文输出。`og` 词条命名空间随之未注册（上游 14 条丢弃待后续 OG PR 一并接回）。
- `web/src/lib/og/fonts.generated.ts`、`web/scripts/generate-og-fonts.ts`、`web/scripts/og-cjk-hanzi.txt`：同上 OG 链路，未引入（og-cjk-hanzi.txt 曾随批量导入误落地，已删）。
- `web/worker.ts`：本仓 2fc41cd 已移除 Cloudflare worker 层，上游对 worker 的多语言改动不适用。
- `web/src/components/ui/VerifiedBadge.tsx`：本仓在邮箱登录改造（5aa99c8）中删除；上游对其的多语言改动不适用，`profile.actions.github`、`common.verifiedTooltip` 键随之丢弃。
- `web/src/app/(main)/shame/page.tsx`、`web/src/components/shame/BannedList.tsx`：本仓 6759e18 已用 Teamboard 取代 Shame 页；shame 命名空间（上游 18 条）未引入。
- 混合文件中的非多语言部分（只做多语言、其余归后续 PR）：`cli/tokens-cli/src/cursor.rs` 的 sync_start 增量同步及其测试模块；上游整榜下发（36d17e1）、榜单流式修复（b3a0a60）、昨日卡片（ba3ad78）；上游 16724c8 版设置页（本仓有自己的隐私模式实现）；上游 MCP opt-in（42e63c2）在 cbf1ecd 之后对 docs/privacy/legal 词条的修订（本 PR 词条取 cbf1ecd 时点）。
- 隐私政策页的 cookie 条文（「仅一枚 cookie，登录后设置」）在旧方案下就已不准确（旧切换器未登录也写 tt_locale）；属既有文案问题，未在本 PR 改法律文本，记录待后续修订。

## 上游缺陷记录

- v27.2.1 `web/src/app/u/[username]/page.tsx`（含 cbf1ecd）把四条 profile 元数据键写成 `meta.title`／`meta.description`／`meta.statsWithRank`／`meta.statsNoRank`，而这些键实际在 `profile.meta.*` 下：title/description 会命中站点级同名字符串（内容错），statsWithRank/statsNoRank 不存在（渲染原始键名）。本仓移植时使用正确的 `profile.meta.*` 键。未向上游提交修复（本任务不开 PR 上游）。

## 词条统计

口径：以旧目录 `t.ts` 的 en 词条全集（921 条，zh 同数一一对应）为基数，按「旧键的去向」分类，每条旧键恰好归入一类；分类由脚本对全文文本匹配＋逐文件改写映射表生成，尾差人工逐条裁定。三方脚本与中间产物在 /tmp（会话临时区），分类全表未入库。

- 迁移 413 条：旧词条文字进入新目录。其中 398 条落入本仓命名空间（teams 118、auth 69、legal 139、docs 41、teamboard 31，均含三语）；15 条落在既有上游命名空间（nav.teamboard／nav.teams、settings.profile.lockedNote、settings.privacy.stateOn／stateOff／turnOn／turnOff、profile.today.dayFallback、profile.membership／team／group／leaveGroup、leaderboard.table.team／group），另有 3 条本仓文案覆盖了上游同键值（settings.privacy.description、settings.readTokens.description、settings.devices.description；迁移数按旧键计，这 3 条含在 398 之外的键位替换里、与上述 15 条并列）。ja 译文全部由本任务按上游 ja 惯例新写（旧库无 ja）。
- 复用上游 467 条：旧键退役，由上游目录同义键承接。含整件采用上游组件后的键位替换（graph.* 104、chart.* 80、profile.* 大部、embed.* 48→profile.embed.*、device.* 16→settings.pairing.*、local.*→settings.local.*、dataInput.*→settings.local.input.* 等），以及 SettingsClient／BannedProfileView／not-found 等逐键映射；上游目录原样保留 491 条（值逐字相同）。
- 丢弃 41 条：weekdays.* 14 条与 profile.habits.weekdays* 2 条（改由 Intl.DateTimeFormat 生成）、relative.*Ago 6 条（改由 Intl.RelativeTimeFormat 生成）、graph.palette.* 9 条（上游改读 themes.ts 英文名）、graph 日期/星期简写 4 条（Intl）、nav.english／nav.chinese 2 条（切换器改用 LOCALE_LABELS 本地化名）、footer.copyright 1 条（上游目录内联字面量）、settings.tokens.defaultName 1 条（上游字面量 "CI token"）、profile.speed.sourceSeparator 1 条（上游字面量逗号）、teams.invite 1 条（与 teams.invite.title 同文，合并）。
- 校验：413＋467＋41＝921，无未归类键。最终 en 目录 903 条＝上游原样保留 491＋上游键位本仓文案 5＋本仓新增键 407（docs.meta.* 3 条为全新键，无旧键对应）。上游目录未引入 292 条：legal 129 与 docs 80（上游法律/文档原文在本仓不渲染，整段替换为本仓文案）、shame 18 与 og 14（对应页面/渲染器不在本仓）、profile 18／settings 18／leaderboard 9／common 3／nav 2／footer 1（昨日功能、上游私有模式设置页、赞助位等本仓没有的功能词条）。

## 引入的上游文件（均取自 cbf1ecd，即上游 PR #117 合并结果）

新增落地（本仓此前没有）：`web/src/lib/i18n/` 下 config.ts、format.ts、translate.ts、server.ts、rich.tsx、config.test.ts、format.test.ts、messages/ 全目录骨架与各命名空间文件、`web/src/components/layout/LanguageSwitcher.tsx`、`LocaleProvider.tsx`、`web/src/app/not-found.tsx`、`web/src/middleware.ts`（上游同名文件经本仓适配：/login 重定向、zh 旧值映射、缓存清单与 matcher 按本仓实际）；`cli/tokens-cli/src/i18n/mod.rs`、`cli/tokens-cli/locales/{en,zh-CN,ja}/{cli,top,commands}.json`。
整体采用上游版本覆盖本仓旧实现的文件：`web/src/lib/providers/Providers.tsx`、`error.tsx`、`local/page.tsx`、`settings/page.tsx`、`local/LocalClient.tsx`、`device/DeviceClient.tsx`（登录链接改 /login）、`u/[username]/not-found.tsx`、`BannedProfileView.tsx`（去掉 shame 回链）、`ProfilePageClient.tsx`、`Tooltip.tsx`、`GraphControls.tsx`、`StatsPanel.tsx`、`TokenGraph2D.tsx`、`TokenGraph3D.tsx`、`GraphContainer.tsx`、`BreakdownPanel.tsx`、`DataInput.tsx`、`docs/CommandBlock.tsx`、`legal/LegalPage.tsx`（保留本仓 CONTACT_EMAIL 环境变量与日期格式化）、`ThemedToastContainer.tsx`、`profile/` 下 SpeedTier、ProfileSocialLinks、ProfileModels、ProfileUsageChart、ProfileContributionGraph、ProfileDevices、ProfileEmbedDialog、ProfileHabits、TokenBreakdown、usageChartData.ts、ProfileToday.tsx（按本仓无昨日卡片的实际改写）、ProfileView.tsx（去掉 VerifiedBadge 与 GitHub 按钮、保留 membership）、`lib/format.ts`、`lib/date-utils.ts`、`lib/constants.ts`、`lib/og` 相关不引入（见未做清单）。
CLI 18 个既有 Rust 源文件以 cbf1ecd^1 为基、cbf1ecd 为上游侧做三方合并（cursor.rs 两处冲突取本仓侧，auth.rs 逐段手工落键，cloud.rs/main.rs 取上游侧）。

## 门禁结论

### Legacy Change Safety Review

- Status: characterized；Review mode: normal。
- 变更行为：语言解析从 cookie 驱动改为 URL 前缀驱动（en 无前缀）、词条目录与切换器整体替换、CLI 输出三语化。
- 保留行为：`/settings` 未登录重定向 `/login?returnTo=...`（邮箱登录）；teams API 错误本地化（serverError 精确映射原样保留）；`tt_locale=zh` 旧值在 `/` 仍视为中文；旧无前缀 URL 不 404；本仓各功能页文案不变。
- 现状证据（改动前）：`layout.tsx` 读 `tt_locale` cookie 并仅认 `zh`；`Navigation.tsx` 写 cookie 切语言；基线 bun 51 pass、cargo 2456 pass、lint/tsc 通过。
- 安全网：既有 51 条 bun 测试＋新增 middleware 19 条与目录覆盖 2 条（先红后绿）＋tsc／lint／cargo 全量＋next start 生产构建 curl 实测。

### Refactoring Review

- Status: proceed；Review mode: normal；Decision: no refactor needed（整体切换，非重构）。
- 既有代码范围：58 个 web 旧 i18n 引用点、18 个 CLI Rust 源文件、middleware、layout。
- 不变行为：如上节保留行为；英文默认渲染逐字节等价于上游设计。
- 结构判断：两套 i18n API 不兼容，适配层会造成第二套并行写法，故整体替换并同任务删除旧模块；无先行重构。
- 延期重构：本仓 `lib/utils.ts` 与 `lib/format.ts` 的数字格式化并存为既有状态，不在本任务统一。

### Ponytail-review（对本次 diff 的复杂度审查）

- 结论：Lean already。新增代码（middleware 适配、coverage 测试、serverError 重写、api.ts 签名化）无投机抽象、无死代码；上游镜像文件不评审删改。唯一记录的既有债务：utils/format 双格式化器（延期，见上）。

### Release Readiness Review

- Status: blocked（按词表：必需验证在会话内无法补齐的部分见下）。
- Production path：站点路由与中间件（前缀 rewrite、重定向、缓存头）、CLI 终端输出。影响面：全部站点访客（URL 行为）与选择非英语的 CLI 用户。
- Failure modes/safeguards：旧链接由「无前缀＝英文」规则保证不 404；旧 cookie 在 `/` 归一为 zh-CN；共享缓存清单只含本仓真正与读者无关的路由，`/leaderboard` 在本仓仍读会话故不缓存；RSC 请求不重写查询串。上游整榜下发（36d17e1）未同步前不得把 `/leaderboard` 加入共享缓存清单——已写入代码注释。
- Capacity/limits：无新增运行时路径、队列或外部调用；中间件为常量路径运算。
- Observability/runbook：上线后抽查 `/`、`/zh-CN/leaderboard`、`/leaderboard` 三者的状态码与 lang 属性即覆盖本 PR 的全部路由新行为。
- Rollout/rollback：纯代码变更，回滚即还原；无 schema／数据迁移。Git 回退不保全 ignored 数据（本任务无新增 ignored 产物）。
- Required validation：上文「改动后验证」全部通过；未运行：Playwright e2e（需数据库与 fixture，CI 亦不跑）、CI（本会话不 commit/push）、真实部署。残余风险由编排方在 PR 流程处置（Reviewy 评审＋CI），本文不记录任何豁免。

### BDD 与工具边界

- BDD：`web/features/i18n.feature` 与 `tests/e2e/i18n-locale.spec.ts` 已改写为前缀方案（中文场景＋英文关键词沿用项目惯例）；e2e 未运行，如实标注。
- Cross-repo context: contract-only（上游 tag 对象）；API Contract: not-needed（无 API 变更）；Mock Strategy: not-needed；E2E Mode: smoke-only（next start＋curl）。
- Chrome/Playwright MCP: skipped（用生产构建＋curl 验证真实 HTTP 行为）；Maestro: not-needed。Graft: skipped（无可调用连接）；LSP: 用于编辑期诊断。rtk: skipped-for-report（全部原生命令，保留完整输出）。
- 独立评审：本会话无可派生的只读子代理能力，未启动；独立评审由编排方在 PR 上进行（沿用 Reviewy 惯例）。作者自检为逐文件 diff 复审＋占位符审计＋污染扫描。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-14T01:46:02.377209+08:00 | planned | in-progress | 开工：范围、验收与 Book Gate Plan 已确认 | task.md 已确认范围与授权节；分支 sync/upstream-v27.2.1-pr-a-i18n，工作区开工前干净 |
| 2026-11-14T04:31:38.605411+08:00 | in-progress | checking | 实现与移植完成，进入收尾核对 | web: tsc exit 0、lint 0 errors、bun test 99 pass/0 fail、next build 成功、next start curl 冒烟全过；CLI: cargo test 2467 pass/0 fail/4 ignored、clippy -D warnings exit 0、release build exit 0、三语 --help 实测正常 |
| 2026-11-14T04:31:42.388367+08:00 | checking | done | 验收条件 1-5 全部满足：三语框架落地且旧实现删除无残留引用、独有词条三语齐备、tt_locale=zh→zh-CN 与无前缀 URL 均有测试且通过、验证命令全绿、统计与文件清单已写入 task.md | task.md 证据节；工作区 diff（88 改＋新增框架与词条文件）；release-readiness 记 blocked：CI／合并／部署不在本会话授权内，由编排方在 PR 流程处置 |
