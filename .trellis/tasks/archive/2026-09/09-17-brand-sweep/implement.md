# 实施计划：tokens.ci 品牌清理
## 共享模块（先做，其余都依赖它）

新增 `web/src/lib/site.ts`：

```ts
/** 部署自身 origin（https://host[:port]）；任何用户可见 URL 都从它派生，不写死上游域名。 */
export const SITE_URL = (process.env.NEXT_PUBLIC_URL || "http://localhost:3000").replace(/\/+$/, "");
/** 部署 hostname，供 OG / embed / 法律页等品牌展示。 */
export const SITE_HOST = new URL(SITE_URL).host;
```

`layout.tsx` 与 `docs/page.tsx` 现有的本地 `SITE_URL` 常量删掉，改为 import。

## 逐文件

| 文件 | 改动 |
|---|---|
| `web/src/app/layout.tsx` | 本地 SITE_URL 删除，import；行为不变 |
| `web/src/app/(main)/docs/page.tsx` | 同上 |
| `web/src/app/(main)/teamboard/page.tsx:30` | `url: "https://tokens.ci/teamboard"` → `` `${SITE_URL}/teamboard` `` |
| `web/src/app/u/[username]/page.tsx:116` | `url: \`https://tokens.ci/u/${username}\`` → SITE_URL 派生 |
| `web/src/app/(main)/privacy/page.tsx:40`、`terms/page.tsx:30` | 链接 href 指 SITE_URL，可见文本 `tokens.ci` → `SITE_HOST` |
| `web/src/components/legal/LegalPage.tsx:7` | `CONTACT_EMAIL` 改为 `process.env.CONTACT_EMAIL`；未配置时调用处不渲染 mailto 链接（privacy 页的联系句改为条件渲染：有邮箱才渲染 mailto，否则只保留站点链接） |
| `web/src/app/api/og/route.tsx:176` | OG 图右下角 "tokens.ci" → SITE_HOST |
| `web/src/lib/embed/embedShared.ts:667`、`renderDetailedEmbedSvg.ts:173`、`renderIsometric3DSvg.ts:248,275`、`renderProfileEmbedSvg.ts:186`、`renderPulseEmbedSvg.ts:171` | SVG 里 "tokens.ci" / "tokens.ci/u/…" 文案改用 SITE_HOST |
| `web/src/components/profile/embedDialogOptions.ts:41` | `TOKENS_URL` → SITE_URL |
| `web/src/lib/auth/requestSession.ts:13` | 默认 origin 列表去掉 `https://tokens.ci`，保留 `http://localhost:3000`（`NEXT_PUBLIC_URL` 已自动加入） |
| `web/src/lib/i18n/t.ts` | `terms.software.p1`（en + zh）"tokens.ci" → `{host}` 占位；`terms/page.tsx` 调用处传 `{ host: SITE_HOST }` |

## BDD + 测试

- 新建 `web/features/site-identity.feature`（中文场景 + 英文关键词）：
  - 文档页/法律页/榜单页 HTML 无 tokens.ci，og 链接与品牌字样显示本站 host
  - 未配置 CONTACT_EMAIL 时法律页无 mailto 链接；配置后渲染
  - `/api/og` 与 embed SVG 文案显示本站 host
- e2e：新增 `tests/e2e/site-identity.spec.ts`（或并入 t9-acceptance）：
  - `/privacy`、`/terms`、`/leaderboard`、`/u/<fixture>` HTML 无 tokens.ci，含本站 host
  - `/api/og?title=…` 返回 SVG/PNG 文本含 localhost 不含 tokens.ci
  - CONTACT_EMAIL 未配置时 `/privacy` 无 `mailto:`

## 验证

1. `rtk bun run lint` + `rtk bun run typecheck`
2. `bun run build`
3. rsync 到 Compose → curl 冒烟：`/privacy` `/terms` `/docs` `/leaderboard` 的 HTML 无 `tokens.ci`，品牌字样为 `localhost:3000`
4. e2e 新场景通过；`t9-acceptance` 既有用例不回退
5. 删掉 `docs/deploy/self-host-production.md` 待开发项 #4

## 门禁记录

- grill-with-docs：未完整调用（环境未暴露 Skill）；两处决策 owner 已拍板
- BDD：场景先于实现
- Refactoring Review：proceed（机械替换 + 一个共享模块收编）
- Legacy Change Safety：characterized（改的是展示文案/链接，不改行为；e2e 兜底）
- DDIA Data Design Review：`confirmed`

```text
Data owner and source of truth: 部署身份的唯一 SOT 是 NEXT_PUBLIC_URL（SITE_URL/SITE_HOST 皆由此派生）；CONTACT_EMAIL 是独立的运营配置。无持久化数据变更、无 schema/迁移。
Write / read / async / failure paths: OG 图与 embed SVG 的内容由 tokens.ci 改为本站 host——由我方服务器渲染，响应形状不变、文案内容变化；第三方站点上已嵌入的徽章引用的是上游 tokens.ci 的 URL，不受本站改动影响。CSRF 默认源移除 tokens.ci：上游域名的跨源写与本站无关，NEXT_PUBLIC_URL 本就自动入列，无合法流量被断。
Consistency model: 站点身份全站一致（同一 SITE_URL/SITE_HOST 单源）。
Idempotency / ordering / retry: 无——纯读路径文案。
Schema / migration / backfill / rollback: 无迁移；回滚 = revert commit。
Observability and repair: 无新增；若 NEXT_PUBLIC_URL 配错，OG/embed/法律页全站显示同一错误 host，一眼可发现。
Required tests: e2e 站点身份场景（/privacy /terms /leaderboard /api/og 零 tokens.ci 且显示本站 host）；CONTACT_EMAIL 未配置时 /privacy 无 mailto。
```

- Release Readiness：`ready`（2026-09-17）。证据：lint/tsc/build 绿；Compose 冒烟 4 页零 tokens.ci；e2e 17/17（最终报告 `tests/e2e/reports/html/playwright-report-site-identity-feature_teamboard-teams-auth-2026_09_17-19_11_42.*`）；CONTACT_EMAIL 双向验证（未配置无 mailto；配置 `ops@localhost.dev` 后 privacy 10 条 / terms 7 条 mailto、无 mailto:undefined、零 tokens.ci）；独立 trellis-check PASS + 3 minor findings 已修复复验。残余风险：/api/og 为 PNG，host 文案只能断言不含 tokens.ci（SVG embed 端点已断言含 host）。


## 偏差记录（2026-09-17 check 回填）

- **`/api/og` 的 host 断言改在 embed SVG 上做**：`/api/og` 输出是 ImageResponse 栅格化的 PNG，品牌字样是像素不是文本，无法断言"响应含 host"。e2e 改为断言 PNG 字节流不含 `tokens.ci`，host 存在性断言落在返回文本 SVG 的 `/api/embed/<username>/svg` 上。
- **配置 CONTACT_EMAIL 的场景为手工验证**（feature 中「配置 CONTACT_EMAIL 后法律页渲染邮箱链接」）：用 `CONTACT_EMAIL=ops@localhost.dev` 起一次性 dev server 验证——privacy 渲染 10 个 mailto 链接、terms 7 个、无 `mailto:undefined`、全站零 `tokens.ci`。未自动化（e2e 栈固定不配置 CONTACT_EMAIL）。
- **部署清单待开发项编号**：`docs/deploy/self-host-production.md` 的品牌清理缺口实际是列表第 3 项（页脚项此前已删），非 prd 写的 #4；已按实际编号移除正确条目。

