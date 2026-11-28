---
schema_version: 1
id: 20261128-v107-amp-sync
workflow_mode: strict
mode_source: user
mode_note: 按用户明确通知与 PRD §5 执行任务 4，单 PR；不部署。
status: "in-progress"
branch: sync/v1.0.7-amp-sync
created_at: '2026-11-28T14:07:05.599298+08:00'
updated_at: "2026-11-28T14:07:11.528508+08:00"
completed_at: null
"sync_scope": ["cli/features"]
---
# 任务 4：Amp 线程同步

基线 8ecf75aea41f6c86e9f0b690251b84b5ea709987。纳入 c3fa4840 的 Amp sync/status、条件刷新、去重、三语与双语文档，不纳入任务 3 以外的解析改动、不改协议、版本或生产数据。未完整调用 grill-with-docs：PRD §5.4 已明确授权、缓存与验收范围。

## Book Gate Plan

- DDD not-required：命令、thread、usage 与 opt-in 的产品含义已明确，无业务术语歧义。
- DDIA passed（confirmed）：Amp 子进程拥有原始线程/凭证；tokens 只投影计量字段到独立私有缓存。按线程 ID 与上次更新时间增量，先原子替换线程，再保存成功项水位；失败保留已缓存文件、未成功项后续可重试。状态文件是显式 opt-in 边界，不加数据库迁移。以合成导出核对内容、失败回退与重试。
- Legacy passed（characterized）：既有 sessions/amp.rs 对旧版数据目录、消息级日期及 ledger 优先级有测试；新缓存仅作为同一线程的另一个来源。建立 BDD 和隔离子进程用例；不触碰真实用户目录、账号、生产服务或凭据。
- Refactoring passed（proceed，normal）：沿用现有提交管线、客户端筛选、路径与原子写 helper。上游未经类型约束的嵌套 JSON 和原始 stderr 不符合隐私边界，需本仓适配。
- Release planned：本机 clippy、workspace test、release build、CLI 行为 smoke 后作正式裁决。双审及同 SHA CI 前不合入。

## 验收

默认不联网；显式 sync 使用本地 amp，status 只读；敏感内容在落盘前丢弃，失败告警且不披露原始输出；重复来源不双计。双语文档包含主代理统计及近 100 条搜索限制。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-28T14:07:11.528508+08:00 | planned | in-progress | omp：开始 Amp 规格、隐私边界与同步实现 | {"changed_files": [], "evidence_files": [], "sync_files": {"cli/features/antigravity-sync.feature": {"mtime_ns": 1793157456643708146, "sha256": "507366c03e2020a0a0c4c6c6cb2cd6779d53038b53b801342109be9e42cf5339", "size": 1660}, "cli/features/inference-speed.feature": {"mtime_ns": 1793157456643769521, "sha256": "904dcd0fbc922e31b06b8dcb66eb9e155a227acaf72a403e54b7fc6d922aa606", "size": 2131}, "cli/features/local-client-usage.feature": {"mtime_ns": 1795845704785246425, "sha256": "1b2215d29ef82a83aff4352f1f7ca365d0989baf6d91912246810512f82a5d4c", "size": 4061}, "cli/features/paseo-discovery.feature": {"mtime_ns": 1793157456643833439, "sha256": "e598869b0ec21eac6ac9b9f797f1d6bd3db777d605488e4f3d7e0d865dd31c5a", "size": 1403}}, "validation": ""} |
