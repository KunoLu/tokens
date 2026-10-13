# 更新日志

本文件只记录本仓库发布的 tag。版本线是 `1.0.x`。`v27.x` 属于上游 `missuo/tokens`，不是本 fork 的发布版本。

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
