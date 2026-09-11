# T12 实施

**依赖**：T0（已 archive `--no-commit`）  
**推进清单**：`docs/TODO.md` T12 节

## 步骤

1. `web/src/components/layout/Navigation.tsx` 的 `TokensMark`：`fill="#2F6FDB"` → `#7C3AED`，白色 T 不变。
2. 注释改为：品牌块是字面色；favicon / 安装图标本轮仍蓝。
3. 不改 `tokens-favicon.svg`、位图、`tokens-mark.svg`。不全局替换 `#2F6FDB`。
4. 核对 demo 品牌 tile 已是 `#7C3AED`。

## 验收

- 导航 Tokens 前图标紫底白 T
- `web/public/brand/tokens-favicon.svg` 仍为 `#2F6FDB`
- `bun run lint` / `bun run typecheck` 过

## Check notes (2026-09-11)

Gates: `bun run lint` (0 errors, 2 pre-existing warnings in docs/page.tsx and worker.ts), `bun run typecheck` (tsc --noEmit exit 0).

```text
Book Gate Plan
book-refactoring-pass: required (edit existing Navigation.tsx) — passed
book-legacy-change-safety: not-required (not an existing-behavior bug)
book-ddia-data-design: not-required
book-ddd-distilled-modeling: not-required
book-release-readiness: not-required
```

```text
Refactoring Review
Status: proceed
Review mode: normal
Existing-code scope: Navigation.tsx TokensMark rect fill + comment
Behavior that must remain unchanged: white T paths; NAV_LINKS; GitHub icon; theme toggle; favicon assets
Structural friction: none
Decision and smallest safe step: no refactor needed — one fill literal + comment
Safety net and validation: bun run lint && bun run typecheck; grep remaining #2F6FDB only in favicon/comment
Deferred refactors: none
```

```text
Code Readability Review
Scope: modified hand-written production code and tests
Findings: none
Ponytail conflicts resolved: none (kept existing brand-literal hex; quality spec “no hardcoded hex” applies to theme UI, not this brand tile)
Changes applied: none
Revalidation required: no
```

Hardcoded `#7C3AED` is the existing TokensMark pattern (was `#2F6FDB`). Not a new theme token. Favicon left blue on purpose.
