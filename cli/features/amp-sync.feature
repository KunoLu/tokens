Feature: 显式启用 Amp 云线程的用量同步
  Amp 自己管理登录；tokens 只缓存线程的计量数据，不保存对话内容。

  Scenario: 首次同步前不自动访问 Amp
    Given 用户没有执行过 tokens amp sync
    When 用户查看 Amp 状态或提交用量
    Then tokens 不执行 Amp 线程命令
    And 不创建同步状态

  Scenario: 首次同步部分成功不启用后台刷新
    Given 第一次显式同步只成功导出了部分线程
    When 用户进行普通提交
    Then 可读取已有用量，但不会自动访问 Amp
    And 用户必须显式再次同步才能完成启用

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

  Scenario: 未完整刷新的 Amp 缓存不能覆盖线上历史
    Given 用户已启用同步并准备替换一个日期范围的 Amp 用量
    And 其中一天的线程缓存缺失
    When 刷新该线程失败但另一天的缓存仍可读取
    Then CLI 拒绝覆盖提交，不向站点发送替换请求

  Scenario: 缺少同步状态的残留线程不能用于覆盖
    Given 同步中断后有线程文件但没有成功的状态记录
    When 用户尝试覆盖提交 Amp 用量
    Then CLI 不自动访问 Amp，也不发送覆盖请求

  Scenario: 发现成功不代表本地历史完整
    Given 一个以前已同步的线程不再出现在发现结果中且其本地副本已丢失
    When 其它可发现线程成功刷新后用户尝试覆盖提交
    Then CLI 拒绝用缺少该线程的缓存替换历史

  Scenario: 已发现但导出失败的线程后来退出搜索窗口
    Given 已启用同步且一次更新导出失败，旧线程文件仍存在
    When 下次搜索不再列出这条线程，用户尝试覆盖提交
    Then 在该更新成功导出前拒绝覆盖

  Scenario: 同一线程同时出现在旧本地目录与同步缓存
    Given 两个来源均包含相同 ID 的线程
    When CLI 扫描 Amp 用量
    Then 该线程只计一次
    And 仅存在于一个来源的其他线程也计入用量
