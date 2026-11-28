Feature: 显式启用 Amp 云线程的用量同步
  Amp 自己管理登录；tokens 只缓存线程的计量数据，不保存对话内容。

  Scenario: 首次同步前不自动访问 Amp
    Given 用户没有执行过 tokens amp sync
    When 用户查看 Amp 状态或提交用量
    Then tokens 不执行 Amp 线程命令
    And 不创建同步状态

  Scenario: 同步时只保留用量
    Given 已登录的 Amp 导出含用户提示、回复、工具输出和标题的线程
    When 用户执行 tokens amp sync
    Then 线程缓存保留 ID、日期与计量字段
    And 即使私密内容出现在未知的嵌套字段中也不会落盘

  Scenario: 自动刷新失败时仍可使用已有用量
    Given 用户已显式同步过 Amp，缓存中已有一条线程
    When 后续提交时 Amp 无法导出该线程的更新
    Then 提示同步失败且不输出 Amp 的原始私密数据
    And 原有缓存和该线程的成功水位保持不变
    And 下次同步仍会尝试获取更新

  Scenario: 同一线程同时出现在旧本地目录与同步缓存
    Given 两个来源均包含相同 ID 的线程
    When CLI 扫描 Amp 用量
    Then 该线程只计一次
    And 仅存在于一个来源的其他线程也计入用量
