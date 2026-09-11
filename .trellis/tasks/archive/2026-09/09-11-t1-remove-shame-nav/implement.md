# T1 实施

**依赖**：T0、T12（均已完成）  
**推进清单**：`docs/TODO.md` T1 节

## 步骤（已做）

1. 删除 `web/src/app/(main)/shame/page.tsx`、`web/src/components/shame/BannedList.tsx`。
2. 导航第二项改为 Teamboard；占位页 `web/src/app/(main)/teamboard/page.tsx`。
3. 改写引用：BannedProfileView（去掉 Hall of Shame 链）、Docs（删 iOS / Architecture / TESTFLIGHT；verified 节去掉 `/shame` 链，整节留给 T2）、Terms、Privacy（去掉 iOS app 文案）、`worker.ts` `PAGE_CACHEABLE`、`error.tsx`、`schema.ts` 注释、GitHub callback 禁号改去 `/u/{username}`、PageHeader 注释。
4. **未改** `ServiceFooter.tsx`。
5. Spec 同步：directory-structure、component-guidelines、data-fetching、error-handling、cloudflare-deployment。

## 验收

- `web/src` 与 `web/worker.ts` 中 `rg -i "hall of shame|/shame|TESTFLIGHT"` 无命中（禁号 GitHub 登录改跳 `/u/{username}`，不再引用 `/shame`）
- lint 0 errors；typecheck `tsc --noEmit` exit 0；**未跑** `build` / `test:migrations`

## Check notes (2026-09-11)

```text
Book Gate Plan
book-refactoring-pass: required (edit existing production) — passed proceed
book-legacy-change-safety: required (delete user-visible shame page) — characterized
book-ddia-data-design: not-required (no schema/migration)
book-ddd-distilled-modeling: not-required
book-release-readiness: not-required this slice
```

```text
Legacy Change Safety Review
Status: characterized
Behavior to change: Hall of Shame page, nav item, public listing copy, /shame links
Behavior to preserve: ban fields, banned profile URL, leaderboard exclusion of banned users, ServiceFooter
Current reproduction evidence: source inventory of /shame and Hall of Shame refs; ban path in schema + BannedProfileView kept
Safety net: bun run lint && bun run typecheck; grep for leftover /shame
Hidden dependencies / seam: worker PAGE_CACHEABLE; GitHub banned redirect
Validation plan: lint + typecheck; grep web/src web/worker.ts
Review mode: normal
```

```text
Refactoring Review
Status: proceed
Review mode: normal
Existing-code scope: nav, docs, terms, worker cache regex, comments, banned profile CTA
Decision and smallest safe step: no extra refactor — delete shame surface, rewrite refs, placeholder teamboard
Safety net and validation: lint + typecheck
Deferred refactors: none
```

```text
Code Readability Review
Scope: modified hand-written production code and tests
Findings: none
Ponytail conflicts resolved: none
Changes applied: none
Revalidation required: no
```
