# 仅作规格，不接 Gherkin runner 或 CI；数据库与浏览器端到端验证未执行。
Feature: Claude Code cloud 会话设备
  云端会话作为独立设备计入用量，不改变私密可见性或普通设备名称掩码。

  # Trace: publicProfileDevices.test.ts；contract-backed mock，不是 full-stack。
  Scenario: 非所有者不能读取私密账号的云端设备
    Given 私密账号包含 Claude Code cloud 会话
    When 非所有者请求该账号的设备列表
    Then 返回 404 且不返回设备数据
    And 不执行云端活跃天数查询

  # Trace: publicProfileDevices.test.ts；访问检查、缓存策略与 cloud 查询的执行顺序。
  Scenario: 所有者读取私密云端设备时禁止共享缓存
    Given 所有者已获准访问其私密账号
    When 所有者请求设备列表
    Then 先确定私密缓存策略再处理云端设备
    And 响应携带私密 Cache-Control
    And 云端活跃天数仅包含该账号的会话

  # Trace: devices/cloud-labels.test.ts。
  Scenario: 云端设备自定义名不能替换固定标签
    Given Claude Code cloud 设备存有自定义名 "my laptop"
    When 公开展示设备名称
    Then 名称为 "Claude Code cloud" 且不打掩码
    And 无自定义名的云端设备显示相同标签
    And 普通设备自定义名仍打掩码

  @manual
  Scenario: 云端设备不能改名
    Given 所有者拥有一台云端会话设备
    When 所有者请求修改该设备名称
    Then 返回 400 且不修改名称

  # Trace: docs/cloud-sessions.md；不启用真实 cloud。
  Scenario: 文档明确 Codex cloud 不受支持
    Given 用户准备配置云端会话上报
    When 用户阅读云端会话使用文档
    Then 文档明确说明 Codex cloud 不受支持
    And 说明其 agent loop 不在容器内运行
