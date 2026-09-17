# Brand sweep: replace tokens.ci with deployment origin

## Goal

清除 `web/` 里残留的 `tokens.ci` 硬编码：法律页正文与链接、OG/embed 图像文案、联系邮箱、CSRF 默认源、各页 og:url。部署身份统一来自 `NEXT_PUBLIC_URL`；联系邮箱来自 `CONTACT_EMAIL`。

## Decisions（owner 2026-09-17 拍板）

- 品牌字样显示**部署域名**（`NEXT_PUBLIC_URL` 的 hostname）。
- 联系邮箱走 **`CONTACT_EMAIL`** 环境变量；未配置时法律页不渲染邮箱链接（不造假地址）。

## Requirements（清单 + 逐条处置）

- **R1** 新增共享模块 `web/src/lib/site.ts`：`SITE_URL`（origin，去尾斜杠）+ `SITE_HOST`（host）。`layout.tsx` 与 `docs/page.tsx` 现有的本地 `SITE_URL` 计算收编进它。
- **R2** og:url：`teamboard/page.tsx`、`u/[username]/page.tsx` 改用 `SITE_URL` 派生。
- **R3** 法律页链接：`privacy/page.tsx`、`terms/page.tsx` 的 tokens.ci 链接改指本站，可见文本显示 `SITE_HOST`。
- **R4** 联系邮箱：`LegalPage.tsx` 的 `CONTACT_EMAIL` 改为读 `process.env.CONTACT_EMAIL`；未配置时不渲染 mailto 链接（只保留站点链接）。
- **R5** OG/embed 文案：`api/og/route.tsx` 与 embed 渲染器（`embedShared.ts`、`renderDetailedEmbedSvg.ts`、`renderIsometric3DSvg.ts`、`renderProfileEmbedSvg.ts`、`renderPulseEmbedSvg.ts`）里的 "tokens.ci" 文案改用 `SITE_HOST`；`embedDialogOptions.ts` 的 `TOKENS_URL` 改用 `SITE_URL`。
- **R6** CSRF 默认源：`requestSession.ts` 默认列表去掉 `https://tokens.ci`（保留 localhost；`NEXT_PUBLIC_URL` 本就自动加入）。
- **R7** i18n 条款文案：`terms.software.p1`（中英）把 `tokens.ci` 参数化为 `{host}`，调用处传 `SITE_HOST`。
- **R8** BDD + e2e：`web/features/site-identity.feature`；e2e 断言 `/privacy` `/terms` `/leaderboard` `/u/<user>` `/api/og` 渲染零 `tokens.ci`、显示本站 host。

## Constraints

- 不动 CLI。
- 不改历史叙述文档里的 `tokens.ci`（`docs/TODO.md`、`docs/prd-teamboard-teams-auth.md` 历史段、`.trellis` 归档豁免）。
- 法律页结构不变（仅链接 / 邮箱 / 域名参数化）。
- `grill-with-docs` 未完整调用（环境未暴露该 Skill）；两处决策已由 owner 直接拍板。

## Acceptance Criteria

- [x] `web/src` 内 grep `tokens.ci` 为零（历史叙述豁免）
- [x] lint / typecheck / build 全绿
- [x] Compose 冒烟：`/docs` `/privacy` `/terms` `/leaderboard` 及 OG 图渲染的 host 为 `localhost:3000`
- [x] e2e：站点身份场景通过
- [x] `CONTACT_EMAIL` 未配置时法律页无 mailto 链接；配置后渲染
- [x] `docs/deploy/self-host-production.md` 待开发项品牌清理条目移除（本任务完成它；实际为第 3 项）
