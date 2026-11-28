# v1.0.7 发布记录

[标注标签](https://github.com/KunoLu/tokens/tree/v1.0.7)与 [tokens.camp](https://tokens.camp/docs) 部署指向同一提交 `6dcaf435f2f4a07926676c0dd3a05b4d84abc135`。以下时间均为 UTC+8。

## 合入与 CI

每条 PR CI 对应该 PR 最终 HEAD；每条 main CI 对应同一行的 squash 提交。所列运行结论均为 `success`。历史失败的运行不据此改记成功。

| PR | PR HEAD / CI | main 提交 / CI |
|---|---|---|
| [#30 提交校验](https://github.com/KunoLu/tokens/pull/30) | `b731814c` / [45989165950](https://github.com/KunoLu/tokens/actions/runs/45989165950) | `54c06394` / [45990081075](https://github.com/KunoLu/tokens/actions/runs/45990081075) |
| [#31 网站与文档](https://github.com/KunoLu/tokens/pull/31) | `e38634d0` / [46008399719](https://github.com/KunoLu/tokens/actions/runs/46008399719) | `8f44dcee` / [46010642283](https://github.com/KunoLu/tokens/actions/runs/46010642283) |
| [#32 本地 CLI 解析](https://github.com/KunoLu/tokens/pull/32) | `7ce5fc51` / [46114247149](https://github.com/KunoLu/tokens/actions/runs/46114247149) | `8ecf75ae` / [46114710887](https://github.com/KunoLu/tokens/actions/runs/46114710887) |
| [#33 Amp 同步](https://github.com/KunoLu/tokens/pull/33) | `7c8bc8f8` / [46147849279](https://github.com/KunoLu/tokens/actions/runs/46147849279) | `69ea41e9` / [46148597103](https://github.com/KunoLu/tokens/actions/runs/46148597103) |
| [#34 发布准备](https://github.com/KunoLu/tokens/pull/34) | `a0d8e526` / [46152317867](https://github.com/KunoLu/tokens/actions/runs/46152317867) | `6dcaf435` / [46153289808](https://github.com/KunoLu/tokens/actions/runs/46153289808) |

## 标签与两次生产部署

- 2026-11-27 20:57:17，提前部署 `dpl_6oy6DGZs1NbjDHaeubeViYf8kfsc` 达到 READY，Git 提交 `54c06394a8416b5ac4d7d9ba847f78633048d4c8`。任务 2 已记录别名、文档及图标抽查；最终部署前再次通过 Vercel 核实该部署仍服务 `tokens.camp`。它是本次切换的回退候选，未执行回退。
- 2026-11-28 22:20:50，在发布提交的 Web、Linux、macOS、Windows main CI 全绿后，推送双语 annotated tag `v1.0.7`。公开 tag 对象 `61a30711af9a7124d2c8c844d8612435632e455e`，解引用为上述 `6dcaf435…`。GitHub Release 按 tag 查询返回 404；tag 下两种预安装脚本均与该提交的 Git blob 一致。
- 用户单独确认最终生产部署后，2026-11-28 23:12:37 向 Vercel 项目 `tokens`（仓库 `KunoLu/tokens`，根目录 `web`）提交指定 Git SHA 的 Production 请求。23:13:41 部署 `dpl_3SC6ffCDCVPKXJYGq4Ecu6niezVS` 达到 READY；23:13:44 查得 `tokens.camp` 指向该 ID。平台的 `gitSource.sha` 和提交元数据均为 `6dcaf435f2f4a07926676c0dd3a05b4d84abc135`，不是回填分支。沿用原生产构建命令；本版未新增数据库迁移。未发布 npm 包、未创建 GitHub Release。

## 验收边界

- **生产站实测：** 匿名 Chrome 打开 `https://tokens.camp/docs` 返回 200，Qoder、kwwk、TRAE CLI 可见；页头标记的计算填色为 `rgb(124, 58, 237)`（`#7C3AED`）。Shell 与 Windows 安装页签均引用 `v1.0.7`。从英文文档选择中文、日本语，主 frame 分别请求 `/zh-CN/docs`、`/ja/docs`，均返回 200，`html lang` 正确，域名不变。
- **同一发布 SHA 的行为回归：** 以合成数据执行 `ProfilePageClient.test.tsx`、`timezone.test.ts`、`validation/submission.test.ts`，35 项通过。覆盖账号时区优先、留空按浏览器时区判定“今天”、三个新客户端，以及 Hermes/Amp 修订 2 的校验与旧修订重放。这里不是登录生产账号的实测，也不是一次生产 `/api/submit` 成功写入；没有读取真实用户资料、写入生产用量或回填历史。
- 此回填不修改运行时、工作流、生产配置、发布 tag 或安装来源。`/install.sh` 仍从上游 `missuo/tokens` 获取二进制。

## 回填 PR 的 CI

[#35](https://github.com/KunoLu/tokens/pull/35) 的初始提交 `44e14962` 只改 `ai/tasks/`，未命中原 CI 路径过滤，没有该 SHA 的 CI 结果。独立 [#36](https://github.com/KunoLu/tokens/pull/36) 在 `pull_request.paths` 增加 `ai/tasks/**`，没有改动检查、权限或 `push main` 规则。配置提交 `8e2535a1` 经双审 CLEAN 和[四项 CI](https://github.com/KunoLu/tokens/actions/runs/46160157675)通过后合入 `main`（`323422d3`）。#35 同步该提交后按自己的 HEAD 验收；发布 tag 和 Production 指向不变。