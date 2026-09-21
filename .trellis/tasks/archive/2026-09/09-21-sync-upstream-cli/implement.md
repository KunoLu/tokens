# Implement: upstream CLI sync

## Ordered checklist

1. 分支 `sync/upstream-20260921`（已完成）。
2. `git cherry-pick -n 53082d75`：留下 `cli/` 与 `.github/assets/client-*`；**不要** restore web；**不要** 把 logo 拷进 `web/public/clients/`。version 已是 `1.0.0`。自提交 `15ed69df`。
3. 其余 8 个 CLI SHA 已 `cherry-pick -x` 到 `70b8245b`。catalog 3-way 时同样不 restore web。
4. 政策已按实测改：混合 core-sync、`.github/assets` 是 logo 源、helpers 不是 TUI、`test:migrations` 要 `DATABASE_URL`。
5. 验证见下。跳过清单供 PR 复用。

## Skip list (`main..upstream/main`)

| SHA | Subject | Reason |
|---|---|---|
| `0def29d6` | feat(deploy): Caddy stack | 部署拓扑，§3 丢弃 |
| `77675a41` | fix(deploy): wrangler types | Workers，丢弃 |
| `6e4283b8` | fix(deploy): pg 18 volume | 上游 PG 拓扑，丢弃 |
| `2c6cbc95` | perf(web): public profiles shared cache | edge cache，丢弃 |
| `c7418313` | perf(web): share cache no-reader pages | edge cache，丢弃 |
| `bdfc2736` | docs(web): sponsors section | 品牌/营销，丢弃 |
| `36d17e16` | perf(web): ship whole board | edge cache，丢弃 |
| `53fd5b66` | docs(web): V.PS sponsors | 品牌，丢弃 |
| `00c77318` | perf(web): shorten edge window | edge cache，丢弃 |
| `7f30fd6d` | fix(web): db blip caches 404 | 上游缓存语义，丢弃 |
| `b3a0a604` | fix(web): stop streaming board | 上游缓存/流式，丢弃 |
| `c3d9d471` | chore(deploy): Caddyfile | 部署，丢弃 |
| `ae37b798` | feat(deploy): daily badge refresh | verified badge，丢弃 |
| `4c989996` | fix(web): GitHub rate limit badges | verified badge，丢弃 |
| `15e3a664` | docs(web): who processes data | 上游隐私文案，丢弃 |
| `6201946a` | fix(web): profile same day twice | web 数据能力需重写，本轮不做 |
| `bfc87bb9` | feat(api): expose verified on profile | verified badge，丢弃 |
| `8415a719` | fix(ci): tag release at build commit | 上游发布/CI，§6 |
| `86ca4c85` | chore(release): 27.0.2 | 版本线，永不合并 |
| `09b18007` | docs: iOS App Store links | 品牌/文档，丢弃 |
| `76279f54` | chore(release): 27.0.3 | 版本线 |
| `6f724b7c` | chore(release): 27.0.4 | 版本线 |
| `8cf8f8e6` | ci(publish): OIDC npm | 上游发布，§6 |
| `6fb6af57` | chore(release): 27.0.5 | 版本线 |
| `9ecae999` | ci: bump actions Node 24 | 上游 CI，§6 |
| `ce0817e0` | chore(release): 27.1.0 | 版本线 |
| `7e8830f6` | chore(release): 27.1.1 | 版本线 |
| `b8da52d9` | Merge main into deploy/self-hosted | 多父 merge；部署分支，§3 丢弃 |
| `b0b81149` | Merge origin/main into deploy/self-hosted | 多父 merge；部署分支，§3 丢弃 |
| `4bcbe47c` | Merge origin/main into deploy/self-hosted | 多父 merge；部署分支，§3 丢弃 |
| `cc67eb75` | Merge origin/main into deploy/self-hosted | 多父 merge；部署分支，§3 丢弃 |
| `69c1ce9c` | Merge PR #70 sync/upstream-4.17.0 | 多父 merge；单父 `53082d75` / `9a6b630c` 已拣 |
| `dc134388` | Merge PR #71 | 多父 merge；单父 `3db04682` 已拣 |
| `3ee911c1` | Merge PR #72 | 多父 merge；单父 `b4f64595` 已拣 |

## Validation

```bash
cargo test --manifest-path cli/Cargo.toml --workspace   # 2417 passed, 4 ignored
bun run lint                                           # 0 error, 既有 docs <img> warning
bun run typecheck                                      # pass
bun run test:migrations                                # environment-blocked: DATABASE_URL missing
```

`rtk`: cargo/bun 用原生命令（`skipped-for-report`）。`test:migrations` **未通过**，不是同步回归，是本机无 Postgres URL。

## Rollback points

- 每个 cherry-pick 提交可独立 revert。
- 不要用 `git checkout 53082d75 -- cli/tokens-core` 作为默认退路（tokens-cli 会编不过）。
