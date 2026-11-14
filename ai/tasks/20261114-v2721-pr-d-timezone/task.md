---
schema_version: 1
id: 20261114-v2721-pr-d-timezone
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-11-14 01:50 在群内开启全自动模式直到 v1.0.5 按模板发布且文档补全；用户 2026-11-14 08:01 群内确认（乙：存账号设置，留空按浏览器自动检测），Lord 记录；迁移随 PR D 一起审，生产环境执行迁移需用户另行批准。本会话交付清单含 Book Gate Plan 与完整验证记录，按 strict 执行。
status: "done"
branch: fix/21-timezone-setting
created_at: '2026-11-14T17:36:18.000000+08:00'
updated_at: "2026-11-14T23:24:39+08:00"
completed_at: "2026-11-14T20:35:29.677565+08:00"
---
# 修复 issue #21（PR D）— 个人页按看的人时区归日 + 账号设置时区选项

## 目标

修 issue #21：个人页图表的「今天」现在按服务器 UTC 日期算，UTC+8 用户每天 00:00–08:00 看不到当天数据。落地四条改动：

1. 默认用浏览器检测到的时区（`Intl.DateTimeFormat().resolvedOptions().timeZone`）决定图表结束日、「今天/昨天」卡片和相关统计的归日，不再用服务器 UTC 日期。
2. 账号设置页加时区选项，默认「自动检测」；选了具体时区就覆盖检测值，全站按它归日。存储：users 表新增可为空的 timezone 字段（null = 自动检测），附数据库迁移文件（drizzle-kit，web/src/lib/db/migrations 现有惯例）；服务端校验必须是合法 IANA 名称；设置接口读写这个字段。
3. 看别人的个人页按看的人的时区归日（看的人设置了就用设置，否则用其浏览器检测值），不按页面主人的；规则写进文档（docs 页，en、zh-CN、ja 三语）。
4. UTC 用户行为不变。

## 已确认范围与授权

- 用户 2026-11-14 01:50 在群内开启全自动模式直到 v1.0.5 按模板发布且文档补全；用户 2026-11-14 08:01 群内确认（乙：存账号设置，留空按浏览器自动检测），Lord 记录；迁移随 PR D 一起审，生产环境执行迁移需用户另行批准。
- 本会话按 strict 执行。不 commit、不 push、不开 PR，只改工作区文件。
- 起点：`main` 8715c0e6f0464801f1ebc1a3144e8028ff06c9d3（已含 PR A #20、PR B #22、PR C #23）；分支 `fix/21-timezone-setting`，开工前 `git status` 实测干净。
- 迁移文件只生成并放进仓库，不对任何真实数据库（生产、预览）执行；本机无 DATABASE_URL、无本地 postgres 实例，本仓验证不实跑迁移；不改 vercel.json、不碰部署。
- 迁移执行时机（按 web/vercel.json 原文核对）：buildCommand 仅当 VERCEL_ENV 为 production 时先跑 db:migrate 再执行构建，预览部署不跑迁移；git.deploymentEnabled.main 为 false，main 分支不自动部署，合入本 PR 本身不动线上数据库；真正执行迁移的是之后那次手动正式部署，需用户另行批准（次序与预览限制的完整论述见 Release Readiness 评审）。
- 未完整调用 grill-with-docs：issue #21 的三条改动请求与四条验收条件逐条写明，方案（乙）与授权边界已选定，需求无歧义，不重复访谈。

## 验收条件

1. UTC+8 浏览器中，本地 00:00 之后提交的数据立刻出现在「今天」格，不必等到 08:00：图表结束日、「今天/昨天」卡片与相关统计按看的人时区归日，不再用服务器 UTC 日期。
2. 设置页选了具体时区后，图表与「今天/昨天」归日随之改变；切回「自动检测」回到浏览器检测时区。users 表新增可为空 timezone 字段与迁移文件；服务端拒绝非法 IANA 名称；设置接口可读写字段。
3. 看别人个人页时按看的人时区归日（设置优先，其次浏览器检测），不按页面主人的；规则写进 docs 页（en、zh-CN、ja 三语）。
4. UTC 用户行为不变。
5. 新增界面文案 en、zh-CN、ja 三语齐备，i18n 覆盖测试通过。
6. TDD：归日逻辑（UTC+8 的 00:30、UTC−5、UTC 边界）、时区校验、设置读写、看别人页面用看的人时区的测试先跑红再转绿，真实时间记入进度节。
7. 验证全部真实执行并记录：`cd web && bunx tsc --noEmit`、`bun run lint`（与 base 8715c0e 对比不得新增错误或警告）、`bun test`、`bun run build`。本 PR 不改 Rust CLI，cargo test 与 clippy 不跑。
8. 本文件模型表为真实计数；本文件通过禁用词与本机路径扫描，零命中。
9. 跑完验证后关掉本会话启动的 next start 等进程。

## Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill；时区、归日、贡献格等术语在 issue #21 与既有代码注释中已定型，无模型歧义 | 需求 | not-required |
| book-ddia-data-design | users 表新增 timezone 列与迁移文件、设置读写路径 | 设计稳定前 | passed（confirmed；结论见门禁结论节） |
| book-legacy-change-safety | 改变个人页图表结束日与 week/month 过滤的既有行为 | 首次行为修改前 | passed（characterized；normal；结论见门禁结论节） |
| book-refactoring-pass | 修改既有生产代码（publicProfileData、session、ProfilePageClient、SettingsClient） | 首次生产编辑前 | passed（proceed；normal；no refactor needed；结论见门禁结论节） |
| book-release-readiness | 新增设置接口、个人页 API tz 参数、迁移文件 | 验证通过后、收尾前 | blocked（本地门禁全绿；迁移执行、CI、合并与部署不在本会话授权内；结论见门禁结论节） |

TDD seam 清单（沿用本仓既有测试位置与 mock 惯例，均为公开接口）：

- `web/src/lib/timezone.ts`（新）：纯函数 `normalizeIanaTimeZone`（第五轮前名 `isValidIanaTimeZone`）、`dateStringInTimeZone`、`addDaysToDateString`、`resolveViewerTimeZone`，bun test 直测；同一校验规则另有 node --test 用例（web/scripts/test-timezone-node.mjs，直引生产文件）。
- `web/src/lib/publicProfileData.ts`：`getProfilePeriodDateRange` / `getRollingProfileDateRange` 导出后按固定时刻直测；路由层 `tz` 查询参数经 mock 的 db 测响应区间与 400。
- `web/src/app/api/settings/timezone/route.ts`（新）：mock db 与 getSessionFromRequest，测未登录 401、非法时区 400、写入与清空、读取。

## 进度

- 2026-11-14 17:19:59 +08:00 主 agent：第一轮开始（会话头；原记 17:20:02 为会话首条消息时间，补记，按会话记录还原）；调查完成于 17:29:10（首个实测时间戳；调查本身在此之前进行）。结论：
  1. 图表结束日来自 `web/src/lib/publicProfileData.ts` 的 `getRollingProfileDateRange`（all 期间，滚动一年）与 `getProfilePeriodDateRange`（week/month），两者用 `toISOString().slice(0, 10)` 取 UTC 日期，`chartRange.end` 即服务器 UTC「今天」，与 issue 实测一致。
  2. period=week/month 时服务器按 periodRange 过滤 daily_breakdown，范围外的行不进响应；UTC+8 看的人的「今天」行（提交者本地日期）在 UTC 午夜前落在范围外，纯前端改区间拿不回该行，故 API 需要 `tz` 查询参数。
  3. 「今天/昨天」卡片（ProfilePageClient）已在挂载后用浏览器本地日期（`toLocalDateString(new Date())`），但不支持设置覆盖；注释明确首屏用区间末日兜底是「安全方向」。
  4. 设置页结构：`web/src/app/settings/SettingsClient.tsx` 主件 + `PrivacySection.tsx` 分节件惯例；设置接口惯例见 `/api/settings/privacy`——`getSessionFromRequest({ allowAuthorizationHeader: false })`（仅浏览器会话，带 CSRF Origin 门），GET/PATCH。
  5. users 表 schema 在 `web/src/lib/db/schema.ts`；迁移目录 `web/src/lib/db/migrations`（drizzle-kit，最近 0027_private_mode，含 meta 快照与 _journal.json）。
  6. i18n 词条在 `web/src/lib/i18n/messages/{settings,docs,profile}/{en,zh-CN,ja}.ts`；`lib/i18n/coverage.test.ts` 强制所有 t() 键在 en 存在且 fork 命名空间三语齐备。
  7. 未登录访客：`/api/auth/session` 返回 `{ user: null }`，看的人时区退到浏览器检测值。
  8. u 页是 ISR（`revalidate = 60`）且经 `unstable_cache` 按 username/period/locale 缓存，服务器渲染不能按看的人会话分叉；看的人时区只能在客户端挂载后解析，首屏维持服务器区间作兜底（取舍说明 2）。
  9. `/api/auth/session` 走 `getSession()`（cookie），已联表查出整行 users；加 timezone 字段为零额外查询。
  10. 文档位置：站内 docs 页 `web/src/app/(main)/docs/page.tsx` 按 Section 组织，文案在 messages/docs 三语；时区规则作为新 Section 写入。
- 2026-11-14 17:36:18 +08:00 主 agent：创建本文件（index 行 17:36:37 写入）；Book Gate Plan 登记；DDIA、Legacy、Refactoring 三项门禁评审结论写入门禁结论节。
- 2026-11-14 17:36:18–17:55:41 +08:00 主 agent：TDD 三切片与收尾，全部实测（各步时钟时间在会话内顺序推进，逐条命令输出留存于会话记录）：
  1. 切片一：`web/src/lib/timezone.test.ts` 先红（模块不存在，1 error）→ 新建 `web/src/lib/timezone.ts`（isValidIanaTimeZone / dateStringInTimeZone / addDaysToDateString / detectBrowserTimeZone / resolveViewerTimeZone）→ 转绿 12 pass。
  2. 依赖恢复：`bun install --frozen-lockfile`（658 包；此前 web/node_modules 缺失，曾致一次 route 测试加载失败）。
  3. 切片二：`publicProfileData.test.ts` 追加 5 条（range 固定时刻三语边界、闰年 2/29 收拢、tz 参数驱动与非法 400、payload 不带 timezone）先红 5 fail（另 3 条保护性断言先绿）→ publicProfileData 两个 range 函数改按 timeZone 归日并导出、路由解析 `tz`（非法 400）→ 与切片一合计 20 pass。路由测试的期望值用 en-CA 直出独立算法，与实现的 formatToParts 组装互为交叉验证。
  4. 切片三：`api/settings/timezone/route.test.ts` 6 条先红（路由不存在）→ 新建路由（GET/PATCH，仅浏览器会话，IANA 校验，null 清除）→ 6 pass。
  5. schema users 加 `timezone: text("timezone")`（可空）；`drizzle-kit generate --name add_user_timezone` 产出 `0028_add_user_timezone.sql`（`ALTER TABLE "users" ADD COLUMN "timezone" text;`）与 0028 快照、journal 条目；不对任何数据库执行。
  6. `getSession` 返回加 `timezone`（SessionUser 可选字段；submit token 路径不带）；设置页新增 `TimezoneSection.tsx` 并挂进 SettingsClient；docs 页新增 time-zone Section；settings/docs 词条 en、zh-CN、ja 三语齐备（修掉一处 zh-CN 错别字后定稿）。
  7. ProfilePageClient：data 改 state、会话应答顺带取看的人 timezone、检测值兜底、resolved 时区算今天/昨天、区间末日不一致时带 `tz` 重取一次（每 period/username/tz 只重取一次防时钟偏斜循环）；UTC 看的人两值恒等，永不重取。
  8. 验证实跑（数字见证据节）：tsc exit 0；bun test 150 pass；lint 0 errors/1 warning，与 base 8715c0e 临时 worktree 实测相同（新增 0，对比后已移除）；build exit 0；冒烟（next start，端口 3121）：未登录 GET 设置接口 401、非法 tz 400、docs 三语 time-zone 小节均在；冒烟后 next 进程已关闭。
  9. ponytail-review：diff 逐文件过一遍——五个纯函数全部被引用、无新依赖、无投机抽象，期间修掉一处真实简化（空贡献列表跳过重取的守卫会在 week 视图留下误空窗，删守卫；随后补一次性重取闸防时钟偏斜循环）。结论：Lean already. Ship.
  10. 可读性复核：手写改动（timezone.ts、路由、TimezoneSection、ProfilePageClient 三个 effect、两处 range 函数）逐段复读，注释说明归日规则与防循环闸；vendor/generated（迁移快照）未动。
- 2026-11-14 18:44:00 +08:00 主 agent：第一轮 TDD 各组红绿时间明细（补记，按会话记录还原；会话时间为 UTC，已加 8 小时换算；以下为命令发出时刻，结果在随后一秒内返回）：
  1. 切片一 `timezone.test.ts`：红 17:37:56（0 pass / 1 fail / 1 error，被测模块不存在）→ 绿 17:38:16（12 pass）。
  2. 切片二 `publicProfileData.test.ts`：用例写入 17:40:24；17:40:36 的首次运行因 web/node_modules 缺失报包找不到（记为环境故障，不算有效红）→ 依赖恢复（bun install 658 包，17:40:51）→ 有效红 17:40:57（5 fail：range 函数未导出、tz 参数被忽略、非法 tz 得 200；3 条保护性断言先绿）→ 实现编辑 17:41:12–17:41:49 → 绿 17:41:55（与切片一合计 20 pass）。
  3. 切片三 `api/settings/timezone/route.test.ts`：红 17:43:12（0 pass / 1 fail / 1 error，路由不存在）→ 绿 17:43:40（6 pass）。
  4. 迁移文件生成 17:44:12；收尾验证：tsc 17:49:51 exit 0、bun test 17:50:15 全量 150 pass、lint 17:50:38（0 errors/1 warning）、base 对照 lint 17:51:19 同警告、build 17:52:12 exit 0、冒烟 17:52:32 通过、next 进程 17:52:48 关闭、合并复跑 17:54:55 全绿。
- 2026-11-14 18:05:05 +08:00 主 agent：第二轮（文档补充，仅改本文件；本轮开始 18:01:50 为会话头，本条时刻 18:05:05 为条目写入时读数，补记，按会话记录还原）。第一轮已提交为 f0778bad0a2b61b87a1a9d538cc11d59d38267ac 并开启 PR #24；Reviewy 复核后要求写清迁移执行时机。本轮按 web/vercel.json 原文核对后落地：授权节新增迁移执行时机条目、取舍说明 8 补充前提与预览验收限制、DDIA 评审 Schema/migration 行与 Release Readiness 评审 Rollout 行补部署次序与预览限制，四处口径一致；模型表补第一轮一行（回复 118、工具调用 150、工具报错 5，2026-11-14 17:19:59–18:00:15 +08:00，起按会话头、原记 17:20:02 为会话首条消息时间，补记，按会话记录还原；逐条核对会话记录原始计数），注脚口径改为按轮取最终计数；frontmatter updated_at 同步。不提交、不推送；状态无迁移，状态事件表不新增行。
- 2026-11-14 18:10:44–18:47:37 +08:00 主 agent：第三轮（Reviewy 18:10 审出 3 条阻塞，一次改完，测试先行；另含三条可选项）。事实与步骤：
  1. 18:25:43 任务重开（done→planned，保留旧完成事件）、18:25:47 进入 in-progress；Legacy/Refactoring 两门禁按本轮范围复核：characterized / proceed（结论见门禁结论节）。
  2. 阻塞①时区校验收紧：先红 18:28:37（timezone.test.ts 新增 4 条用例，3 fail——"+08:00"、非规范大小写、别名 Asia/Chungking/US/Eastern 被构造器校验放行；"GMT+8"、"UTC+8"、空串、超长串此前已被拒）→ `isValidIanaTimeZone` 改为查 `Intl.supportedValuesOf('timeZone')` 规范名集合加显式 "UTC" → 转绿 18:29:55。PATCH 与 `?tz` 本就共用这一个函数，收紧后两处同时生效。
  3. 阻塞②getSession：先红 18:28:44（session.test.ts 新建，4 fail——timezone 来自整行 select 而非单独查询、缺列/出错/无行未降级 null）→ 主查询收窄为只取消费列（id/username/displayName/avatarUrl/bannedAt，与 main 8715c0e 实际消费的列一致），timezone 单独查询、失败降级 null → 转绿。设置接口 PATCH/GET 缺列时返回 503 明确错误（指明先跑 db:migrate）而非笼统 500：先红 18:28:49（route.test.ts 2 fail，得 500）→ 转绿。三文件合计转绿 18:29:55（30 pass）。
  4. 可选项一（授权记录统一）：mode_note 与授权节的乙方案记录统一为「用户 2026-11-14 08:01 群内确认（乙：存账号设置，留空按浏览器自动检测），Lord 记录」，迁移批准事实保留为独立分句。
  5. 可选项二（组件层测试）：新增 `ProfilePageClient.test.tsx`——看别人页面时按看的人时区归日：设置优先（Asia/Shanghai 设置胜过 America/New_York 检测，today 落 2026-11-14 并带 tz 重取一次）、否则浏览器检测（落 2026-11-13 且不重取）。固定时钟 2026-11-13T16:30Z（issue 复现窗口）；归日辅助函数用生产实现，仅壳组件打桩；新增 devDep `@happy-dom/global-registrator` 提供 DOM。18:35:44 两例通过（既有行为钉扎，无红阶段）。
  6. 可选项三（自部署文档）：`docs/deploy/self-host-production.md` 与 `.en.md`（该文档仅中英双语）上线步骤第 2 条后各加一句：升级到含 0028 的版本前先手动跑 db:migrate，未迁移时设置接口时区读写返回 503。
  7. 依赖卫生：`bun add` 夹带的无关改动已还原——package.json 空行格式恢复（只新增一行 devDep）、bun.lock 中 tokens-cli 1.0.2→1.0.4 的无关升级回退（只留 happy-dom 及其 7 个传递依赖条目）；`bun install --frozen-lockfile` 实测 no changes，全量复跑 164 pass。
  8. 验证实跑（数字见证据节）：tsc 18:36:58 exit 0；lint 18:37:18（0 errors/1 warning，与 base 8715c0e 第一轮实测基线同一条 docs/page.tsx no-img-element 警告，新增 0）；bun test 18:37:26（164 pass/0 fail/20 文件，较第二轮 150 净增 14）；build 18:37:51 exit 0。ponytail-review 第三轮：Lean already. Ship.（唯一发现是 bun add 夹带，已当场还原）；可读性复核通过。

- 2026-11-14 18:57:00–19:14:10 +08:00 主 agent：第四轮。18:57:00 第四轮开始（会话头）；18:57:30–19:04:31 读代码与复现（Researchy 18:56 指出的阻塞：isValidIanaTimeZone 依赖 supportedValuesOf 列表，Node 22 列表只有旧名且无 "UTC"，Bun 恰好相反；主 agent 已在 Node 22.20.0 复现）。事实与步骤：
  1. 19:04:31 任务重开（done→planned，保留旧完成事件）、19:04:32 进入 in-progress；Legacy/Refactoring 两门禁按本轮范围复核：characterized / proceed（结论见门禁结论节）。
  2. 双运行时探针（实现前，为测试期望值取独立事实源）：Node 22.20.0 构造器把 Asia/Kolkata→Asia/Calcutta、Europe/Kyiv→Europe/Kiev、Asia/Ho_Chi_Minh→Asia/Saigon 解析为旧名且两种写法都接受；Bun 1.4.4 两种写法原样保留；两运行时对 GMT+8/UTC+8/GMT+08:00/UTC-5/空串/超长串均抛 RangeError，对 "+08:00"/"-05:00" 构造成功，对错误大小写经大小写不敏感查找构造成功。supportedValuesOf 实测：Node 缺 Asia/Kolkata、Europe/Kyiv、UTC；Bun 缺 Asia/Calcutta、Europe/Kiev。
  3. 测试先行：先红 19:06:46（15 pass/2 fail——timezone.test.ts 新「新旧写法均接受」用例在 Bun 下 Asia/Calcutta 被列表法拒绝；TimezoneSection.test.tsx 新建用例中已存 Asia/Kolkata 不在打桩列表时 select.value 为 ""，即显示为自动检测）→ isValidIanaTimeZone 改构造器判定（+/- 开头偏移写法显式拒绝；构造抛错即拒；解析名与输入仅大小写不同即拒，新旧写法名差异放行）、TimezoneSection 在已存值缺席列表时向选项追加该值 → 转绿 19:07:21（两文件 17 pass/0 fail）。
  4. 全量首跑暴露 happy-dom 重复注册（两个组件测试文件同进程各 register 一次，ProfilePageClient.test.tsx 报 already registered）：两文件统一改 isRegistered 守卫后全量转绿。
  5. 双运行时复验（转绿后执行）：同一份探针脚本直接 import 生产文件 web/src/lib/timezone.ts，Node v22.20.0 与 Bun 1.4.4 各跑一遍，27 个值判定逐行一致（明细见证据节）；探针脚本验毕即删。
  6. 验证实跑（数字见证据节）：tsc 19:08:00 exit 0；lint 19:08:17（0 errors/1 warning，与 origin/main 8715c0e 第一轮实测基线同一条 docs/page.tsx no-img-element 警告，新增 0）；bun test 19:10:12 全量 165 pass/0 fail；build 19:10:25 exit 0。本轮未启动 next 冒烟：改动仅限校验纯函数与设置页下拉框选项列表，路由行为由既有路由测试与双运行时探针钉住，无新请求面；无遗留进程。
  7. 接受集变化记录：第三轮拒绝的别名（US/Eastern、Asia/Chungking、Asia/Chongqing、GMT）本轮起接受——新旧写法必须同接受，而无列表时无法区分「改名对」与「历史别名」；落库存提交写法原文、显示侧有下拉框补齐，别名值在归日计算上与目标名等效。PATCH 与 ?tz 仍共用同一函数，接受集变化两处同时生效。
- 2026-11-14 19:18:20 +08:00 主 agent：第五轮开始（会话头）；19:18:32–19:28:10 读代码、核对阻塞；19:28:10 任务重开（Reviewy 19:00 对 1485fc4 判 NOT CLEAN，三条阻塞：A 时区校验改为三规则＋通过后存运行时 resolvedOptions 规范化写法、加 node --test 用例与 package.json 脚本、下拉框两侧先换算同写法再匹配；B ProfilePageClient.test.tsx afterAll 补 GlobalRegistrator.unregister() 并以 bun test --randomize --seed=3475213529 复验零挂；C task.md 时间戳与「未迁移库登录」措辞更正、模型表补第三/四轮）。可选项：route.test.ts 两条 PATCH 测试的顺序依赖顺手修。Legacy/Refactoring 两门禁按本轮范围复核：characterized / proceed（结论见门禁结论节）。
- 2026-11-14 19:28:10–19:44:10 +08:00 主 agent：第五轮实现与验证，全部实测（各步时钟时间为命令发出时刻，逐条输出留存于会话记录）：
  1. 基线复现：`bun test --randomize --seed=3475213529` 在改动前实跑 159 pass/6 fail——middleware 3 条与 publicProfileData 1 条系 happy-dom 全局泄漏，route.test.ts 2 条 PATCH 系 updateValues 队列未清空。
  2. 先红后绿（补记，按会话记录还原）：先红 19:35:23（实现前首跑，早于 19:35:45 首次改实现——timezone.test.ts 校验块改写后报 normalizeIanaTimeZone 未导出（1 error）；route.test.ts 新「存规范化写法」用例得 400（1 fail/8 pass）；TimezoneSection.test.tsx 新「旧名匹配新名选项」用例下 select 仍追加原文（1 fail/1 pass）；node 用例 import 失败（fail 1））；转绿 19:38:30（全量 167 pass/0 fail）。19:43:38 那次是以 git stash 暂存实现文件的事后重验（复证红 1 error，恢复后同刻转绿 timezone.test.ts 16 pass、node 用例 2 pass），仅作补充。
  3. 阻塞 A 实现：`isValidIanaTimeZone` 更名重写为 `normalizeIanaTimeZone`——三规则全满足（不以 +/- 开头且非 UTC±/GMT± 写法；为 "UTC" 或带 "/"；Intl 构造器能构造）才返回 resolvedOptions().timeZone 的写法，否则 null。PATCH 存规范化写法并原样返回；?tz 共用同一函数（规范化结果只用于归日计算，等价）。EST/GMT 由通过转拒绝（无 "/" 且非 "UTC"），asia/shanghai 由拒绝转接受并存 Asia/Shanghai；Etc/GMT+8 带 "/" 仍接受。
  4. 下拉框：TimezoneSection 新增 canonicalTimeZoneName 辅助，已存值与每个选项先经本运行时 resolvedOptions 换算成同一写法再比；换算后仍无匹配才追加原值 option 兜底。组件测试用打桩的 resolvedOptions 模拟会规范化别名的浏览器（Bun 1.4.4 实测不规范化改名对，见证据节），旧名 Asia/Calcutta 匹配上新名选项 Asia/Kolkata，不再显示为自动检测。
  5. node 用例：web/scripts/test-timezone-node.mjs 直接 import 生产文件 src/lib/timezone.ts（Node 22.20.0 类型剥离实测可用，无需新包），node --test 跑同一接受/拒绝集；web/package.json 加 test:node 脚本；CI Web job 加 setup-node@22 与「Time zone validation under Node」一步。
  6. 阻塞 B 与可选项：ProfilePageClient.test.tsx afterAll 加 GlobalRegistrator.unregister()；route.test.ts afterEach 加队列与 updateValues 清空、publicProfileData.test.ts 加 afterEach 队列清空（400 用例入队不消费的遗留会串扰下一条）。首轮修复后扫 20 个 seed 发现 11/13/29 仍挂（21/11/11 fail）——bun 按文件逐个加载执行，TimezoneSection.test.tsx 在 ProfilePageClient 之后跑时其模块守卫会重新注册且无人注销，后续文件被污染；给 TimezoneSection.test.tsx afterAll 加带 isRegistered 守卫的 unregister 后 20 个 seed 全绿。
  7. resolveViewerTimeZone/detectBrowserTimeZone 保持返回原值（校验只决定是否采纳）：服务器落库时已规范化，客户端不再改写；ProfilePageClient 钉扎的「设置胜过检测」用例在冻结 resolvedOptions 的时钟打桩下仍通过。
  8. 验证实跑（数字见证据节）：tsc 19:40:34 exit 0；lint 19:40:38（0 errors/1 warning，与 origin/main 8715c0e 基线同一条 docs/page.tsx no-img-element，新增 0）；bun test 转绿 19:38:30 全量 167 pass/0 fail；randomize 指定 seed 19:38:36 167 pass/0 fail，另 19 个 seed 分批全绿（末轮 20 seed 19:40:13）；node 用例 2 pass/0 fail（19:36:40 起随组合命令复跑）；build 19:40:53 exit 0；19:41:35 收尾组合复跑（build exit 0、全量与指定 seed 各 167/0、node 2 pass）。本轮未启动 next 冒烟：接受集变化由路由测试与 Node 实跑钉住，无新请求面；无遗留进程。
  9. ponytail-review 第五轮：diff 逐文件过一遍——normalizeIanaTimeZone 三个调用方全用到返回值、canonicalTimeZoneName 唯一消费方是下拉框匹配、无新依赖（node 用例用 node:test/node:assert 与既有生产文件）、无投机抽象。结论：Lean already. Ship. 可读性复核：手写改动逐段复读，注释说明三规则与双运行时差异；vendor/generated 未动。
- 2026-11-14 20:05:09–20:36:50 +08:00 主 agent：第六轮（补记，按会话记录还原；Reviewy 20:04 复审：代码全部过关，仅剩 task.md 文字阻塞 C′，另两条可选；仅改本文件与 .github/workflows/ci.yml，不动其它代码）。20:05:09 第六轮开始（会话头）；20:05:30–20:27:39 读 task.md 与第一至五轮会话记录、逐条核对时间；其间 20:14:19–20:24:18 会话无写入（按会话记录时间戳）；20:27:58 起改文件；20:36:50 结束。事实与步骤：
  1. 20:27:58 任务重开（done→planned，保留旧完成事件）并同刻进入 in-progress；Book Gate 本轮无新触发（无生产代码变更），release-readiness 维持 blocked。
  2. 阻塞 C′（三条均对照会话记录只读核实，会话时间 UTC 加 8 小时换算，会话文件路径不写入本文件）：①第五轮开始条目改为 19:18:20 第五轮开始（会话头）、19:18:32–19:28:10 读代码与核对阻塞、19:28:10 任务重开；②第四轮进度补开头「18:57:00 第四轮开始（会话头）；18:57:30–19:04:31 读代码与复现」；③第五轮先红后绿写具体时间并标「补记，按会话记录还原」：先红 19:35:23（timezone、route、TimezoneSection 测试与 node 用例都挂，早于 19:35:45 首次改实现），转绿 19:38:30（167/0）；19:43:38 那次为 git stash 事后重验，仅作补充。进度第 8 条与证据节中沿用 19:41:35 旧口径的时间一并改正（该时刻实为收尾组合复跑）。
  3. 可选项一（模型表口径统一）：第一、二、四轮起止改按会话头核对改正（17:19:59–18:00:15、18:01:50–18:07:00、18:57:00–19:18:14；第三轮 18:10:44–18:48:46 原本即会话头口径，不变），补第五轮一行（回复 88、工具调用 130、工具报错 2，19:18:20–19:54:12，计数对会话文件只读核对），注脚写明口径「起止为会话头时间」、第六轮计数结束后才有。
  4. 可选项二（CI Node 版本）：.github/workflows/ci.yml Web job 的 setup-node node-version 由 22 改为 24（与线上 Vercel 一致），上方注释同步补一句；证据节写明。
  5. 验证实跑（数字见证据节）：bun run test:node 20:31:44（本机无 Node 24，用现有 Node v22.20.0 跑并如实记录版本；pass 2/fail 0）；bun test 20:31:50 全量 167 pass/0 fail；bunx tsc --noEmit 20:31:56 exit 0。禁用词与本机路径扫描零命中（见证据节）。不提交、不推送。
- 2026-11-14 20:37:18–20:42:26 +08:00 主 agent：第七轮（仅改本文件文字记录；起止按会话记录首末时间；原记 20:37:53–20:41:24 的起为本会话首个实测时间戳、止为本条目写入时刻，补记，按会话记录还原）。第六轮进度条目「20:27:58–20:32:30」缺开头，与 Reviewy 对第四、五轮所提问题同类：按会话记录（只读核对，UTC 加 8 小时换算，路径不写入本文件）还原为 20:05:09 第六轮开始（会话头）、20:05:30–20:27:39 读 task.md 与第一至五轮会话记录、逐条核对时间（其间 20:14:19–20:24:18 会话无写入，按会话记录时间戳）、20:27:58 起改文件、20:36:50 结束，条目标「补记，按会话记录还原」；模型表补第六轮一行（回复 48、工具调用 56、工具报错 1，2026-11-14 20:05:09–20:36:50 +08:00，计数已对会话文件只读核对一致），注脚同类口径顺延为第七轮；frontmatter updated_at 同步。第六轮未提交改动原样保留，不提交、不推送；状态无迁移，状态事件表不新增行。改后禁用词与本机路径扫描零命中。

## 取舍说明

1. **API 增加 `tz` 查询参数**：week/month 的行过滤发生在服务器，看的人「今天」的行在 UTC 口径范围外时客户端无从补回，只能让服务器按看的人时区算 periodRange 与滚动区间；`tz` 非法时返回 400（显式失败好过静默落回 UTC 的误归日）。all 期间本可纯前端重算（响应含全部贡献行），但三档期间统一走「挂载后按看的人时区重取一次」的单一路径，比分两套逻辑更可读。
2. **首屏维持服务器区间**：u 页 ISR 共享缓存不含看的人会话，按 Cookie 分叉缓存会让每个时区各存一份页面；首屏用 UTC 区间、挂载后修正，沿用 ProfilePageClient 既有「晚些再声称今天」的安全方向惯例。
3. **UTC 用户零变化**：看的人时区的今日与服务器响应的 `chartRange.end` 一致时客户端不发重取请求；UTC 看的人两值恒等，渲染路径与现状逐帧一致。
4. **徽章与 embed SVG 不动**：它们是服务器渲染的静态图片，没有看的人浏览器可言，issue 三条改动与四条验收均未涉及。
5. **榜单「today」期间不动**：同属服务器 UTC 口径，但 issue 范围是个人页图表与今天/昨天卡片；改榜单属于另一处行为变更，留待单独提出。
6. **timezone 用 text 列、应用层校验**：IANA 时区库随系统更新，数据库枚举会把合法值挡在库外；校验集中在 `normalizeIanaTimeZone`（第五轮起三条规则同时满足才接受：不以 +/- 开头且非 UTC±/GMT± 这类偏移写法；为 "UTC" 或名字里带 "/"——EST、GMT 这类无斜杠缩写因此拒绝；`Intl.DateTimeFormat` 能构造。通过则返回并存本运行时 resolvedOptions().timeZone 给出的写法——Node 上 Asia/Kolkata 与 Asia/Calcutta 都存 Asia/Calcutta，asia/shanghai 存 Asia/Shanghai；不再用 `Intl.supportedValuesOf('timeZone')` 列表——Node 22 列表只有旧名且无 "UTC"，Bun 恰好相反，任何单边列表都会拒绝另一运行时接受的用户）。服务端写入与个人页 `tz` 参数同源。
7. **会话响应携带 timezone 而非新拉取**：个人页本就在挂载后请求 `/api/auth/session`（设备列表用），在其中带上看的人 timezone 避免第二次往返；未登录时 `user` 为 null，自然落到浏览器检测值。
8. **迁移只进仓库不执行**：本机无数据库实例，授权边界亦如此；迁移正确性由 drizzle-kit 生成器、meta 快照与代码评审承担，生产执行需用户另行批准。这个取舍成立的前提是部署管道本身控住时机：合入不触发自动部署，预览构建不跑迁移，生产构建内迁移先于新 web 代码生效，所以「先生成的迁移文件」不会在未获批准时落到任何数据库。代价是预览环境不能用于本 PR 时区设置的验收——预览库没有 timezone 新列时，设置接口读写返回 503；第三轮起会话解析（getSession）缺列降级 null 不报错，但密码登录、设备轮询、团队相关查询仍整行查 users，未迁移的库上仍会出错，其余接口仍需先迁移再上线。

## 模型与调用

主 agent 用 k3:max，密钥从环境变量 KIMI_API_KEY 读取。

| 角色 | 模型 | 密钥来源 | 调用次数 |
|---|---|---|---|
| 第一轮主 agent | k3:max（kimi-code/k3） | 环境变量 KIMI_API_KEY | 回复 118、工具调用 150、工具报错 5（2026-11-14 17:19:59–18:00:15 +08:00） |
| 第二轮主 agent | k3:max（kimi-code/k3） | 环境变量 KIMI_API_KEY | 回复 16、工具调用 17、工具报错 0（2026-11-14 18:01:50–18:07:00 +08:00） |
| 第三轮主 agent | k3:max（kimi-code/k3） | 环境变量 KIMI_API_KEY | 回复 91、工具调用 131、工具报错 7（2026-11-14 18:10:44–18:48:46 +08:00） |
| 第四轮主 agent | k3:max（kimi-code/k3） | 环境变量 KIMI_API_KEY | 回复 42、工具调用 61、工具报错 2（2026-11-14 18:57:00–19:18:14 +08:00） |
| 第五轮主 agent | k3:max（kimi-code/k3） | 环境变量 KIMI_API_KEY | 回复 88、工具调用 130、工具报错 2（2026-11-14 19:18:20–19:54:12 +08:00） |
| 第六轮主 agent | k3:max（kimi-code/k3） | 环境变量 KIMI_API_KEY | 回复 48、工具调用 56、工具报错 1（2026-11-14 20:05:09–20:36:50 +08:00） |
| 第七轮主 agent | k3:max（kimi-code/k3） | 环境变量 KIMI_API_KEY | 回复 8、工具调用 14、工具报错 0（2026-11-14 20:37:18–20:42:26 +08:00） |

统计口径：回复 = 该轮会话文件 assistant 消息条数；工具调用 = tool_execution_start 条数；工具报错 = 会话文件中标记为报错（"isError":true）的工具结果条数（含命令非零退出）。每轮一行取该轮会话文件的最终计数；起止为会话头时间（起 = 会话头记录时间，止 = 会话退出记录时间，均为 UTC 加 8 小时换算）。各轮均已结束，计数为最终值。

未启动子代理；密钥不写入本文件与任何产物。

## 门禁结论

### DDIA Data Design Review

- Status: confirmed
- Data owner and source of truth：users 表新增 `timezone`（text、可空、默认 null）归 web api 所有，是唯一存储点；null 表示自动检测，实际生效值由客户端按「设置优先、浏览器检测兜底」解析，浏览器检测值本身不落库。
- Write / read / async / failure paths：写——`/api/settings/timezone` PATCH（仅浏览器会话＋CSRF Origin 门），合法 IANA 才写库，非法 400 不写；读——设置页 GET 同一路由，个人页经 `/api/auth/session` 顺带读出；无异步路径；写失败返回 500 且设置页显示错误文案，不产生半更新。
- Consistency model：单行单字段更新，读己之写（PATCH 响应即新值）；个人页 ISR 缓存不含该字段，无时滞一致性问题。
- Idempotency / ordering / retry / deduplication：PUT 语义整字段覆盖，重复 PATCH 幂等；无并发写竞态（单用户单字段）。
- Schema / migration / backfill / rollback / replay：`0028_add_user_timezone.sql` 为 `ALTER TABLE users ADD COLUMN timezone text`，可空无默认，存量行天然为 null（自动检测），无需 backfill；回滚即删列，无数据转换；迁移不在本仓验证环境执行，生产执行需用户另行批准。执行时机由部署构建控住：仅 production 构建内先迁移后构建，预览构建不迁移，合入 PR 本身不动线上数据库（vercel.json 原文核对结论见授权节与 Release Readiness 评审）。
- Observability and repair：无派生数据，无修复路径；非法值在写入侧已被 400 挡住。
- Required tests：路由测试覆盖 401/400/写入/清空/读取与规范化落库；`normalizeIanaTimeZone` 边界用例（bun 与 node 两侧）。
- 第三轮修订（2026-11-14）：读路径与失败模式收紧——getSession 主查询只取消费列（id/username/displayName/avatarUrl/bannedAt），timezone 由一条单独查询读取；列不存在（未跑 0028）、查询出错、查无行三种情况一律降级 null（自动检测），会话解析（getSession）不报错（session.test.ts 四条用例钉住）；密码登录、设备轮询、团队相关查询仍整行查 users，未迁移的库上仍会出错。设置接口 GET/PATCH 在列不存在时返回 503 并指明先跑 db:migrate（Postgres 42703 或缺列报文匹配），其余写失败仍为 500、不产生半更新。校验收紧只改应用层接受集，Schema/迁移不变。Status 维持 confirmed。
- 第四轮修订（2026-11-14）：校验实现由 supportedValuesOf 列表改为构造器判定（新旧 IANA 写法与 UTC 在 Node/Bun 均接受；+/- 开头偏移写法、UTC+8/GMT+8 这类、空串、超长串、仅大小写差异仍拒绝），仍只改应用层接受集，Schema/迁移/读写路径不变；落库为提交写法原文，无规范化改写。Status 维持 confirmed。
- 第五轮修订（2026-11-14）：校验并入 `normalizeIanaTimeZone` 并在通过时返回本运行时 resolvedOptions().timeZone 的写法——PATCH 落库由提交原文改为该规范化写法（Node 上 Asia/Kolkata 与 Asia/Calcutta 都落 Asia/Calcutta，asia/shanghai 落 Asia/Shanghai）；asia/shanghai 这类仅大小写差异的写法由拒绝转接受，EST、GMT 这类无斜杠缩写由（第四轮）接受转拒绝；?tz 共用同一函数，规范化结果只参与归日计算，等价。Schema/迁移/读写路径不变。Status 维持 confirmed。

### Legacy Change Safety Review

- Status: characterized
- Behavior to change：`getPublicProfileResponse` 的 chartRange 与 week/month periodRange 由服务器 UTC 今日改为按 `tz` 参数（看的人时区）今日；无 `tz` 时维持 UTC。ProfilePageClient 挂载后按看的人时区重取并以该时区算今天/昨天卡片。
- Behavior to preserve：无 `tz` 参数时响应与现状逐字节一致（UTC 用户行为不变）；`dateRange` 的全量语义（最早/最晚提交日期）、贡献强度分档、排名、隐私门、封禁响应、重定向、缓存标签与 revalidate、embed/badge 路径全部不动。
- Current reproduction evidence：issue #21 实测记录（2026-11-14 07:28 UTC+8，chartRange.end=2026-11-13 而 dateRange.end=2026-11-14）；源码确认两个 range 函数用 `toISOString().slice(0, 10)`；现有 `publicProfileData.test.ts` 为本路径既有安全网。
- Safety net：现有 126 条 bun test（含 publicProfileData、i18n 覆盖）＋本任务新增 timezone 纯函数、range 函数固定时刻、路由 tz 参数与设置路由测试；tsc/lint/build 防回归。
- Hidden dependencies / seam：`loadPublicProfileForPage` 的 `unstable_cache` 不传 `tz`，缓存键不变、共享缓存仍全看的人通用；`getViewerAccess` 读会话判 owner 的路径不受影响；无新 seam。
- Validation plan：先红后绿，再跑项目四条 web 验证（结果见证据节）。
- Review mode: normal
- 第三轮复核（2026-11-14）：Status: characterized（normal）。Behavior to change：`isValidIanaTimeZone` 接受集收窄为规范名集合＋UTC（offset 形式 "+08:00"/"GMT+8"、非规范大小写、别名 Asia/Chungking/US/Eastern 由通过转 400，PATCH 与 `?tz` 同源生效）；getSession 不再整行查 users（主查询收窄至消费列），timezone 单独查询、缺列/出错/无行降级 null；设置接口缺列时由笼统 500 转为 503 明确错误。Behavior to preserve：规范名与 UTC 照旧通过、UTC 用户不变、SessionUser 既有字段与封禁判断不变、submit token 路径不变、设置接口 401/400/成功响应不变。Current reproduction evidence：bun 实测构造器校验对 "+08:00"/"asia/shanghai"/Asia/Chungking/US/Eastern 返回 true；diff 8715c0e..HEAD 显示整行 select 经 schema 卷入 timezone 列；第二轮已记录未迁移库会话查询必崩模式。Safety net：既有 150 条＋本轮新增 14 条（校验 4、session 6、路由缺列 2、组件层 2），全量 164 pass。Validation plan：先红后绿（红 18:28:37/18:28:44/18:28:49 → 绿 18:29:55）＋四条 web 命令全绿（证据节）。
- 第四轮复核（2026-11-14）：Status: characterized（normal）。Behavior to change：`isValidIanaTimeZone` 接受集由 supportedValuesOf 列表＋UTC 改为构造器判定——新旧写法对（Asia/Kolkata↔Asia/Calcutta、Europe/Kyiv↔Europe/Kiev、Asia/Ho_Chi_Minh↔Asia/Saigon）、可构造别名（US/Eastern、Asia/Chungking）、GMT、Etc/GMT+8 由拒绝转接受；+/- 开头偏移写法与仅大小写差异写法维持拒绝。TimezoneSection：已存值不在运行时 supportedValuesOf 列表时向 select 追加该值的 option（此前无匹配项，select 显示为自动检测）。Behavior to preserve：PATCH 与 `?tz` 同源 400 拒绝面（非 IANA 串、offset 写法、错误大小写仍拒）；落库为提交写法原文；UTC 用户行为不变；浏览器检测兜底不变；session 降级与设置接口 503 不变。Current reproduction evidence：Node 22.20.0 实测 supportedValuesOf 缺 Asia/Kolkata、Europe/Kyiv、UTC（Researchy 18:56 阻塞复现）；Bun 1.4.4 实测缺 Asia/Calcutta、Europe/Kiev。Safety net：既有 164 条＋本轮净增 1 条（TimezoneSection 组件测试；校验测试以新旧写法接受用例替换别名拒绝用例，数量不变），全量 165 pass。Validation plan：先红后绿（红 19:06:46 → 绿 19:07:21）＋四条 web 命令全绿与双运行时探针（证据节）。
- 第五轮复核（2026-11-14）：Status: characterized（normal）。Behavior to change：`isValidIanaTimeZone` 更名重写为 `normalizeIanaTimeZone`——三规则同时满足（不以 +/- 开头且非 UTC±/GMT± 偏移写法；为 "UTC" 或带 "/"；构造器能构造）才返回本运行时 resolvedOptions().timeZone 的写法，否则 null。接受集：EST、GMT 由（第四轮）接受转拒绝，asia/shanghai 这类仅大小写差异的写法由拒绝转接受。落库由提交原文改为规范化写法。TimezoneSection：已存值与选项先经本运行时 resolvedOptions 换算成同一写法再比，仍无匹配才追加原值 option（第四轮为缺席即追加原文）。测试顺序无关化：route.test.ts 与 publicProfileData.test.ts 的 mock 队列 afterEach 清空（400 用例入队不消费的遗留曾串扰下一条），ProfilePageClient.test.tsx afterAll 补 GlobalRegistrator.unregister()、TimezoneSection.test.tsx 补带守卫的 unregister（bun 按文件逐个加载执行，后者先跑时无人注销会污染后续文件，seed 11/13/29 实测 21/11/11 fail）。Behavior to preserve：PATCH 与 `?tz` 同源 400 拒绝面（非 IANA 串、+/- 与 UTC±/GMT± 偏移写法、空串、超长串仍拒）；UTC 用户行为不变；浏览器检测兜底不变（resolveViewerTimeZone/detectBrowserTimeZone 校验后仍返回原值）；session 降级与设置接口 503 不变；GET 响应形状不变。Current reproduction evidence：Reviewy 19:00 对 1485fc4 的 NOT CLEAN 三条阻塞；改动前 `bun test --randomize --seed=3475213529` 实测 159 pass/6 fail。Safety net：165 条基础上净增 2 条（路由规范化落库 1、下拉框兜底 1；校验块改写等数替换），全量 167 pass；--randomize 指定 seed 与另 19 个 seed 全绿；node --test 用例在 Node 22.20.0 直引生产文件 2 pass。Validation plan：先红后绿（红复证 19:43:38 → 绿同刻，明细见进度节与证据节）＋tsc/lint/bun test/randomize/node/build 六条全绿。

### Refactoring Review

- Status: proceed
- Review mode: normal
- Existing-code scope：`publicProfileData.ts`（两个 range 函数加时区参数、路由解析 tz）、`lib/auth/session.ts`（SessionUser 加字段）、`ProfilePageClient.tsx`（data 改 state、挂载后重取）、`SettingsClient.tsx`（挂新分节）、docs 页（新 Section）。
- Behavior that must remain unchanged：见 Legacy 节 preserve 清单。
- Structural friction：两个 range 函数已带 `now` 参数且为纯函数，加时区参数即最小扩口；设置分节与 docs Section 均有现成样板（PrivacySection、privateTitle Section），不需要结构调整。
- Decision and smallest safe step：no refactor needed；不夹带清理。
- Safety net and validation：同 Legacy 节。
- Deferred refactors：无。
- 第三轮复核（2026-11-14）：Status: proceed（normal）。范围：`timezone.ts`（校验实现替换）、`session.ts`（select 形状收窄＋一条兜底查询）、设置路由（缺列判定函数＋两个 catch 分支）、三个测试文件与一个组件测试文件。no refactor needed；无延期项。
- 第四轮复核（2026-11-14）：Status: proceed（normal）。范围：`timezone.ts`（校验实现替换）、`TimezoneSection.tsx`（zones memo 追加已存值）、`timezone.test.ts`（接受集用例替换）、新建 `TimezoneSection.test.tsx`、`ProfilePageClient.test.tsx`（happy-dom 注册加 isRegistered 守卫）。no refactor needed；无延期项。
- 第五轮复核（2026-11-14）：Status: proceed（normal）。范围：`timezone.ts`（校验函数更名重写为 normalizeIanaTimeZone）、两个调用方（route.ts 存规范化写法、publicProfileData.ts tz 参数）、`TimezoneSection.tsx`（canonicalTimeZoneName 辅助与 selectedZone/options 匹配）、四个测试文件、新建 `web/scripts/test-timezone-node.mjs` 与 web/package.json test:node 脚本、ci.yml Web job 加一步。no refactor needed；无延期项。

### Release Readiness Review

- Status: blocked
- Production path and affected users / systems：`/api/users/[username]`（公共读，新增 tz 参数）、`/api/settings/timezone`（新，仅浏览器会话写）、`/api/auth/session`（响应多一个字段）、users 表新列（迁移未执行）、u 页挂载后可能多发一次带 tz 的重取。
- Failure modes and safeguards：非法 tz → 400（写设置）/400（读个人页），不静默落回 UTC；重取失败保留 ISR 首屏数据；看的人时钟偏斜由一次性重取闸挡住循环；访客无会话落到浏览器检测值；`PRIVATE_CACHE_CONTROL` 挡住设置接口响应进共享缓存；未迁移库上 getSession 的 timezone 读取降级 null（console.warn 留痕），设置接口读写返回 503 明确错误。
- Capacity / backpressure / limits：非 UTC 看的人在跨日期窗口每个个人页最多多发一次 API 请求；u 页 ISR 缓存键不变，无缓存分裂；设置接口单行单字段更新。
- Observability / alerts / runbook：设置读写失败走既有 console.error 通道；无新指标。
- Rollout / migration / rollback / cleanup：**第三轮起 getSession 不再整行查 users：主查询只取消费列，timezone 单独查询且失败（含缺列）降级 null，未跑 0028 的库上会话解析（getSession）不报错；但密码登录、设备轮询、团队相关查询仍整行查 users，未迁移的库上仍会出错，其余接口仍需先迁移再上线。未迁移时设置接口的时区读写返回 503（明确提示先跑 db:migrate），时区特性本身不可用。** 自部署文档（docs/deploy/self-host-production 中英双语）上线步骤已注明：升级到含 0028_add_user_timezone 的版本前先手动跑 db:migrate。迁移本身是纯加列，旧代码对新列无感，回滚 web 即可。迁移执行需用户另行批准，不在本会话授权内。时机与次序按 web/vercel.json 原文核对：buildCommand 为「`if [ "$VERCEL_ENV" = "production" ]; then ... bun run db:migrate; fi && bun run build`」，即仅正式环境构建才先跑迁移再编译，同一次构建内迁移先于新 web 代码生效；`git.deploymentEnabled.main` 为 false，main 分支不自动部署，合入本 PR 本身不动线上数据库；真正执行迁移的是之后那次手动正式部署。预览部署不跑迁移，预览库缺 timezone 列时设置接口返回 503，预览环境的时区设置功能不能作为验收依据（会话解析不受影响，但密码登录等整行查 users 的接口仍依赖迁移）。
- Required validation and result：第一轮 tsc、lint（对 base 无新增）、bun test 150、build 全绿，未登录 401、非法 tz 400、docs 三语小节冒烟通过；第三轮 tsc、lint（对 base 无新增）、bun test 164、build 全绿，别名/offset tz 400 与规范名放行冒烟通过；第四轮 tsc、lint（对 base 无新增）、bun test 165、build 全绿，Node 22.20.0 与 Bun 1.4.4 双运行时探针 27 值判定逐行一致；第五轮 tsc、lint（对 base 无新增）、bun test 167、bun test --randomize --seed=3475213529（167 pass/0 fail，另 19 个 seed 全绿）、node --test 用例（Node 22.20.0 直引生产文件，2 pass）、build 全绿，Node 与 Bun 接受/拒绝集一致、存储写法各随本运行时 resolvedOptions；第六轮 CI Web job 的 setup-node 由 Node 22 改 24（与线上 Vercel 一致），tsc、bun test 167、node 用例（本机 Node v22.20.0，2 pass）复跑全绿；结果见证据节。
- Optional checks, accountable owner acceptance, and residual risk：带真实数据库的浏览器级 E2E（UTC+8 00:30 实景）本机无 postgres 服务与 docker，无法运行，由路由级测试与区间函数固定时刻测试覆盖归日逻辑；CI、合并、部署与生产迁移执行不在本会话授权内。

### ponytail-review 结论

Lean already. Ship. 五个纯函数全部被引用、无新依赖、无投机抽象；过程中删掉一处会导致 week 视图误空窗的守卫（见进度节第 9 条）。

第三轮（2026-11-14）：Lean already. Ship. 校验函数由构造器试抛改为集合查找（更短）；session 拆分查询为评审要求的必需兜底；唯一发现是 `bun add` 夹带的无关改动（package.json 空行格式、tokens-cli 1.0.2→1.0.4），已当场还原，锁文件净增仅 happy-dom 及其传递依赖（组件测试必需的 DOM 环境，devDep）。

第五轮（2026-11-14）：Lean already. Ship. `normalizeIanaTimeZone` 三个调用方（PATCH、?tz、客户端校验）全用到返回值；`canonicalTimeZoneName` 唯一消费方是下拉框匹配；node 用例只用 node:test/node:assert 直引既有生产文件，无新依赖、无投机抽象；测试顺序无关化只加 afterEach 清空与 afterAll 注销，无新测试基建。

## 证据

全部在分支 fix/21-timezone-setting 工作区实跑，未提交：

- `cd web && bunx tsc --noEmit`：exit 0，无输出。
- `cd web && bun run lint`：0 errors、1 warning（docs/page.tsx 的 `@next/next/no-img-element` img 提示，第 354 行）。base 对比：在 8715c0e 的临时 git worktree 上独立安装同一依赖树实跑 `bun run lint`，同为 0 errors、1 warning（同文件同规则，base 第 340 行；行号差为本 PR 新增 docs 小节所致）。新增 warning 数：0。对比用 worktree 已移除。
- `cd web && bun test`：150 pass、0 fail、5535 expect()，18 个测试文件（base 126 pass；本 PR 净增 24：timezone.test.ts 12、设置路由 6、publicProfileData 追加 6——其中 payload 形状保护断言先绿）。i18n 覆盖测试继续通过，三语词条齐备由它保证。
- `cd web && bun run build`：exit 0，编译成功，`/api/settings/timezone` 出现在路由清单（ƒ 动态）。
- TDD 红绿记录：timezone.test.ts 先红（模块不存在，1 error）→ 12 pass；publicProfileData 时区用例先红 5 fail（range 函数未导出、tz 参数被忽略、非法 tz 得 200）→ 转绿；设置路由先红（路由不存在）→ 6 pass。week/month/all 三档 range 的期望值为手写字面量（2026-11-13T16:30Z 在 Asia/Shanghai 为 2026-11-14、America/New_York 在 2026-11-14T04:30Z 为 2026-11-13、UTC 不动、2028-02-29 收拢 2027-02-28）；路由级期望值用 en-CA 直出独立算法交叉验证。
- 冒烟（next start 于本会话端口 3121，验毕已关闭）：`GET /api/settings/timezone` 未登录 → 401 `{"error":"Not authenticated"}`；`GET /api/users/someone?tz=Not/AZone` → 400（校验先于用户查询）；`/docs`、`/zh-CN/docs`、`/ja/docs` 均含 time-zone 小节（id="time-zone"、「时区」、「タイムゾーン」）。
- 未跑且原因：带真实数据库的浏览器级 E2E——本机无 postgres 服务（仅客户端）、无 docker，无法起库；归日逻辑改由路由级 mock 测试与固定时刻纯函数测试覆盖。Rust CLI 未改动，cargo test/clippy 不跑。迁移文件只生成未执行（无本机库，生产执行需用户另行批准）。`test:migrations`（需 DATABASE_URL）同理未跑。
- 冒烟与验证结束后：无本会话遗留 next 进程；lint 对比用临时 worktree 已移除。

第三轮（2026-11-14 18:25–18:47，分支 fix/21-timezone-setting 工作区，基于 HEAD ba9c8d9 的未提交改动）：

- 红：`bun test src/lib/timezone.test.ts` 18:28:37——3 fail/13 pass（"+08:00"、非规范大小写、别名 Asia/Chungking/US/Eastern 被构造器校验放行；"GMT+8"、"UTC+8"、空串、超长串此前已被拒）；`bun test src/lib/auth/session.test.ts` 18:28:44——4 fail/2 pass（timezone 未走单独查询，缺列/出错/无行未降级 null）；`bun test src/app/api/settings/timezone/route.test.ts` 18:28:49——2 fail/6 pass（缺列得笼统 500 而非 503 明确错误）。
- 绿：18:29:55，三文件合计 30 pass/0 fail。
- 组件层测试：`bun test src/app/u/[username]/ProfilePageClient.test.tsx` 18:35:44——2 pass（看别人页面：设置优先于检测、无设置落检测值且不重取；既有行为钉扎，无红阶段）。
- `cd web && bunx tsc --noEmit`：18:36:58 exit 0，无输出。
- `cd web && bun run lint`：18:37:18 exit 0，0 errors/1 warning（docs/page.tsx 354 行 no-img-element）。base 对比：main 未移动（仍为 8715c0e），沿用第一轮临时 worktree 实测基线 0 errors/1 warning（同文件同规则，base 340 行；行号差为本 PR 新增 docs 小节所致）。新增 errors/warnings：0。
- `cd web && bun test`：18:37:26——164 pass/0 fail/5563 expect()/20 文件（第二轮 150 pass/18 文件；净增 14：校验收紧 4、session 6、路由缺列 2、组件层 2）。
- `cd web && bun run build`：18:37:51 exit 0。
- 冒烟（next start 端口 3122，18:43:59 已关闭并删日志）：`?tz=Asia/Chungking` → 400、`?tz=%2B08:00` → 400（收紧生效）；`?tz=Asia/Shanghai`、`?tz=UTC` → 500（通过校验后死于无 DATABASE_URL 的查询，证明规范名仍被接受）；`GET /api/settings/timezone` 未登录 → 401。
- 依赖卫生：`bun install --frozen-lockfile` 实测 no changes（锁文件相对 HEAD 净增仅 happy-dom 及其 7 个传递依赖条目；tokens-cli 无关升级已回退）。
- rtk：not-needed——四条命令均为终端事实型输出，第一、二轮证据均为 native，保持同口径可比。
- BDD：skipped——无 .feature 变更，行为变化由上述测试钉住（本仓 no-.feature 例外）。Cross-repo context：not-needed。API Contract：not-needed（项目无独立 API 规格文件，路由行为由测试钉住）。Mock Strategy：contract-backed（mock 形状来自真实 drizzle 查询结果与 Postgres 42703 错误码）。
- E2E Mode：mock-backed（路由/会话/组件测试）＋ smoke-only（curl 冒烟）。Final Test Report：not-needed（bun test 无持久报告配置，结果即本记录）；Run Summary MD：not-needed；Targeted Rerun：passed；Final Full Rerun：passed（164 pass）。Evidence Source：developer-local；Source Revision：dirty（任务授权不提交）；Environment Alignment：not-needed；Evidence Publication：local-only。
- Playwright CLI：blocked（本机无 postgres/docker，同第一轮）；Chrome DevTools MCP：not-needed（curl 冒烟已覆盖本轮服务端行为面，登录态页面需真实库）。
- 未跑且原因：同第一轮——真实库浏览器级 E2E 与 `test:migrations` 需 DATABASE_URL/本地 postgres，本机皆无；迁移未执行（生产执行需用户另行批准）；Rust CLI 未改动，cargo 未跑。冒烟与验证结束后无本会话遗留 next 进程。

第四轮（2026-11-14 19:04–19:14，分支 fix/21-timezone-setting 工作区，基于 HEAD 1485fc4 的未提交改动）：

- 红：19:06:46 `bun test src/lib/timezone.test.ts src/app/settings/TimezoneSection.test.tsx`——15 pass/2 fail：「新旧写法均接受」用例在 Bun 下 Asia/Calcutta 被列表法拒绝（Node 方向即 Researchy 指出的 Asia/Kolkata、Europe/Kyiv、UTC 被拒）；TimezoneSection 用例 select.value 为 ""（已存 Asia/Kolkata 不在打桩列表时显示为自动检测）。
- 绿：19:07:21，两文件 17 pass/0 fail。全量首跑暴露 happy-dom 重复注册（两个组件测试文件同进程各 register 一次），两文件加 isRegistered 守卫后全量转绿 19:10:12：165 pass/0 fail/21 文件（第三轮 164 pass/20 文件；净增 1：TimezoneSection 组件测试；校验测试以「新旧写法接受」替换「别名拒绝」，数量不变）。
- 双运行时探针（转绿后执行；临时探针脚本直接 import 生产文件 src/lib/timezone.ts，验毕即删）：Node v22.20.0 与 Bun 1.4.4 对 27 个值判定逐行一致——接受 Asia/Kolkata、Asia/Calcutta、Europe/Kyiv、Europe/Kiev、Asia/Ho_Chi_Minh、Asia/Saigon、UTC、Asia/Shanghai、America/New_York、Pacific/Kiritimati、Etc/GMT+8、GMT、US/Eastern、Asia/Chungking；拒绝 +08:00、-05:00、UTC+8、GMT+8、GMT+08:00、UTC-5、asia/shanghai、AMERICA/NEW_YORK、utc、Not/AZone、Asia、空串、200 字符长串。另实测 supportedValuesOf 列表：Node 缺 Asia/Kolkata、Europe/Kyiv、UTC；Bun 缺 Asia/Calcutta、Europe/Kiev——即阻塞根因，两运行时构造器判定结果一致。
- `cd web && bunx tsc --noEmit`：19:08:00 exit 0，无输出。
- `cd web && bun run lint`：19:08:17 exit 0，0 errors/1 warning（docs/page.tsx 354 行 no-img-element）。base 对比：origin/main 仍为 8715c0e（本地 main 引用停在更早的 3277189 未前移，不影响基线口径），沿用第一轮 8715c0e 临时 worktree 实测基线 0 errors/1 warning（同文件同规则）。新增 errors/warnings：0。
- `cd web && bun test`：19:10:12 全量 165 pass/0 fail/5570 expect()/21 文件。
- `cd web && bun run build`：19:10:25 起跑，exit 0。
- 冒烟：本轮未启动 next start——改动仅限校验纯函数与设置页下拉框选项列表，无新请求面；校验行为由路由级既有测试与双运行时探针覆盖。无本会话遗留进程。
- 依赖：无新增（happy-dom 守卫复用第三轮既有 devDep）；探针脚本为临时文件，已删。
- rtk：not-needed——四条命令均为终端事实型输出，前轮证据均为 native，保持同口径可比。BDD：skipped——无 .feature 变更（本仓 no-.feature 例外）。Playwright CLI / Chrome DevTools MCP：not-needed——无页面结构与请求面变化，下拉框行为由组件测试钉住。
- 未跑且原因：同前轮——真实库浏览器级 E2E 与 `test:migrations` 需 DATABASE_URL/本地 postgres，本机皆无；迁移未执行（生产执行需用户另行批准）；Rust CLI 未改动，cargo 未跑。

第五轮（2026-11-14 19:28–19:44，分支 fix/21-timezone-setting 工作区，基于 HEAD 1485fc4 的未提交改动）：

- 基线复现（改动前）：`bun test --randomize --seed=3475213529` 159 pass/6 fail——middleware.test.ts 3 条与 publicProfileData.test.ts 1 条（happy-dom 全局泄漏所致），route.test.ts PATCH 2 条（updateValues 与队列未在 afterEach 清空所致）。
- 红（实现前首跑，补记，按会话记录还原）：19:35:23，早于 19:35:45 首次改实现——timezone.test.ts 校验块改写后报 `normalizeIanaTimeZone` 未导出（1 error）；route.test.ts 新「存规范化写法」用例得 400（1 fail/8 pass）；TimezoneSection.test.tsx 新「旧名匹配新名选项」用例下 select 仍追加旧名原文（1 fail/1 pass）；`bun run test:node` 因导出缺失失败（fail 1）。19:43:38 以 git stash 暂存实现文件复证红（1 error），恢复后同刻转绿（timezone.test.ts 16 pass；node 用例 2 pass）——该次为事后重验，仅作补充。
- Node/Bun 实测：Node v22.20.0 解析 Asia/Kolkata→Asia/Calcutta、Asia/Calcutta→Asia/Calcutta、Europe/Kyiv→Europe/Kiev、Europe/Kiev→Europe/Kiev、asia/shanghai→Asia/Shanghai、UTC→UTC、Etc/GMT+8→Etc/GMT+8（EST→America/Panama、GMT→UTC，均被规则②拒绝）；Bun 1.4.4 同名原样保留（Asia/Kolkata→Asia/Kolkata、Asia/Calcutta→Asia/Calcutta、Europe/Kyiv→Europe/Kyiv、asia/shanghai→Asia/Shanghai 等；EST→EST、GMT→GMT，同样被规则②拒绝）——Bun 不规范化改名对，组件测试因此用打桩的 resolvedOptions 模拟会规范化的浏览器。接受/拒绝集两运行时一致：接受 Asia/Kolkata、Asia/Calcutta、Europe/Kyiv、Europe/Kiev、UTC、asia/shanghai；拒绝 EST、+08:00、-05:00、UTC+8、GMT+8、空串、200 字符长串。存储写法各随本运行时 resolvedOptions（Node 落旧名、Bun 落提交名），断言一律用运行时自身 resolvedOptions 求得，无写死值。
- 绿：`cd web && bun test` 19:38:30 全量 167 pass/0 fail/5576 expect()/21 文件（第四轮 165；净增 2：路由规范化落库 1、下拉框兜底 1）；19:41:35 收尾组合复跑（build exit 0、全量与指定 seed 各 167/0、node 2 pass）。
- `bun test --randomize --seed=3475213529` 19:38:36：167 pass/0 fail。另抽 19 个 seed 分批全绿（末轮 20 seed 19:40:13）——首版修复（只注销 ProfilePageClient）下 seed 11/13/29 仍 21/11/11 fail（TimezoneSection 后跑时其模块守卫重新注册且无人注销，后续文件被 happy-dom 全局污染）；TimezoneSection.test.tsx 补带 isRegistered 守卫的 unregister 后 20 个 seed 全绿。
- `cd web && bunx tsc --noEmit`：19:40:34 exit 0，无输出。
- `cd web && bun run lint`：19:40:38 exit 0，0 errors/1 warning（docs/page.tsx 354 行 no-img-element，与 origin/main 8715c0e 第一轮实测基线同文件同规则，新增 0）。
- `bun run test:node`（node --test web/scripts/test-timezone-node.mjs，Node v22.20.0 直引 src/lib/timezone.ts，类型剥离无新包）：19:36:40 起 pass 2/fail 0（19:41:35 收尾复跑同结果）。
- `cd web && bun run build`：19:40:53 exit 0，/api/settings/timezone 在路由清单（ƒ 动态）。
- CI：Web job 在 Typecheck 与 Build 之间新增 actions/setup-node@v6（node-version: 22）与「Time zone validation under Node」（`cd web && bun run test:node`）一步。
- 冒烟：本轮未启动 next start——接受集与落库写法变化由路由级测试与 Node 实跑钉住，无新请求面；无本会话遗留进程。
- 依赖：无新增；bun.lock 未动。
- rtk：not-needed——六条命令均为终端事实型输出，前轮证据均为 native，保持同口径可比。BDD：skipped——无 .feature 变更（本仓 no-.feature 例外）。Playwright CLI / Chrome DevTools MCP：not-needed——无页面结构变化，下拉框行为由组件测试钉住。
- 未跑且原因：同前轮——真实库浏览器级 E2E 与 `test:migrations` 需 DATABASE_URL/本地 postgres，本机皆无；迁移未执行（生产执行需用户另行批准）；Rust CLI 未改动，cargo 未跑。

第六轮（2026-11-14 20:27–20:33，分支 fix/21-timezone-setting 工作区，基于 HEAD 3bcaa6e 的未提交改动）：

- 改动范围：仅本文件（阻塞 C′ 三条文字更正、模型表口径统一并补第五轮行、本轮进度与本证据块、状态事件）与 .github/workflows/ci.yml（Web job setup-node 的 node-version 由 22 改为 24，注释同步补「Node 24 matches the production runtime version on Vercel」——与线上 Vercel 一致）。不动其它代码。
- 阻塞 C′ 核对：三条均对照会话记录只读核实（会话时间 UTC 加 8 小时换算；会话文件路径不写入本文件）——第五轮会话头 19:18:20、首个动作 19:18:32（git status/log/branch）、任务重开 19:28:10；第四轮会话头 18:57:00、首个动作 18:57:30、任务重开 19:04:31；第五轮先红 19:35:23（timezone 1 error、route 1 fail/8 pass、TimezoneSection 1 fail/1 pass、node fail 1，早于 19:35:45 首次改实现 timezone.ts）、转绿 19:38:30（全量 167/0）；19:43:38 为 git stash 事后重验（复证红 1 error，恢复同刻转绿 16 pass＋node 2 pass），仅作补充。模型表第五轮计数只读核对：assistant 消息 88、tool_execution_start 130、"isError":true 工具结果 2。
- `cd web && bun run test:node`：20:31:44，pass 2/fail 0。本机无 Node 24，用现有 Node v22.20.0 跑并如实记录版本；CI 上以 Node 24 实跑随下次推送才见。
- `cd web && bun test`：20:31:50，167 pass/0 fail/5576 expect()/21 文件。
- `cd web && bunx tsc --noEmit`：20:31:56 exit 0，无输出。
- rtk：not-needed——三条命令均为终端事实型输出，前轮证据均为 native，保持同口径可比。BDD：skipped——无 .feature 变更（本仓 no-.feature 例外）。Cross-repo context：not-needed。API Contract：not-needed。Playwright CLI / Chrome DevTools MCP：not-needed——无页面与请求面变化。Mock Strategy / E2E Mode：not-needed（本轮无测试改动）。
- 未跑且原因：lint 与 build 未复跑——本轮不改任何源码与依赖，ci.yml 不参与 lint/build 输入，沿用第五轮 19:40 实测结论；真实库浏览器级 E2E 与 `test:migrations` 需 DATABASE_URL/本地 postgres，本机皆无；迁移未执行（生产执行需用户另行批准）；Rust CLI 未改动，cargo 未跑。冒烟未启动（无新请求面），无遗留进程。
- 禁用词、本机路径与密钥扫描：20:32 对本文件实跑，零命中（词表与前三轮同一套）；密钥未写入本文件与任何产物。

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-14T17:36:18.000000+08:00 | planned | in-progress | 开工：范围、验收与 Book Gate Plan 已确认，调查完成，DDIA/Legacy/Refactoring 三门禁结论已出 | task.md 已确认范围与授权节、进度节调查结论、门禁结论节；分支 fix/21-timezone-setting，工作区开工前干净，与 main(8715c0e) 一致 |
| 2026-11-14T17:55:41.000000+08:00 | in-progress | checking | 实现与验证完成：TDD 三切片先红后绿、迁移文件生成、设置页与 docs 三语落地、四条 web 验证命令全绿、冒烟通过、ponytail-review 与可读性复核完成 | task.md 进度节 17:36:18–17:55:41 各条、证据节 |
| 2026-11-14T17:55:41.000000+08:00 | checking | done | 验收条件 1-9 已核对：归日与设置读写由路由级与纯函数测试钉住（1-4），三语齐备且覆盖测试通过（5），红绿时间记入进度（6），tsc/lint/bun test/build 实跑通过且 lint 对 base 无新增（7），模型表计数口径注脚在位（8），next 进程已关（9）；浏览器级实景验证因本机无数据库环境记为未跑，不冒充已通过；release-readiness 记 blocked，迁移执行与部署需用户另行批准；不 commit、不 push、不开 PR | 证据节全部实跑记录；门禁结论节五项评审；grep 禁用词与本机路径扫描零命中 |
| 2026-11-14T18:25:43.854022+08:00 | done | planned | Reviewy 第三轮复核（PR #24，head ba9c8d9）提出 3 条阻塞：时区校验未收查到规范名、getSession 整行查询在未迁移库上必崩、task.md 进度时间与模型表未按会话记录对齐；重开任务一次改完 | 工作区 fix/21-timezone-setting HEAD ba9c8d9049e291562b90fa24b57f4d37a7847994；第三轮 Reviewy 结论 2026-11-14 18:10 到达；会话记录三轮齐全 |
| 2026-11-14T18:25:47.647829+08:00 | planned | in-progress | 进入第三轮实现：Legacy/Refactoring 两门禁已就本轮范围复核（characterized / proceed），按 TDD 先红后绿改三条阻塞 | task.md 门禁结论节第三轮复核记录；分支 fix/21-timezone-setting 工作区干净 |
| 2026-11-14T18:47:33.351779+08:00 | in-progress | checking | 第三轮三条阻塞与三条可选项均已实现并验证：TDD 红绿齐全、组件层测试与自部署文档落地、授权措辞统一、进度节与模型表按会话记录对齐、四条 web 验证命令全绿、冒烟通过 | task.md 进度节第三轮条目与证据节第三轮块；红 18:28:37/18:28:44/18:28:49，绿 18:29:55；tsc 18:36:58、lint 18:37:18（0e/1w 与 base 同条）、bun test 18:37:26（164 pass）、build 18:37:51；冒烟端口 3122 已关 |
| 2026-11-14T18:47:37.208102+08:00 | checking | done | 第三轮验收核对：①校验只认规范名集合与 UTC 且两接口同源、拒绝/通过用例齐全；②getSession 主查询收窄回消费列、timezone 单独查询缺列/出错/无行降级 null，设置接口缺列 503 明确错误，均有测试；③进度节每条真实时间、第一轮红绿补记按会话记录还原、模型表第二轮一行与第三轮注脚在位；三条可选项完成；禁用词与本机路径扫描零命中；不提交、不推送；release-readiness 维持 blocked（迁移执行与部署需用户另行批准） | task.md 证据节第三轮块与门禁结论节第三轮修订；grep 禁用词/路径/密钥扫描零命中；git status 仅列本轮改动文件，无提交 |
| 2026-11-14T19:04:31.109697+08:00 | done | planned | Researchy 第四轮复核（PR #24，head 1485fc4）18:56 提出阻塞：isValidIanaTimeZone 依赖 Intl.supportedValuesOf("timeZone") 列表，而 Node 22（线上运行时）列表只有旧名（Asia/Calcutta、Europe/Kiev、Asia/Saigon）且无 "UTC"，Bun 恰好相反，测试在 Bun 全过但线上会拒绝新名用户；主 agent 已在 Node 22.20.0 复现（supportedValuesOf 缺 Asia/Kolkata、Europe/Kyiv、UTC）；重开任务改为构造器判定 | 工作区 fix/21-timezone-setting HEAD 1485fc4e8abfd27e4afdbfef40b4108c51a984cc 工作区干净；node -e 探针输出（进度节第四轮条目） |
| 2026-11-14T19:04:32.118242+08:00 | planned | in-progress | 进入第四轮实现：Legacy/Refactoring 两门禁按本轮范围复核（characterized / proceed），按 TDD 先红后绿改校验函数与设置页下拉框 | task.md 门禁结论节第四轮复核记录；分支 fix/21-timezone-setting 工作区干净 |
| 2026-11-14T19:16:21.924856+08:00 | in-progress | checking | 第四轮阻塞已实现并验证：校验改构造器判定、下拉框补齐已存值 option、happy-dom 注册守卫；TDD 红绿齐全，双运行时探针逐行一致，四条 web 验证命令全绿 | task.md 进度节第四轮条目与证据节第四轮块；红 19:06:46（15 pass/2 fail），绿 19:07:21（17 pass/0 fail）；tsc 19:08:00、lint 19:08:17（0e/1w 与 origin/main 8715c0e 基线同条）、bun test 19:10:12（165 pass/0 fail）、build 19:10:25 exit 0；探针脚本已删，无遗留进程 |
| 2026-11-14T19:16:22.929143+08:00 | checking | done | 第四轮验收核对：①校验不再依赖 supportedValuesOf，构造器判定下 Asia/Kolkata 与 Asia/Calcutta、Europe/Kyiv 与 Europe/Kiev、Asia/Ho_Chi_Minh、UTC 在 Node 22.20.0 与 Bun 1.4.4 同接受，+/- 开头偏移写法、UTC+8/GMT+8 这类、空串、超长串、仅大小写差异两运行时同拒；②PATCH 与 ?tz 仍共用 isValidIanaTimeZone；③落库存提交写法原文，不改写运行时别名；④设置页下拉框在已存值缺席列表时追加 option，组件测试钉住显示为已选；⑤进度节第四轮条目、模型表第三轮一行与第四轮注脚、门禁结论第四轮修订均在位；禁用词与本机路径扫描零命中；不提交、不推送、未加依赖；release-readiness 维持 blocked（迁移执行与部署需用户另行批准） | task.md 证据节第四轮块与门禁结论节第四轮修订；grep 禁用词/路径/密钥扫描零命中；git status 仅列本轮改动文件，无提交 |
| 2026-11-14T19:28:10.660362+08:00 | done | planned | Reviewy 第五轮复核（PR #24，head 1485fc4）19:00 判 NOT CLEAN，三条阻塞：A 校验三规则＋通过后存运行时 resolvedOptions 规范化写法、加 node --test 用例与 package.json 脚本、下拉框两侧换算同一写法再匹配；B ProfilePageClient.test.tsx afterAll 补 GlobalRegistrator.unregister() 并按 seed=3475213529 复验零挂；C task.md 时间戳与「未迁移库登录」措辞更正、模型表补第三/四轮；重开任务一次改完 | 工作区 fix/21-timezone-setting HEAD 1485fc4e8abfd27e4afdbfef40b4108c51a984cc，重开时仅第四轮未提交改动；改动前 randomize 基线复现 6 fail（证据节第五轮块） |
| 2026-11-14T19:28:10.660362+08:00 | planned | in-progress | 进入第五轮实现：Legacy/Refactoring 两门禁按本轮范围复核（characterized / proceed），按 TDD 先红后绿改三条阻塞与可选项 | task.md 门禁结论节第五轮复核记录；分支 fix/21-timezone-setting 工作区 |
| 2026-11-14T19:50:05.436582+08:00 | in-progress | checking | 第五轮三条阻塞与可选项均已实现并验证：TDD 红绿齐全、Node/Bun 双运行时实测一致、20 个 seed 全绿、六条验证命令全绿、task.md 各更正落地 | task.md 进度节第五轮条目与证据节第五轮块；红复证 19:43:38（1 error）→ 绿同刻（16 pass＋node 2 pass）；tsc 19:40:34、lint 19:40:38（0e/1w 与基线同条）、build 19:40:53、bun test 19:41:35（167 pass）、randomize 指定 seed 19:41:35（167/0）、node 用例 2 pass |
| 2026-11-14T19:52:54.196756+08:00 | checking | done | 第五轮验收核对：①`normalizeIanaTimeZone` 三规则同时满足（不以 +/- 开头且非 UTC±/GMT± 偏移写法；为 "UTC" 或带 "/"；`Intl.DateTimeFormat` 能构造）才返回并存本运行时 resolvedOptions().timeZone 写法，PATCH 与 ?tz 共用，不再依赖 supportedValuesOf；②通过集 Asia/Kolkata、Asia/Calcutta、Europe/Kyiv、Europe/Kiev、UTC、asia/shanghai 与拒绝集 EST、+08:00、-05:00、UTC+8、GMT+8、空串、超长串在 bun 测试与 Node 实跑双侧一致，断言一律用运行时自身 resolvedOptions 求得；③node 用例直引生产文件（web/scripts/test-timezone-node.mjs、package.json test:node、CI Web job 加一步）；④下拉框两侧换算同一写法再比，旧名匹配新名选项、无匹配才追加原值，组件测试钉住；⑤ProfilePageClient.test.tsx afterAll 补 unregister，TimezoneSection.test.tsx 补带守卫的 unregister（seed 11/13/29 曾 21/11/11 fail），--randomize --seed=3475213529 与另 19 个 seed 全零挂，route.test.ts/publicProfileData.test.ts 队列清空后顺序无关；⑥task.md 五处 17:33:43 一并更正为 17:36:18（index 行 17:36:37）、第一轮开始 17:20:02、「未迁移库登录」三处措辞更正为会话解析不报错但密码登录/设备轮询/团队查询仍整行查 users 需先迁移、模型表补第三轮（起止用会话头时间）与第四轮（只读核对口径）行、进度节第四/五轮条目在位；禁用词与本机路径扫描零命中；不提交、不推送、未加依赖；release-readiness 维持 blocked（迁移执行与部署需用户另行批准） | task.md 证据节第五轮块与门禁结论节第五轮修订；grep 禁用词/路径/密钥扫描零命中；git status 仅列本轮改动文件，无提交 |
| 2026-11-14T20:27:58.519419+08:00 | done | planned | Reviewy 第六轮复核（PR #24，head 3bcaa6e）20:04 到达：代码全部过关，仅剩 task.md 文字阻塞 C′（第五轮开始与第四轮开头时间口径、第五轮先红后绿具体时间），另两条可选（模型表各轮起止统一会话头口径并补第五轮行、CI setup-node 22→24 与线上 Vercel 一致）；重开任务一次改完 | 工作区 fix/21-timezone-setting HEAD 3bcaa6e7976580f093c02676fced3fd502831bf3，重开时工作区干净；阻塞 C′ 三条与可选计数已对会话记录只读核对 |
| 2026-11-14T20:27:58.519419+08:00 | planned | in-progress | 进入第六轮修改：仅 task.md 文字与 .github/workflows/ci.yml 的 setup-node 版本，不动其它代码；Book Gate 本轮无新触发（无生产代码变更），release-readiness 维持 blocked | 分支 fix/21-timezone-setting 工作区干净；本轮范围限定两个文件 |
| 2026-11-14T20:35:29.677565+08:00 | in-progress | checking | 第六轮阻塞 C′ 三条与两条可选均已落地并验证：C′ 文字更正逐条对照会话记录只读核实，模型表口径统一并补第五轮行，CI setup-node 22→24 注释同步；test:node、bun test、tsc 三条验证命令全绿 | task.md 进度节第六轮条目与证据节第六轮块；test:node 20:31:44（pass 2/fail 0，本机 Node v22.20.0）、bun test 20:31:50（167/0）、tsc 20:31:56 exit 0；ci.yml YAML 解析通过；git status 仅列本轮两个文件 |
| 2026-11-14T20:35:29.677565+08:00 | checking | done | 第六轮验收核对：①第五轮开始条目改为 19:18:20 会话头开始、19:18:32–19:28:10 读代码核对阻塞、19:28:10 任务重开；②第四轮进度补开头 18:57:00 会话头开始、18:57:30–19:04:31 读代码与复现；③第五轮先红 19:35:23（四项全挂，早于 19:35:45 首次改实现）、转绿 19:38:30（167/0）并标「补记，按会话记录还原」，19:43:38 仅作事后重验补充，进度第 8 条与证据节 19:41:35 旧口径一并改正；④模型表四轮起止统一会话头口径（17:19:59–18:00:15、18:01:50–18:07:00、18:10:44–18:48:46、18:57:00–19:18:14）并补第五轮行（88/130/2，19:18:20–19:54:12），注脚写明口径与第六轮计数结束后才有；⑤ci.yml node-version 24 与线上 Vercel 一致、注释同步、证据节写明；禁用词、本机路径与密钥扫描零命中；不提交、不推送；release-readiness 维持 blocked（迁移执行与部署需用户另行批准） | task.md 证据节第六轮块；grep 禁用词/路径/密钥扫描零命中；git status 仅列 task.md 与 ci.yml 两个文件，无提交 |
