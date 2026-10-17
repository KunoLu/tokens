# 更新日志

[English](./CHANGELOG.md)

本文件只记录本仓库发布的 tag。版本线是 `1.0.x`。`v27.x` 属于上游 `missuo/tokens`，不是本 fork 的发布版本。

## [Unreleased]

### 变更

- 通过 `web/vercel.json` 的 `git.deploymentEnabled` 关闭 `main` 的 Vercel Git 自动部署（#11）。Production 改为手动触发部署。

## [1.0.3] - 2026-10-17

对应提交 `a752960`，即 tokens.camp 上运行的 Production 版本。

### 新增

- 首个在 tokens.camp（https://tokens.camp）上线的生产版本。
- Vercel Hobby 部署路径（#9）。只有同时设置 REST URL 和 token 时才启用 Upstash 限流；自托管和 Docker 继续用进程内 10/60s 限流。Vercel Cron 可用 GET 调用每日维护路由。新增 Vercel Hobby 部署手册。

### 修复

- Upstash 认证限流的 key 按 `VERCEL_ENV` 分命名空间（缺省为 `local`），Preview 与 Production 不再共用一个桶（#10）。

### 变更

- README 顶栏按语言分成两行。`Changelog`、`Deploy`、`CLI` 打开英文文档；`更新日志`、`部署`、`手册` 打开中文文档。英文部署清单和 CLI 手册是 `docs/deploy/self-host-production.en.md` 与 `docs/deploy/tokens-cli-usage.en.md`。（#8）
- 文档的自定义域名章节使用 Vercel 项目名 `tokens`，Vercel 免费地址更正为 `tokens-blush.vercel.app`。

## [1.0.2] - 2026-10-13

### 新增

- 私密模式。`tokens login --private` 让账号离开公开排名，并打印一次性读令牌。设置页可以改回公开；提交令牌不能。
- 文档页增加私密模式说明，并写明 CLI 登录前要先在网站登录，再到 `/device` 填授权码。
- 本 fork 的 tag 更新日志。

### 变更

- 把上游解析器修复拣进 `1.0.x`：Pi 用量记到实际模型，Claude 1 小时缓存写入按 1 小时单价计，可在 `settings.json` 里剥离模型 ID 前缀。
- 用量图不再输出体积过大的 SVG 路径。
- README 不再写 CLI 与上游完全一致，不再推荐用 `bunx` 登录或提交，客户端数量改为目录中的 54 个。
- 已完成的 Teamboard 清单移到 `docs/archive/`。HTML demo 已删除。
- 脚本下载地址和包版本推进到 `v1.0.2`。

## [1.0.1] - 2026-09-21

### 新增

- Team、Group 和 Teamboard 随这个 tag 进入已发布版本线。
- 增加 Craft Agent 扫描，并支持 `WORKBUDDY_CONFIG_DIR`。

### 修复

- 用 cherry-pick 把 CLI 核心对齐到上游 `27.1.1`，不采用 `27.x` 版本号。
- Codex 不再把 exec 捕获计两次；提交解析器修订号为 3。
- Grok 会话区间只取一份日志。Command Code 的缓存 prompt token 只计一次。Antigravity 的限长读取不在 Windows 上启用。用量 helper 仍只在 macOS。
- 脚本下载地址推进到 `v1.0.1`。

## [1.0.0] - 2026-09-18

### 新增

- 本自托管 fork 的第一个 tag：邮箱密码登录，树里已有 Team 数据，中英文界面，紫色品牌标。
- 生产 Docker 资产、本地 Compose 参考副本和部署清单。
- 预安装脚本和常驻提交脚本，把 `TOKENS_API_URL` 固定到操作者自己的站。
- 文档页安装命令指向本站，登录和提交不使用 `brew services` 或 `bunx`。
