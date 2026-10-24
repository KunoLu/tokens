Feature: 推理速度用量兼容
  推理速度证据随提交保留，但不改变模型身份、计价或总用量。

  # Trace: inference_speed::claude_uses_reported_speed_without_losing_streamed_or_unknown_usage
  # Trace: inference_speed::codex_selection_is_scoped_to_the_turn_model_and_snapshot_owner
  Scenario: 区分服务端报告和用户选择的速度
    Given Claude 响应带有报告速度且 Codex 当前线程有模型绑定的速度选择
    When CLI 解析并汇总用量
    Then 已知速度分别标记为 reported 和 selected
    And 缺失证据的用量不被猜测为已知速度
    And 模型身份和总用量保持不变

  # Trace: web/src/lib/speedUsage.test.ts — legacy resubmits add usage without erasing speed; explicit empty clears it
  Scenario: 旧客户端再次提交保留已有速度信息
    Given 同一解析版本已有 100 tokens 的已知速度用量
    When 旧客户端提交 120 tokens 且省略速度信息
    Then 总用量更新为 120 tokens
    And 原有 100 tokens 的速度信息保留

  # Trace: web/src/lib/speedUsage.test.ts — legacy kilocode/kilo alias folding merges speed metadata
  Scenario: 旧客户端别名仍合并为同一客户端
    Given 当天同时存在 kilocode 和 kilo 的同模型用量
    When 服务端合并客户端用量
    Then 用量归入同一个 kilo 客户端
    And 模型计数和速度证据均累加

  # Trace: web/src/lib/speedUsage.test.ts — legacy payloads validate and keep their previous hash (absent or empty)
  Scenario: 没有速度信息的历史提交保持兼容
    Given 历史提交没有速度信息
    When 服务端校验并计算提交哈希
    Then 提交仍合法且哈希保持不变

  # Trace: /workspace/tokens-v104-b-cachetest/run.sh — new→old→new→old
  Scenario: 有完整源日志时升级降级保持统计一致
    Given 隔离环境中保留完整的 Claude 会话日志
    When 用户依次运行新旧新旧 CLI 的 dry-run 提交
    Then 四次命令均成功且统计总量一致
    And 缓存格式依次为 9、8、9、8
    And 降级不输出缓存格式警告
