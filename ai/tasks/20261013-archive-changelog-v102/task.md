---
schema_version: 1
id: 20261013-archive-changelog-v102
workflow_mode: lite
mode_source: user
mode_note: 用户明确要求 lite。补充要求发布 v1.0.2，并把该版本写入两份 changelog。
status: "in-progress"
branch: docs/archive-and-changelog
created_at: '2026-10-13T14:21:28.651117+08:00'
updated_at: "2026-10-13T14:21:29.141928+08:00"
completed_at: null
---
# 归档 TODO、删除 demo、补 changelog 并发布 v1.0.2

## 目标

归档已完成的 `docs/TODO.md`，删除 `docs/demo`，按最新用法修正两份 README，写入中英 changelog，并按本仓发版方式打 `v1.0.2`。

## 范围

- `docs/TODO.md` 移到 `docs/archive/`，保留原文并加归档说明。
- 删除 `docs/demo/`。
- README 修正：CLI 不再写「与上游完全一致」；去掉会绕过本站地址的 `bunx` 推荐；客户端数量改为目录实数；补私密模式和 changelog 链接。
- changelog 只按本仓已发布 tag：`v1.0.0`、`v1.0.1`，以及本次 `v1.0.2`。不把上游 `v27.x` 写成自己的版本线。
- 跑 `scripts/set-version.sh 1.0.2`，提交后打 tag 并推送。
- 不同步本地 Docker。

## 验收

- 原 TODO 路径不在，归档文件在。
- `docs/demo` 不在。
- 两份 changelog 含 1.0.0、1.0.1、1.0.2。
- tag `v1.0.2` 指向发版提交，且 manifest 版本为 1.0.2。
- PR 已 admin merge，开发分支已删。

未完整调用 grill-with-docs：归档、删 demo、changelog 和发版步骤已由用户写明，没有未决产品问题。

## 状态事件

| at | from | to | reason | evidence |
|---|---|---|---|---|
| 2026-10-13T14:21:29.141928+08:00 | planned | in-progress | 用户确认归档、删 demo、changelog，并要求打 v1.0.2 | docs/archive-and-changelog |
