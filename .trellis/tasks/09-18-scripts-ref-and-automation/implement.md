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

## 验证与门禁记录

- 提交：`e4374681264563ae53d1a76508b57cd0fd3e3ca7`（主体）+ `9badc48d20c526b5f499dd0b19337620bcdaac97`（ps1 runner 撇号转义，评审发现）。
- lint 0 errors（1 条 pre-existing warning）/ tsc 干净 / test:migrations 绿 / next build 绿且 public/ 出现两脚本。
- 脚本：`bash -n` 通过；enable `--dry-run` 计划输出正确（占位符域名被拒）；ps1 无 pwsh 仅静态审查通过。
- resolver 三态 eval：默认 SHA / `v9.9.9-test` 覆盖 / 非法值 warn+回退；页面级覆盖在 `:3100` 实例验证（URL 全改 tag）。
- `set-version.sh 99.0.0` 临时副本演练：4 文件替换计数正确；删脚本后 fail-closed（rc=1，零 mutation）。
- 正式 e2e 最终轮 passed（2.3s），报告对：`tests/e2e/reports/html/playwright-report-docs-page-feature_teamboard-teams-auth-2026_09_18-11_25_49.html/.md`；中间轮 11_22_15、11_24_39 均封存。
- Release Readiness Review: **blocked**。未执行的核心平台验证：① Linux systemd 实跑（`systemctl --user is-enabled tokens`、override.conf 内容、服务指向本站）；② Windows pwsh `-DryRun` + 计划任务注册实跑（本机无 pwsh）。可选检查的残留风险**尚未取得 owner（用户）显式接受**；归档/完成以用户接受或真机证据为前提。
