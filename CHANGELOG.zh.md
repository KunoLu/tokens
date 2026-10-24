# 更新日志

[English](./CHANGELOG.md)

本文件只记录本仓库发布的 tag。版本线是 `1.0.x`。`v27.x` 属于上游 `missuo/tokens`，不是本 fork 的发布版本。

## [Unreleased]

## [1.0.4] - 2026-10-25

### 新增

- 模型级推理速度用量：区分 Claude 服务端报告速度与 Codex 选择速度，贯穿 CLI 缓存、提交和 profile 数据，不改变 token 总量或计价（#14；上游 v27.1.5）。
- Profile 的 token 构成占比，以及当天用量、日详情和周期模型表中的推理速度档位；缺失速度仍保持未知（#15）。
- 通过原生提供商数据目录发现 Paseo 管理的 Claude Code 与 Codex CLI 会话，复用既有解析器和文件去重（#15）。
- 本仓 CLI 源码新增面向 Claude Code on the web 会话的 `tokens cloud submit` 和 `tokens cloud status`，采用哈希会话设备，并在 profile／设置页按提供商合并展示（#16）。通过 `/install.sh` 安装的二进制来自上游 Release，上游自 v27.1.4 起同样包含 `tokens cloud`。不支持 Codex cloud。服务端校验 `cloud_` key 与提供商／客户端对应关系，拒绝 backfill 和 replacement，强制设备名为 null、禁止改名、不接管 legacy 历史，也不覆盖账号的 MCP 列表。

### 修复

- 同步上游 v27.1.4 的 Antigravity CLI 修复：存在活跃 language server 时在 submit／autosubmit／serve 提交前同步，隐式同步 RPC 共用 60 秒预算，部分刷新失败时报告结果并保留缓存会话（#13）。显式同步在部分会话刷新失败时成功退出，发现失败或锁错误仍失败；同时包含 Windows 测试 helper 的 clippy 修复。
- Profile 使用官方 OMP 图标（#15）。
- 设备名掩码描述明确：名称不超过四个字符时只显示首字符，更长的名称保留首尾各两个字符。
- Cloud 配置文档改用本站提供的 `/install.sh`；该脚本默认安装上游 `missuo/tokens` 的最新 GitHub Release 二进制（当前为 v27.1.5，`tokens --version` 显示 27.1.5）。Token 设置改为指向 `tokens.camp`，提交要求设置 `TOKENS_API_URL=https://tokens.camp`，文档列出实际 GitHub 下载域名。

### 变更

- 通过 `web/vercel.json` 的 `git.deploymentEnabled` 关闭 `main` 的 Vercel Git 自动部署（#11）。Production 改为手动触发部署。
- 本地缓存格式从 8 升至 9（#14）。**降级到旧 CLI 会从会话日志重建本地缓存。** 若历史仅存在于新版缓存、源日志已删除，则无法从日志恢复；降级前请同时备份缓存与源日志。
- 本仓清单版本（Cargo workspace 与 `packages/cli*`）和钉版脚本 ref 分别推进到 `1.0.4` 与 `v1.0.4`，对应本仓 `v1.0.4` tag。本仓只发布 tag，没有 GitHub Release；这些版本与 ref 改动不改变 `/install.sh` 安装的上游二进制版本。

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
