# Cherry-pick upstream CLI parser fixes

## Goal

把 `missuo/tokens`（`upstream/main`）上尚未合入的 **CLI / tokens-core 解析器与新客户端** 取到本仓 `sync/upstream-20260921`，使本仓 CLI 不再落后于上游 27.1.1 的 core，同时 **不** 把 Workers / edge cache / verified badge / 27.x 发布线带进来。

## Background

- 共同祖先仍是 `f114057f`。GitHub 的 ahead/behind 是长期 fork 常态，禁止 merge / rebase / Sync fork。
- 规则源：`docs/upstream_policy.md`。CLI 命令集不是产品分叉；两点 diff 的 CLI −107k 是 **上游前进**，三点语法下本仓有意保留的几乎只有版本 `1.0.0`。
- `53082d75`（`feat(core): sync tokens-core with upstream tokscale 4.17.0`）是混合提交：137 files / +106k。用 `cherry-pick -n` 再自提交，不要 `cherry-pick -x` 整棵；也不要只取 `tokens-core`。
- 本仓 `usage/helpers.rs` 已存在，职责是 macOS keychain / 密钥原子写，不是 TUI 报表。因此 `ae163d78` 要拣。
- 运行时 URL 是 `GITHUB_CDN_BASE = "/clients"`。**提交的 logo 源是 `.github/assets/client-*`**：`web/package.json` `build` 把它们拷进 gitignored 的 `public/clients/`。不要把 logo 只写入 `web/public/clients/`，也不要 unstage `.github/assets`。

## Confirmed facts

- `main..upstream/main` 共 43 个提交：36 个单父 + 7 个多父 merge。拣了 9 个单父 CLI SHA；merge 不 `-m` 拣（PR merge 内容已由对应单父进入；deploy 分支 merge 丢弃）。
- 新客户端 id：senpi, augment, kimchi, prime-agent, cherrystudio, dsh, mcode, omp, lmstudio, unsloth, hindsight, craft-agent。`SUPPORTED_CLIENTS` 54 + docs 上的 Orca = 55；`SUPPORTED_CLIENT_TYPES` 56（另含 filter-only `synthetic` / `9router`）。
- 提交校验走 `SUPPORTED_CLIENT_TYPES`；CLI 能扫到的 id 若不进 web types，会被整单拒绝。

## Requirements

1. 按 §4.1 **一次一个 SHA** 处理；混合 core-sync 用 `-n` 后自提交并写 `(cherry picked from <sha>)`；干净 SHA 用 `-x`。
2. `53082d75`：取 `cli/`（core + 为编译所需的 tokens-cli，含 helpers / `tui_signal` / `wiki` 模块）。**不要** `git restore web`：3-way 若保住 `/clients` 路径则保留 catalog。**留下** `.github/assets/client-*`。version 保持 `1.0.0`。
3. 后续 CLI 修复按时间顺序 `cherry-pick -x`。catalog 3-way 同样不要 restore web。
4. 任何版本文件冲突或上游写入 `27.x` 时保留 **`1.0.0`**。
5. 跳过的 SHA 及理由写入本任务 `implement.md`（同步 PR 描述复用），不写进 `docs/upstream_policy.md`。

## Acceptance Criteria

- [x] `cli/Cargo.toml` / `packages/cli/package.json` 的 version 仍为 `1.0.0`。
- [x] 未 merge/rebase `upstream/main`；`53082d75` 以 `15ed69df` 进入（3-way catalog、version `1.0.0`、保留 `.github/assets`）；无 Worker / wrangler / edge cache / badge。
- [x] 新 client id 在 `SUPPORTED_CLIENT_TYPES` 与 display/logo/color 表；logo 文件在 `.github/assets/client-*`（不提交 `public/clients/`）。
- [x] 后续 8 个 CLI SHA 已 `-x` 进入历史；其余 SHA 在 implement 跳过表。
- [x] `cargo test --manifest-path cli/Cargo.toml --workspace` 通过（2417 passed）。
- [x] `bun run lint`（既有 `<img>` warning）与 `bun run typecheck` 通过。
- [ ] `bun run test:migrations`：**environment-blocked**（无 `DATABASE_URL`）。本轮未改 journal。

## Out of scope

- 上游 web 排行榜缓存、profile 重复日、verified badge、Caddy/PG deploy、OIDC publish、`chore(release)`、iOS/sponsor 文案。
- 把 CLI 做成 tokscale TUI / 报表命令。
- 本轮不向 `origin/main` merge，不开 GitHub PR（除非用户另说）。

## BDD

跳过。本轮是上游 CLI 解析器同步；可观察行为由 Rust 测试覆盖（政策 §4.2）。不新增本仓产品场景。

## grill-with-docs

未完整调用。Git 供应商同步，不涉及领域模型或长期术语。
