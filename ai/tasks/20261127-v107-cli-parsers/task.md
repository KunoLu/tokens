---
schema_version: 1
id: 20261127-v107-cli-parsers
workflow_mode: strict
mode_source: user
mode_note: PRD §5 任务 3；单 PR，不部署。
status: "in-progress"
branch: sync/v1.0.7-cli-parsers
created_at: '2026-11-27T21:51:46.026310+08:00'
updated_at: "2026-11-27T21:51:49.133619+08:00"
completed_at: null
"sync_scope": ["cli/features"]
---
# 任务 3：CLI 本地扫描与解析

基线 8f44dcee4c557cd5a84ea4d8e237cfa84c355046。按 v27.3.0、v27.4.0、v27.4.1 顺序逐个处理单父提交；混合提交剥离 Web/README/发布工作流，保留 docs/qoder.md。保留本仓 1.0.x、安装来源与既有认证，不执行真实客户端登录或私人目录扫描。Amp 云同步留下一票。未完整调用 grill-with-docs：范围与协议已给定。

## Book Gate Plan

- DDD confirmed：本地 parser_version、远端协议修订、文件缓存格式为三种独立版本。TRAE CLI 不归并桌面 Trae；Qoder 两个版本归同一客户端。
- DDIA confirmed：原始客户端日志/库是来源，解析缓存是可重建副本。用客户端 parser_version 更新局部缓存，不升级全局格式 9；不运行数据迁移。修正日分桶由任务 1 网站接收。保留旧数据和私密信息，只以合成夹具验证。
- Legacy characterized：既有 Rust 用例、对照上游 tag 的回归夹具，保留本仓特有认证/翻译/版本。已识别 d693f5d5 的直接返回修复与本仓现有代码一致，不为 cherry-pick 制造空内容。
- Refactoring proceed：沿用客户端宏、scanner 和 sessions 模块；只移植必要 CLI 入口与三语文案，不重写已分叉文件。
- Release planned：本票不发 CLI；验证调度、缓存、凭证边界与各平台 CI 后给出技术裁决。

## 验收

原生 cargo check/clippy/test/build --release、tokens --version、Web 原生验证；迁移只对可证明是本地测试库的显式配置运行。完整来源清单与跳过理由随 PR 交付。双审与同 SHA CI 均通过后收尾。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-27T21:51:49.133619+08:00 | planned | in-progress | omp：开始 CLI 规格与分段拣入 | {"changed_files": [], "evidence_files": [], "sync_files": {"cli/features/antigravity-sync.feature": {"mtime_ns": 1793157456643708146, "sha256": "507366c03e2020a0a0c4c6c6cb2cd6779d53038b53b801342109be9e42cf5339", "size": 1660}, "cli/features/inference-speed.feature": {"mtime_ns": 1793157456643769521, "sha256": "904dcd0fbc922e31b06b8dcb66eb9e155a227acaf72a403e54b7fc6d922aa606", "size": 2131}, "cli/features/paseo-discovery.feature": {"mtime_ns": 1793157456643833439, "sha256": "e598869b0ec21eac6ac9b9f797f1d6bd3db777d605488e4f3d7e0d865dd31c5a", "size": 1403}}, "validation": ""} |
