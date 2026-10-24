---
schema_version: 1
id: 20261024-v104-b-speed-backend
workflow_mode: strict
mode_source: user
mode_note: 用户 2026-10-24 20:09 群内确认（最终方案），Lord 记录；本会话只做 PR B，保持 strict。
status: "checking"
branch: feat/v1.0.4-b-speed-backend
created_at: '2026-10-24T22:08:44.876292+08:00'
updated_at: "2026-10-24T22:18:13.146148+08:00"
completed_at: null
---
# PR B — speed backend 上游同步

## 目标

仅同步三个 inference speed backend 提交；保持别名、总量、私密边界与旧客户端兼容。

## 已确认范围

- 用户 2026-10-24 20:09 群内确认（最终方案），Lord 记录。
- strict；未完整调用 grill-with-docs：提交与验收边界明确，无领域歧义。
- 不改 README、CI、版本、web/vercel.json、部署文件与 CHANGELOG；降级会自动重建本地缓存的说明进入后续发布 PR F。
- 本 omp 会话自身不执行 push 和开 PR；push 与开 PR 由编排方在 omp 之外执行。不启动子代理。

## 进度

- 基线 `origin/main` 与初始 HEAD 均为 `df417b26c468f07491f486f230f01936c6b88742`，工作区干净，分支匹配。
- 三个 cherry-pick 与一个最小 style 提交已完成。最终代码 tip／全部检查对象：`105d527433d5211b8acdc0793cc1a3b223a5884f`。
- 后续 docs 提交只增加 task、index、BDD 文档，不改代码。检查时仅这些文档未提交，不把代码 tip 的证据标成 docs 后 HEAD 的 CI 证据。

## 门禁

### Book Gate Plan

| 门禁 | 触发事实 | 阶段 | 状态 |
|---|---|---|---|
| book-ddd-distilled-modeling | 无完整 grill、无领域歧义 | 需求 | not-required |
| book-legacy-change-safety | 别名合并与持久缓存兼容 | 首次行为变更前 | passed（characterized；源码与上游回归安全网） |
| book-refactoring-pass | 既有 Rust/TS 生产代码 | 首次生产变更前 | passed（proceed；不额外重构） |
| book-ddia-data-design | 缓存 8→9、JSONB 可选字段、CLI/web contract | 设计稳定前 | passed（confirmed；实测结果另列） |
| book-release-readiness | submit 与缓存运行行为 | 验证后 | blocked（基线 fmt 失败、独立复核及真实数据库链路缺口；不宣称发布就绪） |

### Legacy Change Safety Review

- Status: characterized；Review mode: normal。首次拣入前已核对基线 route 的别名合并及旧缓存 Stale 分支。
- Behavior to change：解析 Claude reported / Codex selected 速度，模型级速度计数贯穿缓存、提交、JSONB 和公开读取。
- Behavior to preserve：模型身份、计价、总量、八项模型／客户端计数、kilocode→kilo、provenance、私密访问规则、旧 payload 哈希。
- Safety net：上游 `inference_speed.rs`、`speedUsage.test.ts`、`publicProfileData.test.ts` 与缓存迁移测试；另执行真实新旧二进制 dry-run。
- Hidden dependencies / seam：本地日志、缓存格式、parser revision、数据库读取。沿用已有解析／合并函数与隔离 fixture；不新建生产 seam。
- TDD 边界：按三个原子上游提交同步，未拆提交执行 failing-before red；不宣称完整 red→green。基线行为由源码和 blob 一致性证明，新增回归与双二进制运行作为验收。

### Refactoring Review

- Status: proceed；Review mode: normal。
- Existing-code scope：Rust parser/cache/aggregation/CLI payload 与 web submit/helpers/validation/profile。
- Decision：no additional refactor needed；接受上游将写入别名折叠集中到 helpers 的结构调整，route 删除本地重复定义。
- Safety net：别名、缺失／显式空速度、旧哈希、跨设备读取及缓存迁移回归。没有抽象新模块、依赖或旁路实现。
- Deferred refactors：保留基线格式问题及其他 PR 的 cloud/UI/部署范围。

### DDIA Data Design Review

- Status: confirmed。Source of truth：源日志及缓存保留历史供 CLI 重放；web 持有提交后的按设备／日期 JSONB，所有权与事务边界不变。
- Write/read paths：parser → UnifiedMessage → 缓存与聚合 → CLI payload → validation → helpers → JSONB；公开读取通过 helpers 与 mergeSpeedUsage 合并模型维度。
- Consistency：已知速度是总量子集，不改变总量或费用。同 revision 旧提交省略 speedUsage，且该模型 tokens/messages 不倒退时保留旧速度；显式 `[]` 可清空；新 revision 的纠错不继承此保留规则。
- Idempotency：非空速度参与哈希并按 speed/source 规范排序；缺失／空数组省略哈希键，旧哈希不变。别名与 provenance、revision floor、同设备回归保护沿用。
- Schema/migration：JSONB 仅增加可选类型，无 SQL/DDL；本地缓存 8→9 实际在第一个提交中发生，第三个提交改为 ModelSpeed 结构体。Claude v8 wire migration 保留缓存历史与 cache_write_1h，再扫描一次现存日志以发现速度；Codex parser 10→11 触发重建。
- Compatibility：新 web 接受旧 payload；旧 web 的非 strict Zod 对象 strip 新字段，不因 speedUsage 拒绝。后台能力本 PR 不包含 UI tier 展示，REPORT §4.3 的 getSpeedTierRows 与 UI 行属于后续 PR。
- Rollback/replay：旧二进制读 v9 envelope 后判为 Stale，静默从日志重建 v8；新二进制可再迁移 v8。无数据库逆向迁移。完整日志 fixture 的总量不变不能外推为仅缓存保存的 compacted 历史也能无损降级；旧程序不会解码这些 v9 历史，回退前应保留缓存和源日志备份。
- Observability：损坏缓存走 Invalid 警告；未知格式／parser revision 走 Stale，正常重建不警告。核对退出码、stderr、分片版本和统计，不能只看 exit 0。

### BDD 与工具边界

- BDD 文档：`cli/features/inference-speed.feature`，中文场景 + 英文关键词；实现前已保存。仅 PR B，不做全仓 BDD 同步。
- Cross-repo context: contract-only；API Contract: verified（本仓 schema、helpers 与上游 payload）；Mock Strategy: contract-backed（profile 数据库查询 fixture），CLI smoke 使用真实二进制与合成日志。
- E2E Mode: backend-only / smoke-only；没有运行数据库、真实 HTTP 鉴权提交、生产或 UI。Chrome/Playwright/Maestro: not-needed（本 PR 无 UI/mobile 变化，数据库链路另记 skipped）。
- Graft: skipped（无 callable 连接）。LSP: not-available（实际 references 请求返回 No language server found），改用源码与 Git blob/diff。
- rtk: skipped-for-report；使用 native 命令保存未经压缩的 stdout/stderr 与 exit code。
- 独立复核：blocked（跨 CLI/web contract 与持久缓存触发 strict 独立复核；本会话不启动子代理，外部人工评审尚未发生）。自审不冒充独立复核。
- BDD: traceable；Rust workspace 与 Bun tests 已执行关联场景，缓存场景由真实 CLI 四步 smoke 证明；不是 Gherkin runner 执行。
- Code Readability Review：自审通过；ModelSpeed 明确模型绑定、reported/selected 分离，wire legacy 类型保持字段顺序；缺失与显式空数组通过注释和行为测试区分。
- Ponytail Review：未发现本次必须删除的推测性抽象；复用 helpers、parser state 和现有缓存迁移，保留兼容 wire 类型，不扩展 scope。

### Release Readiness Review

- Status: blocked。全仓 fmt 基线仍失败；跨 CLI/web contract 的独立人工评审尚未执行，真实数据库提交链路未运行。不是功能回归失败，也不是发布就绪。
- Production path：Claude/Codex 扫描、缓存、submit/autosubmit/serve payload，web 校验、JSONB 模型合并和公开读取。
- Failure modes：缺失速度保持未知；旧 payload 不清除同 revision 的已有速度；坏计数／重复档位／超总量被校验拒绝；缓存 Invalid 警告，Stale 重建。
- Capacity/limits：每个模型最多 3 种速度 × 2 种来源的 6 个 entry；升级一次补读、降级重建会增加本地解析成本。未引入网络重试、后台队列或部署变化。
- Observability/runbook：观察 CLI 总量、exit、stderr 和分片版本；缓存重建无警告属预期，不能据此推断缓存独占历史仍存在。
- Rollout/rollback：建议 web 先于 CLI；无 DDL。本 PR 不部署、不调版本。回退代码需保留源日志与缓存备份，不能只依赖 Git 回退保存 ignored 数据。
- Required validation：2443 Rust tests、28 web tests、clippy/check/build/typecheck 和真实 CLI smoke 通过；fmt 与独立复核缺口明确保留。没有 owner 风险豁免或生产验证记录。

## Cherry-pick 与冲突处理

| 上游提交 | 本仓新提交 | 结果 | 冲突文件与解决方式 |
|---|---|---|---|
| `42d7185079a244079507e8abd6b8c16c36e1fdeb` | `656b7aac46a7eec7e995b5784e2c60df3f296e9a` | 冲突 | `.github/workflows/ci.yml`、`README.md` 取本仓；`validation/submission.ts` 仅保留 speed import，丢弃 PR E 的 devices/cloud import |
| `1216cdd9d38a77254e89f5a7678df795f7fe2f09` | `88e754b197f590b81351db1d074398623034bec6` | 冲突 | `README.md` 继续取本仓；其余 naming 变更自动合并 |
| `00579f78bc61f542f7dcb7db3ca6fbf56b891e33` | `ff060e4ab0c4416f636a25454ad02fc576cab9ae` | clean | 无冲突 |

- 每条使用 `git -c user.name=640 -c user.email=60886434+KunoLu@users.noreply.github.com cherry-pick -x <SHA>`，continue 同样带命令级身份；未改 Git config。
- 上游作者均保留为 `Amp <amp@ampcode.com>`，committer 为上述 640 身份，保留完整原提交信息与 `-x` trailer。
- 冲突解决后检查均无冲突标记、无 cloud 引用；README 与 ci.yml 相对 origin/main 均无改动。
- 独立 style 提交：`105d527433d5211b8acdc0793cc1a3b223a5884f`。仅格式化上游新增 `codex.rs` 的 owns_snapshot 表达式（+6/−4）；不改行为，不格式化基线既有片段。作者与 committer 都为 640。

## submit/helpers/validation 去重核查

- `df417b2` 与上游 v27.1.3 `059cb69110d01b70e8471b07942081c8779f80c7` 的三个文件完全一致。分别比较两个仓库的 `git ls-tree`：route blob `10b5f0a21e7f382c79b05d79f9042bfb99edcda3`，helpers `1f1466261b632aa5b3a1c71e423c22f7c70638c8`，validation `761bf24770e2d9a9fb8d1881e4a28e57dc29f117`。
- 本 clone 没有 v27.1.3 对象，初次短 SHA diff 返回 bad revision；改为只读上游 clone 的 tag/tree，与本仓 baseline blob 对照，没有 fetch 或修改上游。
- `git diff origin/main -- web/src/app/api/submit/route.ts` 只新增 helpers import 并删除原本地别名／模型合并定义。原调用点 668、732 现在为 609、673，均调用导入的 normalizeClientBreakdownAliases。
- `helpers.ts:168-228` 唯一写路径实现仍累加 tokens、cost、input、output、cacheRead、cacheWrite、reasoning、messages，保留 kilocode→kilo、provenance 重算；模型合并改用 addModelBreakdown，额外合并 speedUsage 且不修改输入数组。
- `publicProfileData.ts:17-19` 的 LEGACY_CLIENT_ALIASES / normalizeClientId 保持原样，336、398 两处读路径仍规范化别名。读路径复用模型计数 helper 不等于删除读别名。
- 对照 `00579f7`：本仓 helpers.ts 与 speedUsage.ts 无差异；route/validation 的差异仅为尚未引入的 cloud sessions 行为，不缺速度逻辑。
- 缺失 speedUsage：validation `.optional()`；mergeSpeedUsage 全空返回 undefined；profile 缺失不生造速度；helpers 仅有值才写，旧提交保留已知速度受同 revision、模型 tokens/messages 不倒退条件约束。显式 `[]` 与缺失不等价。
- `git diff origin/main` 保留本仓 schema 的 teams/password/private 与 profile 访问边界；无 migration、版本、部署或 UI 修改。

## 验证环境与基线证据

- Linux x64；命令进程内设置 `PATH=/workspace/tokens-tools/rust-1.90.0/bin:$PATH`、`OPENSSL_NO_VENDOR=1`、`OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu`、`OPENSSL_INCLUDE_DIR=/usr/include`、`CARGO_BUILD_JOBS=4`；不改全局环境或配置。
- `reports/` 已通过 `git check-ignore`，未跟踪；原始 JSON 逐条保存命令、cwd、完整 SHA、真实开始时间、耗时、exit、stdout、stderr，另有同 stem 中文汇总。本地报告不随提交发布。
- 初轮 `ff060e4` fmt 与只读 `src-old` 的 df417b2 fmt 均 exit 1、18 个失败文件，没有新增失败文件；但原有 codex.rs 中多一处本 PR 新代码格式 hunk，已由独立 style 提交修复。
- 上游 main.rs 顺带删除原文件末尾空行，导致另一处已有空行诊断上下文变化／合并；不把该变化误报为新增格式失败。基线 18 个文件未整体格式化。
- 数据库脚本 `test:migrations`、`test:teams`、`test:private`：skipped；未配置本地 DATABASE_URL，不访问生产、不创建数据库。

## 最终验证结果

全部 final 命令针对代码 tip `105d527433d5211b8acdc0793cc1a3b223a5884f`，文档提交之前运行。Bun 1.4.2、Rust 1.90.0。Evidence Source: developer-local；Source Revision: dirty（仅 task/index/BDD 文档）；Environment Alignment: verified（本地 fixture）；Evidence Publication: local-only。不是 docs 后 HEAD 的 CI 证明。

| 命令 | 结果 | 原始证据（相对本任务目录） | tip SHA |
|---|---|---|---|
| `bun install --frozen-lockfile` | exit 0；658 packages；未改锁文件 | `reports/final-bun-install.json` | `105d527433d5211b8acdc0793cc1a3b223a5884f` |
| `cargo test --manifest-path cli/Cargo.toml --workspace` | exit 0；2443 passed / 0 failed / 4 ignored | `reports/final-cargo-test.json` | 同上 |
| `cargo clippy --manifest-path cli/Cargo.toml --workspace --all-targets` | exit 0；无编译 warning | `reports/final-cargo-clippy.json` | 同上 |
| `cargo check --manifest-path cli/Cargo.toml --workspace --all-targets` | exit 0；额外执行 upstream_policy 的检查 | `reports/final-cargo-check.json` | 同上 |
| `cargo build --manifest-path cli/Cargo.toml -p tokens-cli` | exit 0；产出真实 CLI | `reports/final-cargo-build.json` | 同上 |
| `cargo fmt --manifest-path cli/Cargo.toml --all --check` | exit 1；仍为基线 18 文件，新增失败文件 0；本 PR 新增 hunk 已修复 | `reports/final-cargo-fmt.json`、`reports/baseline-cargo-fmt.json`、`reports/initial-cargo-fmt.json` | final 同上；baseline 为 df417b2，initial 为 ff060e4 |
| `bun run --cwd web typecheck` | exit 0 | `reports/final-web-typecheck.json` | 同上 |
| `bun run --cwd web lint` | exit 0；0 error / 1 warning，既有 docs/page.tsx:324 no-img-element | `reports/final-web-lint.json` | 同上 |
| `cd web && bun test` | exit 0；28 pass / 0 fail，4534 expect，5 文件 | `reports/final-web-tests.json` | 同上 |
| `test:migrations` / `test:teams` / `test:private` | skipped；无本地 DATABASE_URL，不访问生产、不建库 | 未运行，无通过证据 | 不适用 |

- Rust 分项：tokens-cli 196、tokens-core 单元 2193、integration 54；四个 ignored 为真实 CherryStudio/OpenCode 数据与两个本地 Anthropic catalog 测试。
- 关键新增回归：4 个 inference_speed integration 测试；v8 unrelated metadata、v8 Claude retained history、当前 speed/Codex state roundtrip 三个缓存测试；web 7 个 speedUsage contract/merge 与 2 个 profile tests 均通过。
- Bun 输出里的 Redis 故障／超时是既有受控错误路径测试，不是实际外部 Redis 失败。lint 警告文件未被本 PR 改动，PR A 也已记录相同警告。
- 对 final/baseline fmt 去除根路径和行号后，差异仅为上游 main.rs 删除末尾空行引起的已有诊断上下文变化；codex 新增格式 hunk 不再出现。未触碰基线 18 文件的既有失败片段。
- 源码原始对照：`reports/source-boundary-audit.json`、`reports/upstream-comparison.json`；README、CI、vercel、版本、lock 和 migration 边界检查通过。
- Final Test Report / Run Summary MD: generated（逐条 JSON + 同 stem 中文汇总）；Targeted Rerun: passed（新增格式 hunk）；Final Full Rerun: skipped-with-risk（功能命令通过，全仓 fmt 基线失败保留）。

## 缓存降级证据

精确命令：

```sh
cp cli/target/debug/tokens /workspace/tokens-v104-b-cachetest/bin/tokens-new-105d527
/workspace/tokens-v104-b-cachetest/run.sh /workspace/tokens-v104-b-cachetest/bin/tokens-new-105d527
```

- OLD `/workspace/tokens-v104-b-cachetest/bin/tokens-old-df417b2` 已存在，无需等待；由编排方构建，本会话未修改 src-old。NEW 对应完整代码 tip `105d527433d5211b8acdc0793cc1a3b223a5884f`，两个二进制均报告 tokens 1.0.2（本 PR 不改版本）。
- 原脚本未改；实际运行目录 `/workspace/tokens-v104-b-cachetest/run-20261024-221447`，表格 `table.md`，各步 stdout/stderr 与 `cache-after-step1..4` 留存。执行输出另存 `reports/cache-downgrade.json` 与同 stem 中文汇总。
- 脚本 env -i、隔离 HOME/config、假 token、`TOKENS_API_URL=http://127.0.0.1:9`；每步执行 `tokens submit --dry-run -c claude --since 2026-10-01`。不访问生产，不使用真实 token。

| step | binary | version | exit | stdout totals | shard 首字节 / 大小 | stderr bytes |
|---|---|---|---|---|---|---|
| 1 | tokens-new-105d527 | tokens 1.0.2 | 0 | 13,600 tokens；$0.02 | shard-14.bin：9 / 584 B | 0 |
| 2 | tokens-old-df417b2 | tokens 1.0.2 | 0 | 13,600 tokens；$0.02 | shard-14.bin：8 / 580 B | 0 |
| 3 | tokens-new-105d527 | tokens 1.0.2 | 0 | 13,600 tokens；$0.02 | shard-14.bin：9 / 584 B | 0 |
| 4 | tokens-old-df417b2 | tokens 1.0.2 | 0 | 13,600 tokens；$0.02 | shard-14.bin：8 / 580 B | 0 |

- 四步 stdout 原始字节完全一致，分片快照首字节逐一核对为 9/8/9/8，stderr 均空。
- 与 REPORT §6.1 的行为一致；分片名称／大小不同并非错误：CacheKey 的 SHA-256 包含绝对源路径，本次首摘要字节 20（十六进制 14），旧 fixture 路径为 18（十六进制 12）。序列化中源日志与 `.meta.json` 路径各增加 4 字节，合计解释 584/580 相对 576/572 增加 8 B。
- 验证边界：本次完整日志 fixture 的统计不变；没有执行 compacted-history 降级、真实数据库提交或跨平台验证，不把自动重建描述为所有历史数据无损保证。

## 模型记录

| # | 会话 | 模型 | 请求／工具调用 | 产出 | 会话记录 |
|---|---|---|---|---|---|
| 1 | ping | openai-codex/gpt-6-astra（high） | 1／0 | 连通性检查（--no-tools） | `~/.omp/logs/tokens-v104-b/ping/sessions/2026-10-24T14-04-31-822Z_01a16e3a-38ce-72ad-b13d-aca7bd2edf55.jsonl` |
| 2 | s1 main（本会话） | openai-codex/gpt-6-astra（high） | 未知（落盘时本轮尚未结束），实际计数见群内交付报告 | cherry-pick 3 个提交、task.md、本地检查、缓存降级测试 | `~/.omp/logs/tokens-v104-b/s1/sessions/` |

所有编辑和提交由 omp 会话完成；push 与开 PR 由编排方在 omp 之外执行。

## 行为变更说明

- Claude 只采纳明确响应速度；Codex 选择绑定线程、模型与 turn 快照，不以 selected 证明实际服务速度。
- 速度是模型用量子集；未知保持未知，不拆模型、不改变总 tokens 或费用。旧提交哈希与字段缺失行为兼容。
- 缓存格式 8→9，升级保留旧缓存历史并补读现存证据；降级重新解析日志。CHANGELOG 不在本 PR 改，降级会自动重建本地缓存的说明进入后续发布 PR F。

## 下一步

由编排方在 omp 外执行 push 与开 PR，进入独立人工评审；本记录保持 checking、completed_at 为 null。基线格式问题、真实数据库链路与仅缓存历史降级边界必须继续披露。后续 C+D/E/F 与发布不在本会话；CHANGELOG 的降级说明进入 PR F。ignored 报告、NEW/OLD 二进制和 run 目录仅本地保留，Git 提交不保存它们，跨环境恢复须保留这些证据或重新验证。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-10-24T22:08:45.932139+08:00 | planned | in-progress | 分支与范围核对完成，开始实现前安全核查 | origin/main df417b26c468f07491f486f230f01936c6b88742；三个单父上游补丁；三个 web 基线 blob 与 v27.1.3 一致 |
| 2026-10-24T22:18:13.146148+08:00 | in-progress | checking | 三个上游提交与最小格式修复完成；本地回归和双二进制 smoke 已验证，保留基线 fmt 与外部独立复核缺口 | 代码 tip 105d527433d5211b8acdc0793cc1a3b223a5884f；Rust 2443 passed/0 failed/4 ignored；web 28 passed；cache run-20261024-221447 四步 13600 tokens、exit 0、stderr 0、9/8/9/8；reports/final-cargo-fmt.json 18 个基线文件 |
