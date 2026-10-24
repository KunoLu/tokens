Feature: 发现 Paseo 管理的原生客户端用量
  Paseo 配置提供目录位置，但不成为独立计费用量来源。

  # Trace: paseo_scan_discovers_profiles_and_archives_with_native_clients
  Scenario: 发现自定义配置与历史目录
    Given Paseo 配置含 Claude 和已禁用的 Codex profile 的绝对目录
    When 用户扫描原生客户端用量
    Then Claude 项目和 transcripts 归入 Claude
    And Codex sessions 和 archived_sessions 归入 Codex
    And Paseo timeline 不计入用量

  # Trace: paseo_scan_deduplicates_default_and_overlapping_profile_roots
  Scenario: 重叠目录只扫描一次
    Given 默认目录与多个 Paseo profile 指向同一个 Codex 会话
    When 用户扫描 Codex 用量
    Then 该会话仅发现一次

  # Trace: paseo_profile_overrides_base_env_and_skips_ambiguous_paths
  Scenario: profile 覆盖与不明确路径
    Given profile 覆盖基础 provider 的目录且其他 profile 使用相对路径或空值
    When 用户扫描用量
    Then 使用 profile 的绝对目录
    And 不推测相对路径或展开 provider 目录中的波浪号

  # Trace: paseo_missing_or_invalid_config_is_nonfatal；paseo_home_override_is_ignored_for_explicit_home
  Scenario: 无效配置不阻断原生扫描
    Given Paseo 配置缺失或不是合法 JSON
    When 用户扫描用量
    Then 仍可按原生默认目录扫描
