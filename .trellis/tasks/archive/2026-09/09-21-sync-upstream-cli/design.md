# Design: upstream CLI sync

## Book Gate Plan

| Skill | Trigger | Phase | Gate state |
|---|---|---|---|
| book-refactoring-pass | 修改既有生产 CLI | 首次编辑前 | completed（cherry-pick，无额外重构） |
| book-legacy-change-safety | 非既有行为 bug；CLI 有 Rust 测试 | — | not-required |
| book-ddd-distilled-modeling | 无领域边界变更 | — | not-required |
| book-ddia-data-design | 不改持久化 schema | — | not-required |
| book-release-readiness | 本轮不合 main、不发布 | — | not-required |

## Refactoring Review (pre-edit)

```text
Refactoring Review
Status: proceed
Review mode: normal
Existing-code scope: cli/tokens-core, cli/tokens-cli, web client catalog
Behavior that must remain unchanged: 本仓 web 自托管/teams/邮箱认证；版本线 1.0.0；运行时 logo URL `/clients`，提交源 `.github/assets/client-*`
Structural friction: 53082d75 把解析器、CLI 集成、web catalog、GitHub assets 捆在一起
Decision and smallest safe step: 不重构本仓代码。混合提交用 cherry-pick -n；3-way 保住 `/clients` 则不 restore web；留下 assets
Safety net and validation: cargo test --manifest-path cli/Cargo.toml --workspace；bun run lint/typecheck。test:migrations 要 DATABASE_URL，缺则 environment-blocked
Deferred refactors: none
```

## Ponytail

最短路径：`git cherry-pick -n` / `-x`，不要重写解析器。web catalog 只加 id，不改 i18n 基础设施。

## Boundaries

| 取 | 不取 |
|---|---|
| `cli/tokens-core/**` 解析器、session、pricing、测试 | Worker / wrangler / edge cache / badge |
| 为新客户端编译所需的 `cli/tokens-cli/**`（含已有的 `usage/helpers.rs`） | `27.x` 版本号、`chore(release)`、CI publish |
| `.github/assets/client-*`（`web` build 复制到 gitignored `public/clients/`） | 把 logo 只写入 `web/public/clients/` |
| 3-way 把新 client id 合进本仓 `types.ts` / `constants.ts`（路径保持 `/clients`） | 上游 GitHub CDN URL 当 logo 源 |

`tui_signal.rs` / `wiki.rs`：若 `tokens-core` lib 声明了这些模块，为了与 missuo core 对齐并编译，**留下**；它们不是 TUI 命令入口。本仓没有 `tui_signal` 引用不等于要剥掉模块——剥掉会再改 lib.rs，扩大 diff。

## 53082d75 procedure（实测）

1. `git cherry-pick -n 53082d75`
2. 3-way 已把 `constants.ts` / `types.ts` 合进 `/clients` 路径，**不要** `git restore web`。
3. **留下** `.github/assets/client-*`。
4. 确认 `cli/Cargo.toml` version 仍为 `1.0.0`（本轮 auto-merge 已保住）。
5. 自己 commit，写 `(cherry picked from 53082d75)`。

## Later SHAs (oldest first, `-x`)

| SHA | Subject | Decision |
|---|---|---|
| `2b5aafa2` | fix(grok): take each session span from one log only | pick |
| `ab4b2793` | fix(commandcode): count cached prompt tokens once in v3 usage | pick |
| `5df7d1f7` | feat(workbuddy): honour WORKBUDDY_CONFIG_DIR | pick |
| `0532b9d5` | feat(clients): add Craft Agent session parsing | pick |
| `ae163d78` | fix(cli): keep the usage helpers macOS-only | pick（helpers 在本仓存在） |
| `9a6b630c` | fix(antigravity): gate the capped RPC reader off Windows | pick |
| `3db04682` | fix(codex): bump the submit parser revision to 3 | pick |
| `b4f64595` | fix(codex): stop counting exec captures twice and re-dating them | pick |

`0532b9d5` 含 web catalog 时同样 **不要 restore web**；本轮 3-way 已加上 `craft-agent`。

## Compatibility

- 不改迁移 journal。
- 不改认证/teams。
- Submit 校验：新 id 必须进入 `SUPPORTED_CLIENT_TYPES`，否则新 CLI 会被拒。

## Rollback

`git reset --hard e90e3ddd` 回到拣入前（即当时的 `main`）。HEAD 现为 `70b8245b`。每个 SHA 单独提交，可逐个 `revert`。未提交的 `docs/upstream_policy.md` 与任务目录需另处理。
