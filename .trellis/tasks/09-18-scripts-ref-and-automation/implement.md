# 实施计划

执行顺序：BDD 场景先行 → R1 ref 解析 → R3 set-version 扩展 → R4/R5 根目录脚本 → R6 build 链 → R7 文档 → 验证 → 报告。

## Refactoring Review（mandatory gate）

- Status: proceed
- Review mode: normal
- Existing-code scope: `docs/page.tsx`（一行常量 + 两处 URL 拼接）、`scripts/set-version.sh`（追加段落）、`web/package.json`（cp 链）、`web/.gitignore`（追加）
- Behavior that must remain unchanged: 无 env 时 docs 页 URL 与现状逐字一致（`4921ccbe…` SHA）；build 拷贝既有三文件；set-version.sh 既有版本写入语义
- Structural friction: 无——`preinstallRaw` 单常量双消费，抽成 lib helper 是天然 seam
- Decision and smallest safe step: no refactor needed；抽 `scriptsRef.ts` 并替换消费点
- Safety net and validation: docs e2e（钉版 URL 断言）+ compose 冒烟 + build 后 public/ 核查
- Deferred refactors: 无

## Ponytail 决策

- 凭据探活用 `tokens status`（只读），不用 `tokens submit`（有上报副作用）。
- scriptsRef 独立小文件（沿用 `lib/site.ts` 的微小模块粒度约定），不并入 site.ts（origin 与 repo ref 是两个关注点）。
- 不做 git tag 扫描、不做 GitHub API 运行时拉取；env + 常数两级。
