---
schema_version: 1
id: 20261025-v104-f-release
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-10-24 20:09 群内确认（最终方案），Lord 记录；编排方整理 PR F 范围，本会话明确选择 strict。
status: "checking"
branch: chore/v1.0.4-f-release
created_at: '2026-10-25T00:59:44.530970+08:00'
updated_at: "2026-10-25T01:14:20.106358+08:00"
completed_at: null
---
# PR F — v1.0.4 发布准备与积压回填

## 目标

为已合并的 A→B→C+D→E 整理 v1.0.4 版本、双语 CHANGELOG 与积压记录。本轮只做发布准备，不打 tag、不发布。

## 已确认范围

- 用户 2026-10-24 20:09 群内确认（最终方案），Lord 记录：分 PR A→B→C+D→E→F 同步并发布 v1.0.4；本轮为 F。
- 编排方整理：十个独立提交，先 E，再 A、B、C+D；各提交作者和 committer 均为 640，邮箱 `60886434+KunoLu@users.noreply.github.com`，仅使用命令级 Git 身份，不改 git config。
- strict；未完整调用 grill-with-docs：范围、来源与验收明确，无模型歧义；不重复需求访谈。
- 本 omp 会话自身不执行 push 和开 PR；push 与开 PR 由编排方在 omp 之外执行。不 merge、不改 PR 正文、不访问生产、不读取或打印真实密钥。
- 生产源码只改两个 `settings.devices.desc` 字符串与版本脚本负责的 ref；不改掩码逻辑、安装器逻辑、workflow、vercel、根／web package.json、bun.lock 或 migrations。
- 旧任务只回填合并事实，保留原分支归属、旧事件及历史门禁结论；done 表示实现与修正已合并，不将旧 blocked 门禁追认成发布通过。B、C+D 关闭已被本轮范围明确纳入，清单旧疑问不构成未决范围。

## 进度

- 初始 HEAD／origin/main：`fab72885b91a09742901d3c2e59652491e5cf0eb`；分支 `chore/v1.0.4-f-release`，初始工作区干净。
- 已完成前九个提交。检查对象为“提交 10 之前的 HEAD（提交 9）”：`ab15d118fe34fdd307ae41b556db5e5d1ec8d2ec`；检查时只有 F task/index 文档未提交，不冒充最终 HEAD 或 CI 证据。
- 第 10 个提交仅 F task 与 index；ignored `.sbtd/active-task.json` 只保存 schema_version、task_id、task_path，strict 模式属于本 task。
- 四个旧 task 的完成时间均为本次真实 +08:00 微秒时钟，completed_at = updated_at = done 事件 at；merge 的 %cI 与 mergedAt 另列于事件 evidence。E 合并前评审结论不可考，不补造 CLEAN。

## 门禁

### Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill、无领域歧义 | 需求 | not-required |
| book-ddia-data-design | 不修改持久数据、缓存格式、schema 或数据契约；仅描述已合并能力 | 设计 | not-required |
| book-legacy-change-safety | 既有设备名说明与短名行为不一致 | 首次文案修改前 | passed（characterized；normal） |
| book-refactoring-pass | 修改既有 TS 字典值及版本 ref | 首次生产编辑前 | passed（proceed；normal；no refactor needed） |
| book-release-readiness | 版本发布准备、cloud 安装指引 | 验证后 | blocked（基线 fmt、未发布 tag／资产、真实部署未验证） |

### Legacy Change Safety Review

- Status: characterized；Review mode: normal。读取 `maskDeviceName` 并在修改前真实执行，`a`／`abcd` → `a***`，`abcde` → `ab****de`，四字中文 → `设***`；旧描述遗漏短名分支。
- 保留行为：trim、Unicode code point 计数、短名首字符＋三个星号、长名前后两字符＋四个星号，以及 cloud 固定标签／provider 分组均不改变。
- 安全网：既有 `cloud-labels.test.ts` 与完整 Bun tests；修改后七个真实函数边界 smoke 通过，en/zh 实际字典输出人工语义一致。不新增只断言文字的永久测试，不新增 seam；不是逻辑修复的 red→green。

### Refactoring Review

- Status: proceed；Review mode: normal；Decision: no refactor needed。复用既有字典与 `scripts/set-version.sh`，不改 key、调用点、掩码算法或安装器。
- Code Readability Review：两条翻译明确区分四字符边界；cloud 文档区分脚本托管站、二进制来源和 API 目标。没有新增手写生产逻辑、抽象或依赖。
- Ponytail：保留最小文字／文档／清单变更；非平凡生产重构不适用，不扩展为安装器或全仓格式清理。

### BDD 与工具边界

- BDD: skipped（本轮不改变既有运行规则，仅改说明与版本）；不新增或修改 `.feature`。既有中文场景＋英文关键词保持不动；本地 Rust/Bun 测试不是 Gherkin runner。
- Cross-repo context: contract-only（上游补丁与已合并 PR 原文）；API Contract: not-needed（本轮无 API 变更）；Mock Strategy: contract-backed（安装器 fixture 基于真实 fetch 参数与归档结构）；E2E Mode: smoke-only。
- Chrome/Playwright: skipped，本轮未启动带数据库／认证的设置页，真实 UI 布局未验证；真实字典与函数 smoke 不冒充浏览器验收。Maestro: not-needed，无移动端。
- Graft: skipped，无 callable managed connection；LSP: not-needed，没有符号或调用关系变更。rtk: skipped-for-report，使用原生命令保留完整 stdout/stderr/exit。
- 独立复核：passed；ReviewReleaseDocs 对文档／翻译／安装链路判定 CLEAN，ReviewRecordsVersions 对提交 1–9 的记录／版本边界判定 correct；两路均无发现项。ReviewRecordsVersions 随后复核本 F task 草稿、index 与 active，结论 clean、无发现项；最终 checking 事件与提交由主会话完成。原生只读 reviewer 不运行测试，不启动持久协作 runtime；验证环境由主会话唯一控制。结果摘要：reports/independent-review.json 与同 stem 中文汇总。

### Release Readiness Review

- Status: blocked；本地功能命令通过不等于已发布。无 CI 验证、未打 tag、未发布、真实部署未验证；全量 fmt 保留基线失败。
- Production path：脚本 ref 与清单推进至 v1.0.4；cloud 文档指导安装与 API 目标，不修改任何服务端规则。
- Failure modes/safeguards：v1.0.4 钉版下载地址在 tag 发布前不能当可用发布证明；cloud 环境必须设置 `TOKENS_API_URL=https://tokens.camp`，使用专用 token，不复用本机 token。本文不记录真实 token。
- Capacity/limits：没有新增 runtime 路径、队列、重试或服务端容量变化；已存在 cloud 每轮 submit 与 teleport 重复计数限制保持，未做真实容量测试。
- Observability/runbook：检查 `tokens --version`、API 目标及 cloud status；不要仅以 hook 的 exit 0 判断上传成功（原 hook 保留错误吞并策略）。
- Rollout/rollback：编排方另行处理最终 HEAD 检查、PR、tag、资产和部署。Git 回退不保全 ignored 缓存／报告，也不撤销已提交的用量；旧 CLI 从日志重建缓存的限制已写入双语 CHANGELOG。
- Required validation：Rust 2456 pass／0 fail／4 ignored；Bun 51 pass／0 fail；clippy -D warnings、check、build、typecheck 成功；真实二进制输出 tokens 1.0.4。fmt 18 文件／37 hunk 与提供的基线完全一致。
- 额外风险：本站提供的 install.sh 仍硬编码上游 missuo/tokens 二进制来源；本轮文档已如实披露，未擅改安装器，也未声称 fork 二进制分发已完成。没有任何 owner 风险豁免或生产验收记录。

## 条目与来源

来源主清单：`/workspace/tokens-v104-f-verify/f-items.md`（编排方整理，2026-10-25 01:00）；PR 原文为同目录 `pr-reviews/pr13.json` 至 `pr16.json` 的 `.body`。群内 transcript 不可读，群内时间与评审结论依据该清单转述的 bot 记忆，不冒充本会话亲历。

| 条目 | 来源 | 提交 | 处理 |
|---|---|---|---|
| 1 版本与脚本 ref | 编排方整理；set-version.sh | 提交 8 | 16 文件，1.0.4；依赖版本未升级 |
| 2 双语 CHANGELOG | #13–#16 原文；1.0.3 先例 `121680e23236d0762e9099febd61eb1685c72e26` | 提交 9 | Added/Fixed/Changed 与中文一致，#11 移入 1.0.4，Unreleased 留空 |
| 3 A 原范围注释 | Reviewy／Lord 群内 2026-10-24 20:40，清单转述 | 提交 2 | 原 :23 全句保留，末尾加会话自身限制注释 |
| 4 A 关闭与 s1 | 编排方整理；CodeX/Reviewy/Lord 2026-10-24 10:09 规则，清单转述 | 提交 2 | #13 checking→done，50／127，去掉本会话标记 |
| 5 B cloud import 措辞 | Reviewy 群内 2026-10-24 22:38，清单转述 | 提交 3 | 改为上游冲突上下文、本仓本来没有；全仓同类旧措辞仅一处 |
| 6 B 关闭与 s1 | 编排方整理，沿同一关闭规则 | 提交 3 | #14 checking→done，45／94 |
| 7 README Paseo | #15 原文与 C+D task；上游 `427b51421eb3057f785863162e9a2c5f3bb23b95` | 提交 7 | 英文原段落、中文翻译；不引入图标表；catalog 54 不变 |
| 8 C+D 23:42 漏记注释 | Reviewy 群内 2026-10-25 00:09，清单转述 | 提交 4 | 新事件补记 23:16 的原提交 push 前改写注释；23:42 及全部旧事件原行不动 |
| 9 C+D 关闭与 s3 | 编排方整理，沿同一关闭规则 | 提交 4 | #15 checking→done，24／55 |
| 10 E 关闭与 s2 | 编排方整理；第一个提交仅回填上一个 PR 合并 | 提交 1 | #16 checking→done，10／20；评审结论不可考 |
| 11 短设备名文案 | Reviewy 笔记 Non-blocking 3；Lord 2026-10-25 00:41 群内裁定（清单转述） | 提交 5 | 仅两个翻译值；短名首字符、长名首尾两字符 |
| 12 cloud 安装与站点 | Researchy 群内 2026-10-25 00:35；Lord 2026-10-25 00:38 群内裁定（清单转述）；Reviewy Non-blocking 2 | 提交 6 | 去掉 npm 安装和 tokens.ci，设置 API URL；披露实际上游二进制来源，禁用 systemd 安装 |
| 13 #16 过时 push 措辞 | Reviewy 238b0dd 笔记，经清单转述 | 本轮无新增修改 | 已在 #16 s2 head `3e54a7b4c868ceeaf4f2180ccbda4c43c2814d8e` 修复 |
| 14 #16 Non-blocking 2/3 | 同上 | 提交 6／5 | 对应条目 12／11，不重复处理 |
| 15 #15 非阻塞与 PR 正文 | 群内 2026-10-25 00:01／00:09，清单转述 | 提交 4；PR 正文不改 | CLEAN 留项为条目 8；正文更新不在 F 范围 |
| 16 CodeX 评审失败 | pr-reviews/pr*.json | 无 | #13–#16 因用量上限失败，没有发现项；不等于评审通过 |
| 17 上游 Cloud agent sessions README 段 | E task 丢弃内容；编排方整理 | 无 | 未纳入 F，双语 README 不补此段；不改 PR 正文 |

## 提交顺序与身份

所有提交命令均为 `git -c user.name=640 -c user.email=60886434+KunoLu@users.noreply.github.com commit …`；前九个完整身份已检查。第 10 个提交使用同一身份，提交内不自引用尚未产生的 SHA。

| # | SHA | 标题 |
|---|---|---|
| 1 | `1226233aec5b317bd6e3350caabdca5801e068d0` | docs(ai): close v1.0.4 PR E task after #16 merge |
| 2 | `d41e575504a101347a16a31b01e87a54aedcff4d` | docs(ai): backfill and close v1.0.4 PR A task |
| 3 | `2b57527eb628bdc94fa15ca11bb76cb0c1fd2bac` | docs(ai): backfill and close v1.0.4 PR B task |
| 4 | `ad2f42186a20b98c1da049783b75783accfe4ef8` | docs(ai): backfill and close v1.0.4 PR C+D task |
| 5 | `e4fa5a5579357d73e4904539b09a2c97c0608fca` | fix(web): describe the short device name mask accurately |
| 6 | `5f81eb7fa5208c45c0b27b94ebf9e9a95f21da59` | docs: point cloud session setup at this fork's CLI and site |
| 7 | `1d16e5b68767e4c8a7dfb23e91db52dfaabac4d6` | docs(readme): document Paseo-managed usage |
| 8 | `8c1e5a687df6387a0aa11e2e2291edb0c94abf98` | chore(release): version 1.0.4 |
| 9 | `ab15d118fe34fdd307ae41b556db5e5d1ec8d2ec` | docs(changelog): add 1.0.4 |
| 10 | 提交后由编排方／交付记录核对，不在文档内自引用 | docs(ai): add v1.0.4 PR F strict task record |

## set-version 改动文件与输出

执行 `PATH=/workspace/tokens-tools/rust-1.90.0/bin:$PATH OPENSSL_NO_VENDOR=1 scripts/set-version.sh 1.0.4`，exit 0。完整原始输出见 `reports/set-version.json` 与同 stem 中文汇总；输出如下：

```text
  packages/cli/package.json -> 1.0.4
  packages/cli-darwin-arm64/package.json -> 1.0.4
  packages/cli-darwin-x64/package.json -> 1.0.4
  packages/cli-linux-arm64-gnu/package.json -> 1.0.4
  packages/cli-linux-arm64-musl/package.json -> 1.0.4
  packages/cli-linux-x64-gnu/package.json -> 1.0.4
  packages/cli-linux-x64-musl/package.json -> 1.0.4
  packages/cli-win32-arm64-msvc/package.json -> 1.0.4
  packages/cli-win32-x64-msvc/package.json -> 1.0.4
  web/src/lib/scriptsRef.ts: v1.0.2 -> v1.0.4 (1)
  README.md: v1.0.2 -> v1.0.4 (1)
  README_zh.md: v1.0.2 -> v1.0.4 (1)
  docs/deploy/tokens-cli-usage.md: v1.0.2 -> v1.0.4 (2)
  docs/deploy/tokens-cli-usage.en.md: v1.0.2 -> v1.0.4 (2)
set-version: 1.0.4
```

脚本实际修改 16 文件（`git status --porcelain` 证据 `reports/set-version-status.json`；另有 F task/index 未提交，不属于脚本改动）：

- `cli/Cargo.toml`
- `cli/Cargo.lock`
- `packages/cli/package.json`
- `packages/cli-darwin-arm64/package.json`
- `packages/cli-darwin-x64/package.json`
- `packages/cli-linux-arm64-gnu/package.json`
- `packages/cli-linux-arm64-musl/package.json`
- `packages/cli-linux-x64-gnu/package.json`
- `packages/cli-linux-x64-musl/package.json`
- `packages/cli-win32-arm64-msvc/package.json`
- `packages/cli-win32-x64-msvc/package.json`
- `web/src/lib/scriptsRef.ts`
- `README.md`
- `README_zh.md`
- `docs/deploy/tokens-cli-usage.md`
- `docs/deploy/tokens-cli-usage.en.md`

- workspace version、Cargo.lock 的 tokens-cli/tokens-core、9 个 npm 包与 launcher 的 8 个 optionalDependencies 均为 1.0.4；5 个 ref 文件 v1.0.2→v1.0.4。
- Cargo.lock 只有两个 workspace crate version 行变化；脚本已同步，无需另跑 cargo update；第三方依赖变化为 0。
- 根 package.json 保持 1.0.0；web/package.json、bun.lock、workflow、vercel 均不变。

## CHANGELOG 摘要

- A（#13）：上游 v27.1.4 Antigravity 隐式同步、共享 RPC 预算、部分失败结果与 Windows helper 修复。
- B（#14）：速度后端与 v9 缓存；明确降级到旧 CLI 重建本地缓存，仅存缓存而源日志已删的历史不能恢复，建议同时备份两者。
- C+D（#15）：profile 构成占比／速度档位、官方 OMP 图标、Paseo 原生 provider 目录发现。
- E（#16）：cloud submit/status、Claude Code cloud、Codex cloud 不支持、cloud_ 服务端规则与 provider 分组。
- 本轮：短名掩码、cloud 文档安装／站点说明、README Paseo、版本及脚本 ref；#11 main Vercel 自动部署条目移入 1.0.4。en/zh 内容一致，Unreleased 标题下没有条目，不声称发布已经发生。

## 安装链路与文案核对

- `web/package.json` build 把根 install.sh 复制到 public/install.sh；根脚本 `REPO="missuo/tokens"`，从 GitHub API 查 latest，再下载平台 tar.gz 及校验文件。并不是从本站下载 fork 二进制。
- README 的 Linux 安装思路是本站 `/install.sh`。cloud hook 使用 Bash 而非 README 的 sh，因为脚本有 `set -euo pipefail`；设置 `TOKENS_NO_SERVICE=1`，保留已有 hook 的条件与最终 true。不修改 README 原安装命令或安装器逻辑。
- 放行说明改为 tokens.camp、api.github.com、github.com、release-assets.githubusercontent.com；pricing 的 models.dev/openrouter.ai 保留。未访问生产核实实际部署脚本，GitHub 下载重定向未实网验证。
- 环境说明与 Stop hook 均固定 `TOKENS_API_URL=https://tokens.camp`；专用 token 仅说明变量名；Codex 不支持、服务端规则、Tested 记录等其余章节保持不动。Tested 是上游历史记录，不是本仓本轮验收。
- README／docs／web 搜索同类掩码描述仅命中 t.ts 两条，无其他用户文档需改；实际短名与长名分支验证通过。
- scanner catalog 54 个条目，clients.rs 的 COUNT 断言为 54；clients.rs 和 web constants 均无 Paseo 客户端 ID。Paseo 归属底层 Claude/Codex，因此不改 README 数字；英文 Paseo 段落与上游逐字相等。

## 验证环境与结果

检查对象均为提交 10 之前的 HEAD（提交 9）`ab15d118fe34fdd307ae41b556db5e5d1ec8d2ec`，仅 F task/index 文档 dirty；Linux x64、Bun 1.4.2、Rust/Cargo 1.90.0。命令进程内设置 PATH、OPENSSL_NO_VENDOR=1、OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu、OPENSSL_INCLUDE_DIR=/usr/include、CARGO_BUILD_JOBS=4，不改全局环境。

Evidence Source: developer-local；Source Revision: dirty（F 文档）；Environment Alignment: verified（本地命令／fixture）；Evidence Publication: local-only。报告目录先经 git check-ignore，JSON 保留命令、cwd、完整 SHA、真实时间、耗时、exit、完整 stdout/stderr 与环境；均配同 stem 中文汇总。不是 CI 或 full-stack 证明。

| 命令／检查 | 结果 | 原始证据 |
|---|---|---|
| bun install --frozen-lockfile | exit 0；658 packages，lock 不变 | reports/final-bun-install.json |
| cargo test --manifest-path cli/Cargo.toml --workspace | exit 0；2456 pass／0 fail／4 ignored | reports/final-cargo-test.json |
| cargo clippy --manifest-path cli/Cargo.toml --workspace --all-targets -- -D warnings | exit 0 | reports/final-cargo-clippy.json |
| cargo check --manifest-path cli/Cargo.toml --workspace --all-targets | exit 0；upstream_policy 补充要求 | reports/final-cargo-check.json |
| cargo build --manifest-path cli/Cargo.toml -p tokens-cli | exit 0 | reports/final-cargo-build.json |
| cli/target/debug/tokens --version | exit 0；tokens 1.0.4 | reports/final-cli-version.json |
| cargo fmt --manifest-path cli/Cargo.toml --all --check | exit 1；18 文件／37 hunk，基线相同；无新增 | reports/final-cargo-fmt.json；reports/fmt-comparison.json |
| bun run --cwd web typecheck | exit 0 | reports/final-web-typecheck.json |
| bun run --cwd web lint | exit 0；0 error／1 旧 warning | reports/final-web-lint.json |
| cd web && bun test | exit 0；51 pass／0 fail；4579 expect；8 文件 | reports/final-web-tests.json |
| 真实 maskDeviceName＋t 字典 smoke | 7 个边界通过，en/zh 实际输出语义一致 | reports/mask-i18n-smoke.json |
| SessionStart／Stop 隔离 shell smoke | 4 场景通过；实际 install.sh＋fixture transport/binary，无真实 HTTP 或 token | reports/cloud-hook-smoke.json |
| git diff --check origin/main..HEAD | exit 0，输出为空 | reports/code-diff-check.json |
| 受保护文件 stat／migration／新增禁用措辞与路径 | 输出为空／0 改动／0 命中 | reports/protected-files.json；reports/code-boundaries.json；reports/acceptance-audit.json |
| 历史事件、版本与 README 段落 | 旧行 byte-identical；版本全对齐；英文 Paseo 与上游一致 | reports/acceptance-audit.json |
| test:migrations／test:teams／test:private | skipped，无本地 DATABASE_URL，不建库、不连生产；无 migration 修改 | 未执行，不计通过 |
| 真实 cloud 容器／浏览器设置页／部署 | skipped，本轮无真实环境验收 | 未执行，不计通过 |

- Rust：tokens-cli 204、tokens-core 2198、integration 54；4 ignored 为既有本地数据用例。
- lint 唯一旧 warning：docs/page.tsx:326 no-img-element；本轮不改此页。
- fmt 只读既有 `/workspace/tokens-v104-f-verify/base-fmt.out` 与 base-fmt-files.txt；未重跑已知失败的基线。去掉仓库绝对路径后诊断全文完全一致，18 文件／37 hunk；新增文件／hunk 都为 0。
- Final Test Report: generated；Run Summary MD: generated；Targeted Rerun: not-needed（无本轮新增失败）；Final Full Rerun: failed（仅已知 fmt 基线）。编排方在提交 10 后最终 HEAD 再完整运行，当前不声称已执行该复跑。

## 模型记录

| # | 会话 | 模型 | 请求／工具调用 | 产出 | 会话记录 |
|---|---|---|---|---|---|
| 1 | ping | openai-codex/gpt-6-astra（high） | 1／0 | 连通性检查（--no-tools） | `~/.omp/logs/tokens-v104-f/ping/sessions/2026-10-24T16-57-15-766Z_01a16ed8-5cf5-7492-95e2-aa944ff90910.jsonl` |
| 2 | s1 main（本会话） | openai-codex/gpt-6-astra（high） | 未知（落盘时本轮尚未结束），实际计数见群内交付报告 | 版本号、CHANGELOG、回填、task.md、本地检查 | `~/.omp/logs/tokens-v104-f/s1/sessions/` |

## 下一步

F 保持 checking、completed_at null。由编排方在最终 HEAD 完整复跑后推进外部 PR／评审与发布流程；本会话不打 tag、不 push、不发布。安装器仍下载上游二进制与未验证的真实部署／cloud 链路是明确恢复边界；不在本轮擅改。ignored reports、.sbtd、node_modules、cli/target 不随 Git 保存，跨环境须保留报告或重新运行，不承诺自动恢复。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-10-25T01:00:14.803432+08:00 | planned | in-progress | 编排方整理，范围与来源核对完成，开始十个顺序提交。 | origin/main fab72885b91a09742901d3c2e59652491e5cf0eb；f-items.md；PR #13–#16 原始 JSON；历史 task 与版本脚本。 |
| 2026-10-25T01:14:20.106358+08:00 | in-progress | checking | 编排方整理，前九个独立提交、F 文档与本地计划检查完成，独立只读复核无发现项；保留基线 fmt、上游安装器来源及真实部署／发布边界，不标记 done。 | 提交 10 之前的 HEAD（提交 9）ab15d118fe34fdd307ae41b556db5e5d1ec8d2ec；Rust 2456 pass/0 fail/4 ignored；Bun 51 pass/0 fail；CLI tokens 1.0.4；fmt 18 文件/37 hunk 与提供的基线全文归一化相同；reports/final-*.json；reports/fmt-comparison.json；reports/independent-review.json。 |
