---
schema_version: 1
id: 20261114-v2721-pr-d-timezone
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-11-14 01:50 在群内开启全自动模式直到 v1.0.5 按模板发布且文档补全；2026-11-14 08:01 用户就 issue #21 选定乙：时区存进账号设置（用户表加可为空的时区字段，为空表示自动检测），迁移随 PR D 一起审，生产环境执行迁移需用户另行批准，Lord 记录。本会话交付清单含 Book Gate Plan 与完整验证记录，按 strict 执行。
status: "done"
branch: fix/21-timezone-setting
created_at: '2026-11-14T17:33:43.000000+08:00'
updated_at: '2026-11-14T18:05:05.000000+08:00'
completed_at: '2026-11-14T17:55:41.000000+08:00'
---
# 修复 issue #21（PR D）— 个人页按看的人时区归日 + 账号设置时区选项

## 目标

修 issue #21：个人页图表的「今天」现在按服务器 UTC 日期算，UTC+8 用户每天 00:00–08:00 看不到当天数据。落地四条改动：

1. 默认用浏览器检测到的时区（`Intl.DateTimeFormat().resolvedOptions().timeZone`）决定图表结束日、「今天/昨天」卡片和相关统计的归日，不再用服务器 UTC 日期。
2. 账号设置页加时区选项，默认「自动检测」；选了具体时区就覆盖检测值，全站按它归日。存储：users 表新增可为空的 timezone 字段（null = 自动检测），附数据库迁移文件（drizzle-kit，web/src/lib/db/migrations 现有惯例）；服务端校验必须是合法 IANA 名称；设置接口读写这个字段。
3. 看别人的个人页按看的人的时区归日（看的人设置了就用设置，否则用其浏览器检测值），不按页面主人的；规则写进文档（docs 页，en、zh-CN、ja 三语）。
4. UTC 用户行为不变。

## 已确认范围与授权

- 用户 2026-11-14 01:50 在群内开启全自动模式直到 v1.0.5 按模板发布且文档补全；2026-11-14 08:01 用户就 issue #21 选定乙：时区存进账号设置（用户表加可为空的时区字段，为空表示自动检测），迁移随 PR D 一起审，生产环境执行迁移需用户另行批准，Lord 记录。
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

- `web/src/lib/timezone.ts`（新）：纯函数 `isValidIanaTimeZone`、`dateStringInTimeZone`、`addDaysToDateString`、`resolveViewerTimeZone`，bun test 直测。
- `web/src/lib/publicProfileData.ts`：`getProfilePeriodDateRange` / `getRollingProfileDateRange` 导出后按固定时刻直测；路由层 `tz` 查询参数经 mock 的 db 测响应区间与 400。
- `web/src/app/api/settings/timezone/route.ts`（新）：mock db 与 getSessionFromRequest，测未登录 401、非法时区 400、写入与清空、读取。

## 进度

- 2026-11-14 17:29:10 +08:00 主 agent：调查完成（首个实测时间戳；调查本身在此之前进行）。结论：
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
- 2026-11-14 17:33:43 +08:00 主 agent：创建本文件与 index 行；Book Gate Plan 登记；DDIA、Legacy、Refactoring 三项门禁评审结论写入门禁结论节。
- 2026-11-14 17:33:43–17:55:41 +08:00 主 agent：TDD 三切片与收尾，全部实测（各步时钟时间在会话内顺序推进，逐条命令输出留存于会话记录）：
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
- 2026-11-14 18:05:05 +08:00 主 agent：第二轮（文档补充，仅改本文件）。第一轮已提交为 f0778bad0a2b61b87a1a9d538cc11d59d38267ac 并开启 PR #24；Reviewy 复核后要求写清迁移执行时机。本轮按 web/vercel.json 原文核对后落地：授权节新增迁移执行时机条目、取舍说明 8 补充前提与预览验收限制、DDIA 评审 Schema/migration 行与 Release Readiness 评审 Rollout 行补部署次序与预览限制，四处口径一致；模型表补第一轮一行（回复 118、工具调用 150、工具报错 5，2026-11-14 17:20:02–18:00:15 +08:00，逐条核对会话记录原始计数），注脚口径改为按轮取最终计数；frontmatter updated_at 同步。不提交、不推送；状态无迁移，状态事件表不新增行。

## 取舍说明

1. **API 增加 `tz` 查询参数**：week/month 的行过滤发生在服务器，看的人「今天」的行在 UTC 口径范围外时客户端无从补回，只能让服务器按看的人时区算 periodRange 与滚动区间；`tz` 非法时返回 400（显式失败好过静默落回 UTC 的误归日）。all 期间本可纯前端重算（响应含全部贡献行），但三档期间统一走「挂载后按看的人时区重取一次」的单一路径，比分两套逻辑更可读。
2. **首屏维持服务器区间**：u 页 ISR 共享缓存不含看的人会话，按 Cookie 分叉缓存会让每个时区各存一份页面；首屏用 UTC 区间、挂载后修正，沿用 ProfilePageClient 既有「晚些再声称今天」的安全方向惯例。
3. **UTC 用户零变化**：看的人时区的今日与服务器响应的 `chartRange.end` 一致时客户端不发重取请求；UTC 看的人两值恒等，渲染路径与现状逐帧一致。
4. **徽章与 embed SVG 不动**：它们是服务器渲染的静态图片，没有看的人浏览器可言，issue 三条改动与四条验收均未涉及。
5. **榜单「today」期间不动**：同属服务器 UTC 口径，但 issue 范围是个人页图表与今天/昨天卡片；改榜单属于另一处行为变更，留待单独提出。
6. **timezone 用 text 列、应用层校验**：IANA 时区库随系统更新，数据库枚举会把合法值挡在库外；校验集中在 `isValidIanaTimeZone`（`Intl.DateTimeFormat` 构造试抛 RangeError），服务端写入与客户端选项同源。
7. **会话响应携带 timezone 而非新拉取**：个人页本就在挂载后请求 `/api/auth/session`（设备列表用），在其中带上看的人 timezone 避免第二次往返；未登录时 `user` 为 null，自然落到浏览器检测值。
8. **迁移只进仓库不执行**：本机无数据库实例，授权边界亦如此；迁移正确性由 drizzle-kit 生成器、meta 快照与代码评审承担，生产执行需用户另行批准。这个取舍成立的前提是部署管道本身控住时机：合入不触发自动部署，预览构建不跑迁移，生产构建内迁移先于新 web 代码生效，所以「先生成的迁移文件」不会在未获批准时落到任何数据库。代价是预览环境不能用于本 PR 的登录相关验收——预览库没有 timezone 新列时，读用户整行的会话查询会因缺列报错。

## 模型与调用

主 agent 用 k3:max，密钥从环境变量 KIMI_API_KEY 读取。

| 角色 | 模型 | 密钥来源 | 调用次数 |
|---|---|---|---|
| 第一轮主 agent | k3:max（kimi-code/k3） | 环境变量 KIMI_API_KEY | 回复 118、工具调用 150、工具报错 5（2026-11-14 17:20:02–18:00:15 +08:00） |

统计口径：工具报错 = 会话文件中标记为报错的工具结果条数（含命令非零退出）。每轮一行取该轮会话文件的最终计数；本轮（第二轮）计数要等本轮结束才有，不写进表。

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
- Required tests：路由测试覆盖 401/400/写入/清空/读取；`isValidIanaTimeZone` 边界用例。

### Legacy Change Safety Review

- Status: characterized
- Behavior to change：`getPublicProfileResponse` 的 chartRange 与 week/month periodRange 由服务器 UTC 今日改为按 `tz` 参数（看的人时区）今日；无 `tz` 时维持 UTC。ProfilePageClient 挂载后按看的人时区重取并以该时区算今天/昨天卡片。
- Behavior to preserve：无 `tz` 参数时响应与现状逐字节一致（UTC 用户行为不变）；`dateRange` 的全量语义（最早/最晚提交日期）、贡献强度分档、排名、隐私门、封禁响应、重定向、缓存标签与 revalidate、embed/badge 路径全部不动。
- Current reproduction evidence：issue #21 实测记录（2026-11-14 07:28 UTC+8，chartRange.end=2026-11-13 而 dateRange.end=2026-11-14）；源码确认两个 range 函数用 `toISOString().slice(0, 10)`；现有 `publicProfileData.test.ts` 为本路径既有安全网。
- Safety net：现有 126 条 bun test（含 publicProfileData、i18n 覆盖）＋本任务新增 timezone 纯函数、range 函数固定时刻、路由 tz 参数与设置路由测试；tsc/lint/build 防回归。
- Hidden dependencies / seam：`loadPublicProfileForPage` 的 `unstable_cache` 不传 `tz`，缓存键不变、共享缓存仍全看的人通用；`getViewerAccess` 读会话判 owner 的路径不受影响；无新 seam。
- Validation plan：先红后绿，再跑项目四条 web 验证（结果见证据节）。
- Review mode: normal

### Refactoring Review

- Status: proceed
- Review mode: normal
- Existing-code scope：`publicProfileData.ts`（两个 range 函数加时区参数、路由解析 tz）、`lib/auth/session.ts`（SessionUser 加字段）、`ProfilePageClient.tsx`（data 改 state、挂载后重取）、`SettingsClient.tsx`（挂新分节）、docs 页（新 Section）。
- Behavior that must remain unchanged：见 Legacy 节 preserve 清单。
- Structural friction：两个 range 函数已带 `now` 参数且为纯函数，加时区参数即最小扩口；设置分节与 docs Section 均有现成样板（PrivacySection、privateTitle Section），不需要结构调整。
- Decision and smallest safe step：no refactor needed；不夹带清理。
- Safety net and validation：同 Legacy 节。
- Deferred refactors：无。

### Release Readiness Review

- Status: blocked
- Production path and affected users / systems：`/api/users/[username]`（公共读，新增 tz 参数）、`/api/settings/timezone`（新，仅浏览器会话写）、`/api/auth/session`（响应多一个字段）、users 表新列（迁移未执行）、u 页挂载后可能多发一次带 tz 的重取。
- Failure modes and safeguards：非法 tz → 400（写设置）/400（读个人页），不静默落回 UTC；重取失败保留 ISR 首屏数据；看的人时钟偏斜由一次性重取闸挡住循环；访客无会话落到浏览器检测值；`PRIVATE_CACHE_CONTROL` 挡住设置接口响应进共享缓存。
- Capacity / backpressure / limits：非 UTC 看的人在跨日期窗口每个个人页最多多发一次 API 请求；u 页 ISR 缓存键不变，无缓存分裂；设置接口单行单字段更新。
- Observability / alerts / runbook：设置读写失败走既有 console.error 通道；无新指标。
- Rollout / migration / rollback / cleanup：**部署次序是硬前提——`getSession` 走整行 users 查询，web 上线前必须先执行 0028 迁移，否则所有会话查询因缺列报错；迁移本身是纯加列，旧代码对新列无感，回滚 web 即可。** 迁移执行需用户另行批准，不在本会话授权内。时机与次序按 web/vercel.json 原文核对：buildCommand 为「`if [ "$VERCEL_ENV" = "production" ]; then ... bun run db:migrate; fi && bun run build`」，即仅正式环境构建才先跑迁移再编译，同一次构建内迁移先于新 web 代码生效；`git.deploymentEnabled.main` 为 false，main 分支不自动部署，合入本 PR 本身不动线上数据库；真正执行迁移的是之后那次手动正式部署。预览部署不跑迁移，预览库缺 timezone 列时整行会话查询报错，预览环境的登录相关功能不能作为验收依据。
- Required validation and result：tsc、lint（对 base 无新增）、bun test 150、build 全绿；未登录 401、非法 tz 400、docs 三语小节冒烟通过；结果见证据节。
- Optional checks, accountable owner acceptance, and residual risk：带真实数据库的浏览器级 E2E（UTC+8 00:30 实景）本机无 postgres 服务与 docker，无法运行，由路由级测试与区间函数固定时刻测试覆盖归日逻辑；CI、合并、部署与生产迁移执行不在本会话授权内。

### ponytail-review 结论

Lean already. Ship. 五个纯函数全部被引用、无新依赖、无投机抽象；过程中删掉一处会导致 week 视图误空窗的守卫（见进度节第 9 条）。

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

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-14T17:33:43.000000+08:00 | planned | in-progress | 开工：范围、验收与 Book Gate Plan 已确认，调查完成，DDIA/Legacy/Refactoring 三门禁结论已出 | task.md 已确认范围与授权节、进度节调查结论、门禁结论节；分支 fix/21-timezone-setting，工作区开工前干净，与 main(8715c0e) 一致 |
| 2026-11-14T17:55:41.000000+08:00 | in-progress | checking | 实现与验证完成：TDD 三切片先红后绿、迁移文件生成、设置页与 docs 三语落地、四条 web 验证命令全绿、冒烟通过、ponytail-review 与可读性复核完成 | task.md 进度节 17:33:43–17:55:41 各条、证据节 |
| 2026-11-14T17:55:41.000000+08:00 | checking | done | 验收条件 1-9 已核对：归日与设置读写由路由级与纯函数测试钉住（1-4），三语齐备且覆盖测试通过（5），红绿时间记入进度（6），tsc/lint/bun test/build 实跑通过且 lint 对 base 无新增（7），模型表计数口径注脚在位（8），next 进程已关（9）；浏览器级实景验证因本机无数据库环境记为未跑，不冒充已通过；release-readiness 记 blocked，迁移执行与部署需用户另行批准；不 commit、不 push、不开 PR | 证据节全部实跑记录；门禁结论节五项评审；grep 禁用词与本机路径扫描零命中 |
