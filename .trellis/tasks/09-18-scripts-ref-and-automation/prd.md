# 脚本引用 tag 化与常驻自动化脚本

## Goal

docs 页 pre-install 脚本 URL 改为可随 release 推进的 ref；新增两个根目录常驻自动化脚本（Linux systemd / Windows 计划任务）。

## Background

- 当前 `docs/page.tsx` 把 `preinstallRaw` 钉死在 commit `4921ccbed1f4286e75c35f676c400ec8f83012a6`，手册 §3 与 README 同样钉该 SHA；每次脚本更新要手工同步多处。
- 用户要求：仓库有 tag 时优先用 tag 拼接脚本 URL，无 tag 才用 SHA，且发布新 tag 后自动用最新 tag。
- **已验证的陷阱**：现存 tag `v27.0.0` / `v27.0.1` 只含 `install.sh`，不含 pre-install 双脚本（脚本由 `4921ccbe` 引入，不在任何 tag 内）。"扫描最新 tag 直接用"今天就会让 docs 安装链接 404。
- 发布链事实：`ci.yml` 无 deploy / tag 触发；`publish-cli.yml` 是 CLI 的 on-demand 发布；web 为 self-host Docker 部署（build context 排除 `.git`），不存在现成的 tag→deploy 注入点。release 的既有动作是 `scripts/set-version.sh <version>`（写 Cargo/npm 版本，不打 tag）。

## Requirements

- **R1 ref 解析链**（`web/src/lib`，沿用 `site.ts` 的 env 模式）：`TOKENS_SCRIPTS_REF` env（部署覆盖/逃生口）→ 仓内常数（release 时推进）→ 兜底钉版 SHA `4921ccbe…`。运行时不做 git 调用、不请求 GitHub API。
- **R2 兼容不变式**：一个 tag 只有在其 commit **同时包含** `pre-install-tokens.sh` 与 `pre-install-tokens.ps1` 时才允许被 docs 引用；否则保持 SHA 兜底。实现上不扫描历史 tag——常数只由 release 流程向前推进，天然满足该不变式；`set-version.sh` 写入前校验两脚本在树内存在，缺失则拒绝写入。
- **R3 `set-version.sh <version>` 扩展**：打 tag 前把 ref 常数写成未来的 `v<version>`，并同步替换 `README.md` / `README_zh.md` / `docs/deploy/tokens-cli-usage.md` 中的旧 ref 字符串；替换后校验每处命中计数，0 命中报错退出。
- **R4 `enable-tokens-service.sh`（根目录，macOS 不适用——brew services 同理但脚本只管 Linux systemd；macOS 用注记引导）**：显式站点参数（origin 校验沿用 preinstall 同款正则）；`~/.config/systemd/user/tokens.service` 不存在时用**本站** `install.sh` 生成（不用上游 s.ee 移动目标）；写 drop-in `tokens.service.d/override.conf`（**不改 base unit**——`install.sh` 重装会重写它）；先写 `Environment=TOKENS_API_URL=<site>` 再 `daemon-reload && enable --now tokens`；提示 `loginctl enable-linger`；凭据检查（credentials.json 存在性，无上报副作用，缺失 → 提示先重开终端 login）；**不 logout**；幂等；`--dry-run`（dry-run 跳过平台/依赖检查以便开发机演示）。
- **R5 `register-tokens-submit-task.ps1`（根目录）**：`-Site` 必传（同款校验）；`-IntervalMinutes` 默认 30；`Get-Command tokens -CommandType Application` 解析绝对路径（与 preinstall 同一查找）；runner ps1 写到稳定路径（`$env:LOCALAPPDATA\tokens\tokens-submit.ps1`），内设 `$env:TOKENS_API_URL` 并调绝对路径 submit；注册计划任务 `powershell.exe -NoProfile -WindowStyle Hidden -File <runner>`；`-Remove` 卸载回滚；**不 logout**；幂等；`-DryRun`。
- **R6 web build 链**：`web/package.json` build 增加两个 `cp`；`web/.gitignore` 增加两个 `public/` 副本条目。
- **R7 文档**：手册 `tokens-cli-usage.md` §3 增 tag ref 机制 + release 交接（set-version 顺序：set-version → commit → tag → deploy）+ 两新脚本用法；docs 页 Linux / Windows Tab note 指向常驻脚本（本站下载地址）；README 双语钉版措辞同步。
- **R8 BDD/e2e**：`docs-page.feature` 更新（ref 钉 tag 或 commit、两脚本可从本站下载、常驻脚本不替代 login）；e2e 的 `[0-9a-f]{40}` 正则放宽为 tag-or-SHA 交替，新增两脚本 URL 断言。

## Constraints

- CLI 契约不破坏，Rust 零改动；不动 publish-cli.yml 发布语义。
- 脚本安全沿用 preinstall 同款约束：显式站点参数、拒绝占位符、dry-run、幂等。
- 新脚本不做 login（device flow 必须用户本人授权），不做 logout（凭据复用是设计前提：凭据单槽位、不记所属站——README.md:88-89）。
- tag 语义视同钉版 commit（不可变 ref），"不跟随分支漂移"原则不变。
- Windows 脚本本机无法真机验证：静态审查 + `-DryRun` 输出 + 用户真机复验，风险如实记录。

## Acceptance Criteria

- [x] 默认（无 env）下 docs 页 preinstall URL 仍为 `4921ccbe…` SHA；设 `TOKENS_SCRIPTS_REF=v9.9.9-test` 后页面 URL 改用该 tag（`:3100` 实例页面级验证）
- [x] `set-version.sh` 用假版本演练：常数与三处 markdown 的旧 ref 全部替换为 `v<version>`；任一 0 命中即报错；脚本缺失时拒绝写入（临时副本演练，含 fail-closed）
- [x] `enable-tokens-service.sh --dry-run` 打印计划动作；实跑（systemd 容器）后 `systemctl --user is-enabled tokens` 且 `override.conf` 含正确 `Environment`，且服务不携带旧上游指向
- [ ] `register-tokens-submit-task.ps1 -DryRun` 输出正确；静态审查通过；真机验证列为用户侧遗留项（静态审查已过；无 pwsh，dry-run/真机未跑）
- [x] `bun run build` 后 `web/public/` 出现两脚本，站点 `/enable-tokens-service.sh` 可下载（含 `/register-tokens-submit-task.ps1`，内容含撇号转义修复）
- [x] `bun run lint` / `bun run typecheck` / `bun run test:migrations` 绿；docs e2e 绿并按报告契约出命名报告 + 同 stem 中文 MD

## Gate Records

- **grill-with-docs**：未完整调用（环境未暴露该 Skill）；需求边界由用户逐轮澄清与本评估（tag 内容验证、发布链核查）建立
- **DDD Boundary Review**：not-required——无业务术语/领域模型变化（工具链与部署脚本）
- **DDIA Data Design Review**：not-required——无持久化数据、schema、queue、缓存变化（systemd/计划任务是用户机器配置，非应用数据）
- **Refactoring Pass**：required——修改既有生产代码（`docs/page.tsx`、`set-version.sh`、`web/package.json`），首次编辑前执行
- **Release Readiness**：required——docs 生产页行为 + 用户机器运行时配置脚本变更，完成前执行
