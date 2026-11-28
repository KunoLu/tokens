---
schema_version: 1
id: 20261128-v107-amp-sync
workflow_mode: strict
mode_source: user
mode_note: 按用户明确通知与 PRD §5 执行任务 4，单 PR；不部署。
status: "checking"
branch: sync/v1.0.7-amp-sync
created_at: '2026-11-28T14:07:05.599298+08:00'
updated_at: "2026-11-28T14:23:15.672746+08:00"
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
- Release Readiness Review — passed（ready，仅本票 CLI 技术就绪）：已验证 opt-in、失败不阻塞提交、状态与线程缓存的原子写、相同 ID 去重。导出至多四路并行，发现/导出单次超时分别为 120/300 秒；首次同步时间随账户线程数增长。状态与脱敏错误可由 status/sync 观察。无需数据库迁移；回退 CLI 不删除用户缓存。CLI 双审与同 SHA CI 是合入前置条件，本票不发布、不部署。

## 验收

默认不联网；显式 sync 使用本地 amp，status 只读；敏感内容在落盘前丢弃，失败告警且不披露原始输出；重复来源不双计。双语文档包含主代理统计及近 100 条搜索限制。

本地原生 clippy、workspace test、release build 通过。隔离合成 Amp 的 5 个 CLI 集成用例全部通过，包含真实 `submit --dry-run --json --client amp`，不触发真实账号操作。检查了 `status --json` 与中日帮助；未提前升级版本，二进制仍为 1.0.6。Rust 4 个 ignored 不计入通过。BDD 对应 `cli/features/amp-sync.feature`，真实外部账户未验，不将子进程夹具声称为云端 full-stack。留存 `reports/amp-20261128-142035.{log,md}`。

来源为上游 `c3fa4840c0ec1ca7585c67c42343e3102746dee1`。保留本仓双语 README、域名和既有云会话说明；额外收紧嵌套计量字段、stderr、锁、错误状态及覆盖旧缓存边界。不纳入任务 3 已实现的 Amp 日期/模型解析修复，不扩展其他云同步。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-11-28T14:07:11.528508+08:00 | planned | in-progress | omp：开始 Amp 规格、隐私边界与同步实现 | {"changed_files": [], "evidence_files": [], "sync_files": {"cli/features/antigravity-sync.feature": {"mtime_ns": 1793157456643708146, "sha256": "507366c03e2020a0a0c4c6c6cb2cd6779d53038b53b801342109be9e42cf5339", "size": 1660}, "cli/features/inference-speed.feature": {"mtime_ns": 1793157456643769521, "sha256": "904dcd0fbc922e31b06b8dcb66eb9e155a227acaf72a403e54b7fc6d922aa606", "size": 2131}, "cli/features/local-client-usage.feature": {"mtime_ns": 1795845704785246425, "sha256": "1b2215d29ef82a83aff4352f1f7ca365d0989baf6d91912246810512f82a5d4c", "size": 4061}, "cli/features/paseo-discovery.feature": {"mtime_ns": 1793157456643833439, "sha256": "e598869b0ec21eac6ac9b9f797f1d6bd3db777d605488e4f3d7e0d865dd31c5a", "size": 1403}}, "validation": ""} |
| 2026-11-28T14:23:12.091456+08:00 | in-progress | in-progress | omp：完成 Amp 规格、隐私边界与同步实现 | {"changed_files": ["README.md", "README_zh.md", "cli/tokens-cli/locales/en/cli.json", "cli/tokens-cli/locales/en/top.json", "cli/tokens-cli/locales/ja/cli.json", "cli/tokens-cli/locales/ja/top.json", "cli/tokens-cli/locales/zh-CN/cli.json", "cli/tokens-cli/locales/zh-CN/top.json", "cli/tokens-cli/src/amp.rs", "cli/tokens-cli/src/main.rs", "cli/tokens-cli/tests/amp_sync.rs", "cli/tokens-core/src/scanner.rs", "cli/tokens-core/src/sessions/amp.rs", "docs/cloud-sessions.md"], "evidence_files": ["ai/tasks/20261128-v107-amp-sync/reports/amp-20261128-142035.md", "ai/tasks/20261128-v107-amp-sync/reports/amp-20261128-142035.log"], "sync_files": {"README.md": {"mtime_ns": 1795846186730577178, "sha256": "b380e576a4c4ace237def1b7322c240bf784764929fbb737133c410a2e0b41bf", "size": 14206}, "README_zh.md": {"mtime_ns": 1795846186731506182, "sha256": "b4ce4d2827f67776d9a6375e9fae4a75f142cd8130bb67c955c0eded990bf8d7", "size": 13664}, "ai/tasks/20261128-v107-amp-sync/reports/amp-20261128-142035.log": {"mtime_ns": 1795846921474311703, "sha256": "2a065f9556d1afe99ddf387a3b2be0beb3e9bbe8c12125746bead4b37f976422", "size": 240436}, "ai/tasks/20261128-v107-amp-sync/reports/amp-20261128-142035.md": {"mtime_ns": 1795846948003192486, "sha256": "98c061e071bdace86324e36e1846a818315f8956d5fac84a942a9391021779d8", "size": 1155}, "cli/features/amp-sync.feature": {"mtime_ns": 1795846043733342051, "sha256": "7b2e7bcb51ae4e36f114fed34e9f0900c8fadbfc13b269c05be5d03824e7fdd7", "size": 1263}, "cli/features/antigravity-sync.feature": {"mtime_ns": 1793157456643708146, "sha256": "507366c03e2020a0a0c4c6c6cb2cd6779d53038b53b801342109be9e42cf5339", "size": 1660}, "cli/features/inference-speed.feature": {"mtime_ns": 1793157456643769521, "sha256": "904dcd0fbc922e31b06b8dcb66eb9e155a227acaf72a403e54b7fc6d922aa606", "size": 2131}, "cli/features/local-client-usage.feature": {"mtime_ns": 1795845704785246425, "sha256": "1b2215d29ef82a83aff4352f1f7ca365d0989baf6d91912246810512f82a5d4c", "size": 4061}, "cli/features/paseo-discovery.feature": {"mtime_ns": 1793157456643833439, "sha256": "e598869b0ec21eac6ac9b9f797f1d6bd3db777d605488e4f3d7e0d865dd31c5a", "size": 1403}, "cli/tokens-cli/locales/en/cli.json": {"mtime_ns": 1795846052271583959, "sha256": "f0d36c1540f7d624eac2f63295d503c78ec1fa14cb455bd4ccd76ed847b3ed8b", "size": 25871}, "cli/tokens-cli/locales/en/top.json": {"mtime_ns": 1795846406027252727, "sha256": "cb073980709350d97d6362c820fc53e37b91239155452627e0437b434038138b", "size": 28017}, "cli/tokens-cli/locales/ja/cli.json": {"mtime_ns": 1795846052271976045, "sha256": "3e0557323cb241e38794fa8cbef26988917c0648efb8e72dcb5468b246776464", "size": 32988}, "cli/tokens-cli/locales/ja/top.json": {"mtime_ns": 1795846406030055782, "sha256": "cb49cb797460469c3a3778cf8a0566c688214e6ee02d8be0187da095a19c7087", "size": 36142}, "cli/tokens-cli/locales/zh-CN/cli.json": {"mtime_ns": 1795846052272688048, "sha256": "8960aca5e071dfeb1a700e1f16c0ab929ebb9ee9b430a3d8a8a197804f04ebf4", "size": 25610}, "cli/tokens-cli/locales/zh-CN/top.json": {"mtime_ns": 1795846406031924166, "sha256": "ea4fb0730d4389d9593f13abef3884f42038592a7bcb4fbeabe590243c09af38", "size": 27334}, "cli/tokens-cli/src/amp.rs": {"mtime_ns": 1795846806857264403, "sha256": "feebcb41b1f80a95f82dd5b810106b03c67f06d6da50870bd35cdb532d7f732d", "size": 29493}, "cli/tokens-cli/src/main.rs": {"mtime_ns": 1795846052274024596, "sha256": "319759bcec09e538c95dd005426eeba9e905eec193e1acd6e2ac1d7e20ce14ac", "size": 158401}, "cli/tokens-cli/tests/amp_sync.rs": {"mtime_ns": 1795846705547477347, "sha256": "f17c9638bc090deb6ecf5970b3cb21af1f7dff963277c95d16f67d0e1c3362ac", "size": 15027}, "cli/tokens-core/src/scanner.rs": {"mtime_ns": 1795846279896152350, "sha256": "d971ccb13a27e170b5ba03e0fa3458dc7e86a66a5353bed4805df2b1da1e87f4", "size": 389167}, "cli/tokens-core/src/sessions/amp.rs": {"mtime_ns": 1795846641173851878, "sha256": "04d3d9ad0e992ebc5002df3c63364dd57074361f6a9b273dc3515ccf826e376d", "size": 26081}, "docs/cloud-sessions.md": {"mtime_ns": 1795846186731996851, "sha256": "84de32598d783c32dd20f21178f654f4512b260394e4cf33b65464fecd79d6e9", "size": 9700}}, "validation": "5/5 CLI integration、clippy、workspace test、release build 及 CLI 冒烟通过。无真实账号；三语与双语文档同步。"} |
| 2026-11-28T14:23:15.672746+08:00 | in-progress | checking | omp：本地验收通过，送双审及同 SHA CI | reports/amp-20261128-142035.md |
