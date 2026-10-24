Feature: Antigravity 会话同步
  提交前刷新本地 Antigravity 用量，刷新失败时保留已有缓存。

  # Trace: sync_antigravity_cache_non_explicit_short_circuits_without_connections
  Scenario: 没有活跃连接时不创建隐式同步缓存
    Given 本机没有活跃的 Antigravity language server 连接
    And 本机没有 Antigravity 缓存
    When 提交路径尝试隐式同步
    Then 同步跳过且不创建缓存目录

  # Trace: implicit_sync_deadline_bounds_stalled_metadata_and_reports_partial_cache
  Scenario: 隐式同步遇到停滞的服务时返回部分结果
    Given 活跃的 Antigravity language server 能列出会话但无法及时返回元数据
    When 提交路径隐式同步这些会话
    Then 同步 RPC 受共享的 60 秒预算限制
    And 同步返回部分结果和刷新失败计数
    And 本地缓存清单仍被保存

  # Trace: explicit_sync_keeps_its_cache_and_succeeds_when_a_session_cannot_be_refreshed
  Scenario: 显式同步允许单个会话刷新失败
    Given Antigravity language server 列出的一个会话已无法刷新
    When 用户执行 "tokens antigravity sync"
    Then 命令保存缓存清单并输出同步汇总和警告
    And 命令退出码为 0

  # Trace: explicit_sync_keeps_its_cache_and_succeeds_when_a_session_cannot_be_refreshed
  # Trace: sync_lock_guard_excludes_a_second_sync_until_the_first_is_dropped
  Scenario: 锁争用仍使显式同步失败
    Given 另一个 Antigravity 同步持有本地缓存锁
    When 用户执行 "tokens antigravity sync"
    Then 命令以非 0 退出码报告错误
    And 不覆盖另一个同步的缓存
