---
schema_version: 1
id: 20261115-brand-purple
workflow_mode: strict
mode_source: user
mode_note: Lord 2026-11-15 按用户 2026-11-14 23:26 群内确认（选甲）排定本 PR，按 strict 执行，Lord 记录。
status: "done"
branch: fix/brand-purple
created_at: '2026-11-15T00:51:54+08:00'
updated_at: "2026-11-15T01:56:48+08:00"
completed_at: "2026-11-15T01:03:49+08:00"
---
# 品牌紫色统一（页头图标、应用图标、主题变量、对外品牌色）

## 目标

把站点残留的上游蓝色品牌元素统一为站内紫色系（品牌紫 #7C3AED），覆盖页头图标、四张应用图标、亮/暗两套主题变量、按钮与链接、焦点框、对外 OG 图与嵌入 SVG 品牌色；非品牌含义的颜色保留并在「保留清单」节列明。

依据：用户 2026-11-14 23:23 群内确认（截图指出站点左上角图标是上游蓝色、页面按钮与可跳转链接文字也是蓝色，要求换站内紫色系并全站检查按钮与链接颜色统一），Lord 记录；用户 2026-11-14 23:26 群内确认（选甲：本 PR 只改颜色，不改版本号、不改 CHANGELOG；完成后发 v1.0.6，发版另走 PR），Lord 记录。

## 已确认范围与授权

- 本会话按 strict 执行（Lord 2026-11-15 按用户 2026-11-14 23:26 群内确认（选甲）排定本 PR，Lord 记录）。不 commit、不 push、不开 PR、不访问生产、不启动子代理、不加新依赖，只改工作区文件。
- 起点 main 9355a5e9c517908ce9545737e40aa65617e9b6c1；分支 fix/brand-purple；开工前 git status 实测干净，HEAD 与起点一致。
- 色值基准：品牌紫 #7C3AED 出自存档 ai/tasks/archive/2026-Q3/t12-brand-purple/：prd.md:8（TokensMark fill 改为 #7C3AED）、implement.md:8（fill #2F6FDB → #7C3AED）、implement.md:11（demo 品牌 tile 已是 #7C3AED）、implement.md:53（Hardcoded #7C3AED 为 TokensMark 既有模式）；提交 79e8f13（2026-09-17，「feat(web): recolor favicons to brand purple and slim the footer」），色值 #7C3AED；web/public/brand/tokens-favicon.svg 已为 #7C3AED。存档未定义亮/暗两套主题的 oklch 取值，缺失部分按 #7C3AED 推导，推导见「色值推导」节。
- 未完整调用 grill-with-docs：改动对象、色值来源、保留范围与验收条件均已逐条写明，需求无歧义，不重复访谈。

## 验收条件

1. 页头 TokensMark 为紫底白 T（fill #7C3AED），与 web/public/brand/tokens-favicon.svg 同色；web/public/brand/tokens-app-icon-180.png 与 web/public/brand/tokens-app-icon-1024.png 为铺满 #7C3AED 紫色方块，四角不透明，版式同原蓝色版；web/public/android-chrome-192x192.png 与 web/public/android-chrome-512x512.png 由该 SVG 按各自原像素尺寸重导为紫色。
2. globals.css 亮/暗两套主题：--primary、--ring、--chart-1、--accent 改紫色系；--primary-foreground 与 --primary 对比度不低于 WCAG AA 4.5:1，计算结果写入「色值推导」节。
3. 按钮、链接、焦点框、标签页选中态统一走主题变量或紫色系；web/src 无残留品牌蓝色硬编码（「保留清单」节所列除外）。
4. 非品牌颜色保留清单与理由写入本文件「保留清单」节。
5. 验证全部真实执行并记录：cd web && bunx tsc --noEmit、bun run lint（与 main 9355a5e 基线比无新增错误或警告）、bun test、bun run build。
6. 本文件通过禁用词与本机路径扫描，零命中。
7. 验证完成后关掉本会话启动的 next 进程。

## Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill；纯色值变更，无领域术语或模型歧义 | 需求 | not-required |
| book-ddia-data-design | 无持久数据、schema、迁移、跨进程流变更 | 设计稳定前 | not-required |
| book-legacy-change-safety | 非既有行为 bug；纯视觉色值替换，不改变行为 | 首次行为修改前 | not-required |
| book-refactoring-pass | 修改既有生产代码（globals.css、Navigation.tsx、layout.tsx、og/route.tsx、embedShared.ts、renderIsometric3DSvg.ts、renderProfileBadgeSvg.ts） | 首次生产编辑前 | passed（proceed；normal；结论见门禁结论节） |
| book-release-readiness | 触及 api/og 路由文件与对外嵌入 SVG（徽章、嵌入卡被第三方站点引用，OG 图被社交平台缓存） | 验证通过后、收尾前 | passed（ready；结论见门禁结论节） |

TDD：本任务为字面色值替换，无逻辑缝可红绿；对比度以脚本实测计算为准，色值落地由编译产物与页面 HTML 实测钉住。BDD：skipped——本仓 no-.feature 例外，无 .feature 变更。

## 色值推导

品牌紫 #7C3AED = oklch(0.5413 0.2466 293.01)（sRGB→OKLCH 换算，bun 脚本实测，公式：sRGB 线性化→OKLab→OKLCH）。

亮色主题：
- --primary / --ring / --chart-1 = oklch(0.5413 0.2466 293.01)（即 #7C3AED 原值换算）。
- --primary-foreground 保持 oklch(0.9911 0.0015 90)（= #FCFCFB）。对比度：相对亮度 L(#FCFCFB)=0.9728、L(#7C3AED)=0.1343，(0.9728+0.05)/(0.1343+0.05) = 5.55:1，不低于 WCAG AA 4.5:1。
- --accent = oklch(0.9433 0.0284 294.59)（= #EDE9FE，violet-100 实测换算）。原值 oklch(0.9603 0.0069 250) 为近中性浅蓝（chroma 0.0069，视觉上不呈蓝）；紫色要可辨认需更高 chroma，故取 violet-100。

暗色主题：
- --primary / --ring / --chart-1 = oklch(0.7090 0.1592 293.54)（= #A78BFA，violet-400，与品牌 violet-600 同族；原暗色主色 L 0.6845，新值 L 0.7090 保持同级亮度以保证暗底可读）。
- --primary-foreground 保持 oklch(0.1822 0.0043 264)（= #111214）。对比度：L(#A78BFA)=0.3358、L(#111214)=0.0060，(0.3358+0.05)/(0.0060+0.05) = 6.89:1，不低于 4.5:1。
- --accent = oklch(0.2789 0.0300 293.01)（= #292636，sRGB 色域内）。原值 oklch(0.2789 0.0063 264) 为近中性暗面；hue 转品牌紫 293.01、chroma 提至 0.03，使暗色悬停面呈可辨认暗紫。

对外品牌色（字面 hex；OG/嵌入 SVG 输出给第三方站点，无主题变量环境）：
- OG 图（api/og/route.tsx）：BRAND #4f8ff0 → #7C3AED；HEAT 蓝色阶梯改紫色阶梯，grade0 #EEF0F3 不变，其余按品牌色相 293.01 等距插值（L 0.90→0.5413、C 0.05→0.2466，脚本换算回 sRGB）：#DFD9FD、#BAA9FB、#9978F5、#7C3AED；头像兜底底色 #E9F0FC → #EDE9FE；背景装饰 radial-gradient rgba(79,143,240,0.13)/rgba(79,143,240,0) → rgba(124,58,237,0.13)/rgba(124,58,237,0)（透明度不变）。
- 嵌入卡（embed/embedShared.ts THEMES）：亮色 brand/accentTokens/glowColor #0969DA → #7C3AED、tokenEnd #54AEFF → #A78BFA；暗色 brand/accentTokens #2F8FFF → #A78BFA、tokenEnd #A5D6FF → #C4B5FD（violet-300，暗底上保持渐变可读）。
- 徽章（embed/renderProfileBadgeSvg.ts）：METRIC_COLORS.tokens "0073FF" → "7C3AED"（tokens 度量色即品牌色）；cost 绿、rank 橙为度量语义色，保留。
- 顶部进度条（layout.tsx NextTopLoader）：#0073FF → #7C3AED。
- 应用图标 PNG（四张）：原蓝色版（git show 9355a5e 取出）均为蓝底 #2F6FDB 满幅方块（四角不透明、无圆角留白）。
  - web/public/brand/tokens-app-icon-180.png（180×180）、web/public/brand/tokens-app-icon-1024.png（1024×1024）：改为铺满 #7C3AED 紫色方块，四角不透明，版式同原蓝色版。做法：用仓库既有依赖 sharp 读 git show 9355a5e 取出的原蓝色版，逐像素求其在 #2F6FDB→白 线段上的投影比例 t，按同比例取 #7C3AED→白 插值色——蓝色背景（含抗锯齿过渡）映射为紫色，白色字形（t=1）保持不变；输出原尺寸 RGBA PNG，alpha 全 255；一次性脚本未留在仓库。实测：尺寸不变（180×180／1024×1024）；四角像素均为不透明 #7C3AED，alpha 最小值 255；中心字形白 #FFFFFF。与原蓝色版逐像素比较：两边各按背景→白投影比例 t 分类（t≥0.5 为字形、否则为背景），字形掩膜逐像素零差异（180：字形 4117 像素两边一致；1024：134538 像素两边一致），字形外接框一致（180：[32,38]–[147,144]；1024：[184,216]–[839,823]），max|Δt| 为输出整数量化误差内（0.0026／0.0029），即非背景区域位置一致。
  - web/public/android-chrome-192x192.png（192×192）、web/public/android-chrome-512x512.png（512×512）：仍按 favicon 重导，不变——由 web/public/brand/tokens-favicon.svg 按各自原像素尺寸重新栅格化的紫底（#7C3AED）白 T，RGBA PNG、尺寸不变；做法沿用提交 79e8f13 生成 favicon PNG 的方式：由 tokens-favicon.svg 直接栅格化导出（79e8f13 提交说明 favicon.ico 与 favicon-16/32.png 均由该 SVG 重新生成，安装/PWA 图标当时未动）；一次性 bun 脚本调用仓库既有依赖 sharp 完成导出，脚本未留在仓库。导出后实测：尺寸与导出前一致，图标主体像素 #7C3AED、中心白 T #FFFFFF。

中性色说明：--foreground、--muted-foreground、--border、--input、--secondary 等 hue 264、chroma 不超过 0.0122 的变量为中性灰（chroma 过低不呈蓝色），保持不变。

## 保留清单（非品牌颜色，不改）

行号为本轮 rg -n 对当前工作树实测，与文件现行内容一致。

| 文件路径 | 行号 | 颜色 | 理由 |
|---|---|---|---|
| web/src/app/globals.css | 82–86、95–99 | --token-input/-output/-cache-read/-cache-write/-reasoning：亮色 #0072b2 等五色、暗色 #56b4e9 等五色 | 用量图表固定分类图例（TokenBreakdown 消费），多色序列语义，非品牌 |
| web/src/app/globals.css | 202–205、236–239 | --chart-2..5：青/黄/品红/绿 | 图表多色序列；chart-1 才是品牌序列 |
| web/src/app/globals.css | 78–80、91–93、195、229 | --danger、--danger-solid、--success、--destructive：红/绿 | 状态语义色 |
| web/src/lib/themes.ts | 25–100 | 贡献图主题目录：GitHub 风格九套色板（含默认 blue 与 purple 选项） | 用户可选数据可视化主题，非品牌；改默认项会改变全部既有个人页与嵌入图外观，留待单独决定 |
| web/src/lib/embed/renderIsometric3DSvg.ts | 48、60、71 | graphGrade 蓝阶梯 #79B8FF/#388BFD/#1F6FEB/#0D419D 等 | 3D 嵌入卡贡献图色阶，镜像默认贡献图主题（lib/themes.ts blue），数据可视化语义；同文件 accent 为品牌色，已改紫 |
| web/src/lib/embed/clientLogos.generated.ts | 16–185 | 各厂商 logo 原色（含蓝） | 第三方品牌标识，generated 文件 |
| web/src/lib/brandIcons.generated.ts | 1–45 | 无内嵌色值：厂商品牌单色路径走 currentColor | 第三方品牌标识，generated 文件 |
| web/src/lib/constants.ts | 157–214 | SOURCE_COLORS 各家厂商品牌色（含 #00A8E8、#1A73E8 等蓝） | 第三方品牌标识 |
| web/src/components/profile/usageChartData.ts | 569 | 未归属提供方兜底 #3b82f6 | 没有归属的提供方在图表里用的兜底色 |
| web/src/lib/avatar.ts | 17 | #64748b 板岩灰 | 默认头像占位中性色 |
| web/src/components/TokenGraph2D.tsx | 55–57 | 画布底色/网格色 | 2D/3D 画布渲染专用色 |
| web/src/components/TokenGraph3D.tsx | 88、115 | 画布底色/网格色 | 2D/3D 画布渲染专用色 |
| web/src/components/GraphControls.tsx | 41、45 | 画布底色/网格色 | 2D/3D 画布渲染专用色 |
| web/src/app/settings/SettingsClient.tsx | 275 | hover:bg-[#8f1d14] 深红 | 危险操作悬停态 |
| web/src/components/profile/SpeedTier.tsx | 57 | #17181B/#e5e5e5 深灰 tooltip | 中性浮层 |
| web/src/components/profile/usageChartData.ts | 568、570、583–590、853 | #737373、#94a3b8、#090d14 灰/深画布 | 图表中性色与未归属提供方色 |

## 取舍说明

1. 导航登录按钮本就硬编码 bg-[#7C3AED]（t12 先例：品牌块用字面色，不走主题变量），保持不变；改走主题变量会让暗色主题下按钮变浅紫并需换文字色，偏离品牌块两主题一致的定位。
2. embed/OG 用字面 hex 而非 CSS 变量：这些 SVG/图片输出给第三方站点，无主题变量环境。
3. 贡献图默认主题 blue 未改：属用户可选数据可视化目录，改动会波及全部既有个人页与嵌入图外观，超出本任务范围；如需要可单独立项。
4. --token-input（蓝）与 chart-1（新紫）此后不同源：两者服务不同组件（TokenBreakdown 分类图例 vs ProfileToday/ProfileView 输入系列），按任务口径 chart-1 改、分类图例保留。
5. Lord 2026-11-15 01:35 定，甲：180/1024 改为铺满紫色方块，避免 iOS 主屏黑边。

## 模型与调用

主 agent 用 k3:max，密钥从环境变量 KIMI_API_KEY 读取；未启动子代理。

| 角色 | 模型 | 密钥来源 | 回复／工具调用／工具报错（起止） |
|---|---|---|---|
| 主 agent（r1） | k3:max | 环境变量 KIMI_API_KEY | 回复 55／工具调用 88／工具报错 6（2026-11-15 00:35:50–01:04:32 +08:00） |
| 主 agent（r2） | k3:max | 环境变量 KIMI_API_KEY | 回复 9／工具调用 15／工具报错 0（2026-11-15 01:05:04–01:09:23 +08:00） |
| 主 agent（r3） | k3:max | 环境变量 KIMI_API_KEY | 回复 12／工具调用 22／工具报错 1（2026-11-15 01:11:19–01:16:53 +08:00） |
| 主 agent（r4） | k3:max | 环境变量 KIMI_API_KEY | 回复 22／工具调用 45／工具报错 1（2026-11-15 01:20:53–01:34:15 +08:00） |
| 主 agent（r5） | k3:max | 环境变量 KIMI_API_KEY | 回复 10／工具调用 18／工具报错 0（2026-11-15 01:36:10–01:44:05 +08:00） |

工具报错 6 条明细（均为工具结果标记为报错，时间 +08:00）：2026-11-15 00:40:00 bash、00:50:34 bash、00:57:54 bash、00:58:48 write、01:02:32 bash、01:03:14 bash。

注：本轮（r6）计数只写在 PR 正文。

## 门禁结论

### Refactoring Review

```text
Refactoring Review
Status: proceed
Review mode: normal
Existing-code scope: web/src/app/globals.css（亮/暗主题变量与区块注释）、web/src/components/layout/Navigation.tsx（TokensMark fill 与注释）、web/src/app/layout.tsx（NextTopLoader color）、web/src/app/api/og/route.tsx（BRAND/HEAT/头像兜底底色、背景 radial-gradient rgba 与注释）、web/src/lib/embed/embedShared.ts（THEMES 亮/暗 brand/accentTokens/tokenEnd/glowColor）、web/src/lib/embed/renderIsometric3DSvg.ts（亮/暗 accent）、web/src/lib/embed/renderProfileBadgeSvg.ts（METRIC_COLORS.tokens）
Behavior that must remain unchanged: 白色 T 路径与 viewBox、布局/排版/交互逻辑、OG 与嵌入 SVG 的结构与文案、badge 度量与宽度表、主题切换机制、token-* 图例、chart-2..5、SOURCE_COLORS、贡献图主题目录
Structural friction: none——全部为字面色值与同义注释替换，无结构改动
Decision and smallest safe step: no refactor needed——每处仅替换色值字面量并同步注释措辞
Safety net and validation: bunx tsc --noEmit、bun run lint（对 main 基线）、bun test、bun run build；编译产物 CSS 与页面 HTML 实测紫值在位
Deferred refactors: none
```

### Release Readiness Review

```text
Release Readiness Review
Status: ready
Production path and affected users / systems: 站点全部页面的主题变量（按钮/链接/焦点框/chart-1）、页头图标、顶部进度条；/api/og 社交分享图（被社交平台缓存）；嵌入徽章与嵌入卡 SVG（第三方站点引用）
Failure modes and safeguards: 仅色值常量替换，无逻辑、协定或请求面变化；失败面为纯视觉（颜色不如预期），不涉及超时、重试、数据写入
Capacity / backpressure / limits: not-applicable（无请求面变化）
Observability / alerts / runbook: not-applicable（无运维行为变化）
Rollout / migration / rollback / cleanup: 随正常部署上线；回滚 = revert 本 diff；OG 图与嵌入图在第三方平台有缓存（社交平台卡片缓存、GitHub camo），旧蓝色图在缓存期内继续存在并自然过期，无需主动清理
Required validation and result: bunx tsc --noEmit exit 0；bun run lint 0 errors/1 warning（与 main 基线同一条，新增 0）；bun test 167 pass/0 fail；bun run build exit 0；next start 冒烟亮/暗两主题实测紫值在位——全部通过
Optional checks, accountable owner acceptance, and residual risk: 浏览器级全站 E2E 未跑（本机无数据库环境）；残留风险为个别未截图页面的视觉偏差，颜色由主题变量单一来源控制，风险低
```

### ponytail-review 结论

Lean already. Ship.——diff 逐文件过一遍：全部为字面色值一对一替换与同义注释更新，无新依赖、无新抽象、无死代码、无可删灵活性。

### 可读性复核

手写改动逐段复读：色值字面量保持各文件既有写法（CSS 变量 oklch、SVG hex 大写、badge hex 无 # 前缀），两处注释措辞随色值同步（Navigation TokensMark、OG Mark），embedShared 品牌色注释更新为新值；vendor/generated（clientLogos.generated、brandIcons.generated）未动。无发现。

## 进度

- 2026-11-15 00:35:50–01:04:32 +08:00 主 agent：第一轮（起止补记，按会话记录还原；条目内为子时间，保留原记录）。00:49 调查结论：
  1. 页头 TokensMark（Navigation.tsx:60）fill 为 #2F6FDB——t12 在 2026-09 改紫后又被上游同步带回蓝色；同文件 :287 登录按钮已是 bg-[#7C3AED]。
  2. globals.css 亮/暗两套 --primary/--ring/--chart-1 为蓝（hue 250/245），--accent 为近中性浅蓝/暗面。
  3. 全站扫描：无 Tailwind blue-/sky-/indigo- 类名命中；品牌蓝硬编码共五处——Navigation.tsx #2F6FDB、layout.tsx NextTopLoader #0073FF、og/route.tsx BRAND #4f8ff0 与 HEAT 蓝阶梯、embedShared.ts THEMES 亮 #0969DA/暗 #2F8FFF 系列、renderProfileBadgeSvg.ts tokens "0073FF"。
  4. 链接与按钮全部走主题变量（button.tsx link 变体 text-primary、focus-visible ring-ring、tabs 选中态 text-foreground 中性），改 --primary/--ring 即全站生效。
  5. 无任何测试/快照引用旧色值（rg 对 src 与 tests 扫描零命中），无需同步测试。
  6. lint 基线：web/node_modules 缺失，bun install --frozen-lockfile 恢复声明依赖（666 包，未加新依赖）后补测。
- 2026-11-15 00:51:56–01:00:43 +08:00 主 agent：实施与验证（第一轮子时间；各步时钟时间取自命令输出，编辑在基线 lint 与 tsc 之间顺序完成）：
  1. 基线 lint（00:51:56 任务转入 in-progress 之后、00:56:12 tsc 之前执行）：0 errors/1 warning（docs/page.tsx:354 no-img-element），即 main 9355a5e 基线（HEAD 与 main 一致、工作区干净）。
  2. 七个生产文件改色：Navigation.tsx（TokensMark fill + 注释）、layout.tsx（NextTopLoader）、globals.css（亮/暗 --primary/--ring/--chart-1/--accent + 区块注释）、og/route.tsx（BRAND/HEAT/头像兜底底色 + 注释）、embedShared.ts（亮/暗 brand/accentTokens/tokenEnd/glowColor + 品牌色注释）、renderProfileBadgeSvg.ts（tokens 度量色）。
  3. 全站 hex 色相复扫（bun 脚本对 src 全部 6 位 hex 算 oklch，hue 220–265 且 chroma>0.05）暴露两处漏项：embedShared.ts:635 注释引旧品牌色（同步更新）、renderIsometric3DSvg.ts 亮/暗 accent #0969DA/#57A5FF（改 #7C3AED/#A78BFA）；其余命中全部落在保留清单类别。
  4. 验证实跑（数字见证据节）：tsc 00:56:12 exit 0；lint 00:56:38 与基线同条（新增 0）；bun test 00:56:45 全量 167 pass/0 fail；build 00:57:08 exit 0。
  5. 冒烟：next start 端口 3123，/login 返回 200；HTML 含 fill="#7C3AED"、无 2F6FDB；编译 CSS 实测 --primary/--ring/--chart-1 为 #7c3aed（亮）/#a78bfa（暗）、--accent 为 #ede9fe/#292636，旧蓝 oklch 零命中；浏览器截图确认亮/暗两主题页头紫标、按钮与链接呈紫色；冒烟后 next 进程已关闭（pgrep 实测为空）。
  6. ponytail-review：Lean already. Ship.；可读性复核通过；Release Readiness Review 结论 ready。

## 证据

第一轮（2026-11-15 00:49:14–01:00:43 +08:00，分支 fix/brand-purple 工作区，基于 HEAD 9355a5e 的未提交改动；起按本会话首个实测时间戳，止按最后实测时间戳）：

- `cd web && bunx tsc --noEmit`：00:56:12 exit 0，无输出。
- `cd web && bun run lint`：00:56:38 exit 0，0 errors/1 warning（src/app/(main)/docs/page.tsx 354:19 no-img-element）。基线：main 9355a5e（HEAD 一致、改动前实测）同文件同规则同一条，新增 errors/warnings：0。
- `cd web && bun test`：00:56:45 全量 167 pass/0 fail/5576 expect()/21 文件。
- `cd web && bun run build`：00:57:08 exit 0，路由清单正常（含 /api/og 动态路由）。
- 冒烟（next start，端口 3123）：GET /login 200；页面 HTML 实测 `fill="#7C3AED"` 在位、`#2F6FDB` 零命中；编译产物 CSS（/_next/static/chunks 三个 CSS 文件合并扫描）实测 `--primary:#7c3aed`（亮）/`#a78bfa`（暗）、`--ring` 与 `--chart-1` 同值、`--accent:#ede9fe`（亮）/`#292636`（暗），旧蓝 oklch 值（0.5461/0.6845/0.9603 系）零命中；Chrome 截图确认亮主题（紫底白 T 页标、紫色 Sign in 按钮与 Create one/Resend it 链接）与暗主题（页标品牌紫不变、表单主按钮浅紫 #A78BFA 配深字、链接浅紫）两态；进程已关闭。
- 对比度计算（bun 脚本，sRGB 线性化→相对亮度→WCAG 比值）：亮色 #FCFCFB on #7C3AED = 5.55:1；暗色 #111214 on #A78BFA = 6.89:1；均不低于 4.5:1。
- 色相复扫脚本：对 web/src 全部 .ts/.tsx/.css 的 6 位 hex 逐枚算 oklch，hue 220–265 且 chroma>0.05 的命中共 18 行，逐条核对全部属于保留清单（token 图例、贡献图色板与 3D 嵌入色阶、SOURCE_COLORS、generated 厂商 logo、usageChartData 兜底）；Tailwind blue-/sky-/indigo- 类名扫描零命中；命名蓝色（steelblue 等）扫描仅命中 generated 厂商 logo 的 azure 路径数据。
- git diff --stat：第一轮 7 个生产文件 28 insertions/27 deletions（按提交 12d9f89 实测：git diff 9355a5e..12d9f89 --shortstat -- web/src；本文件此前所记 29 与实际不符，按实测改为 28）；本轮再改 og/route.tsx 背景 rgba 一行（1 insertion/1 deletion），当前 web/src 对 9355a5e 合计 7 文件 29 insertions/28 deletions，另有 web/public 四张 PNG 二进制替换（diff --stat 标 Bin），无越界改动。
- rtk：not-needed——全部命令为终端事实型输出，未走缓存包装。BDD：skipped——无 .feature 变更（本仓 no-.feature 例外）。Playwright 套件：not-needed——无页面结构变化，视觉面由 next start 冒烟与截图覆盖。
- 未跑且原因：浏览器级全站 E2E 与 test:migrations 需 DATABASE_URL/本地 postgres，本机皆无；Rust CLI 未改动，cargo 未跑；OG/嵌入图未实机请求（/api/og 需数据库取用户数据），embed 与 badge 无既有测试文件（rg 实测零命中），其色值常量经 tsc 类型检查与 build 编译覆盖。
- 依赖：无新增（bun install --frozen-lockfile 仅恢复声明依赖，bun.lock 未动）。

复验（2026-11-15 01:32:18–01:33:12 +08:00，HEAD 8d63ebb 加本轮未提交修正：og/route.tsx 背景 rgba、四张应用图标 PNG、本文件文字）：

- `cd web && bunx tsc --noEmit`：exit 0，无输出。
- `cd web && bun run lint`：exit 0，0 errors/1 warning（src/app/(main)/docs/page.tsx 354:19 no-img-element，与 main 基线同一条，新增 0）。
- `cd web && bun test`：167 pass/0 fail/5576 expect()/21 文件。
- `cd web && bun run build`：本轮复验未重跑，沿用第一轮实跑结果（00:57:08 exit 0，路由清单正常，含 /api/og 动态路由）；第一轮后 web/src 增量仅 og/route.tsx 一行 rgba 字面量（1 insertion/1 deletion），构建配置与依赖未变。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-15T00:51:56.654224+08:00 | planned | in-progress | 开工：范围、验收、Book Gate Plan 与色值推导已写入，调查完成，Refactoring Review 结论 proceed | task.md 已确认范围与授权节、色值推导节、进度节调查结论、门禁结论节 Refactoring Review；分支 fix/brand-purple，工作区开工前干净，HEAD 与 main(9355a5e) 一致 |
| 2026-11-15T01:03:44.517944+08:00 | in-progress | checking | 实现与验证完成：七个生产文件改色、全站色相复扫零残留品牌蓝、tsc/lint/bun test/build 全绿、亮暗两主题冒烟截图确认、ponytail-review 与可读性复核通过、Release Readiness Review 结论 ready | task.md 进度节 00:51:56–01:00:43 条目与证据节第一轮块；tsc 00:56:12 exit 0、lint 00:56:38 与基线同条、bun test 00:56:45 167 pass/0 fail、build 00:57:08 exit 0；冒烟端口 3123 已关 |
| 2026-11-15T01:03:49.598875+08:00 | checking | done | 验收条件 1-7 已核对：页标 fill #7C3AED 与 favicon 同色（1）；亮暗两套主题变量改紫且对比度 5.55:1/6.89:1 不低于 4.5:1（2）；按钮/链接/焦点框/标签页走主题变量、品牌蓝硬编码零残留（3）；保留清单与理由在位（4）；四条验证命令实跑全绿且 lint 对 main 基线新增 0（5）；禁用词与本机路径扫描零命中（6）；next 进程已关（7）；浏览器级全站 E2E 因本机无数据库环境记为未跑，不冒充已通过；不 commit、不 push、不开 PR | task.md 证据节全部实跑记录；门禁结论节 Refactoring/Release Readiness/ponytail-review/可读性复核；grep 禁用词与本机路径扫描零命中；git status 仅列本任务与七个生产文件 |
| 2026-11-15T01:44:05+08:00 | done | done | r3–r5 修正：保留清单补行号与色值出处；OG 背景光晕 rgba(79,143,240,…) 改 rgba(124,58,237,…)；四张应用图标改紫（tokens-app-icon-180/1024 铺满紫色方块、android-chrome 192/512 按 favicon 重导）；记录修正（确认时间补到分钟、「达标」改「不低于 4.5:1」、insertions 按实测改 28、审查范围补 renderIsometric3DSvg.ts）；r4 后复验 tsc、lint（0 错 1 警告，同 main 基线）、bun test 167/0；web/src 与四张应用图标中品牌蓝零残留（保留清单所列除外） | 行号 rg -n 对当前工作树实测；色值出处对照存档 ai/tasks/archive/2026-Q3/t12-brand-purple/ 与提交 79e8f13；四张 PNG 经 sharp 脚本逐像素实测（尺寸不变、180/1024 四角不透明 #7C3AED、字形掩膜与原蓝色版逐像素一致）；web/src 全部 6 位 hex 色相复扫脚本命中均落保留清单；复验实跑 cd web && bunx tsc --noEmit、bun run lint、bun test（01:32:18–01:33:12：exit 0、0 errors/1 warning 同 main 基线、167 pass/0 fail）；git diff --stat 核对改动面 |
