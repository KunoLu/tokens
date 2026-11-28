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

  Scenario: 完整刷新也不能用云端线程替换历史
    Given 用户已启用同步且所有找到的线程均已成功导出
    When 用户提交带日期范围的 Amp --replace
    Then CLI 拒绝覆盖，不调用 Amp，也不向站点发送替换请求

  Scenario: 缺少同步状态的残留线程不能用于覆盖
    Given 同步中断后有线程文件但没有成功的状态记录
    When 用户尝试覆盖提交 Amp 用量
    Then CLI 不自动访问 Amp，也不发送覆盖请求

  Scenario: 发现成功不代表本地历史完整
    Given 一个以前已同步的线程不再出现在发现结果中且其本地副本已丢失
    When 其它可发现线程成功刷新后用户尝试覆盖提交
    Then CLI 拒绝用缺少该线程的缓存替换历史

  Scenario: 纯旧版本地 Amp 仍可预览范围覆盖
    Given 当前配置从未启动 Amp 云同步且没有同步缓存
    When 用户预览范围内的本地 Amp 替换提交
    Then 清单保留该范围的完整覆盖标记且不访问 Amp

  Scenario: 同一线程同时出现在旧本地目录与同步缓存
    Given 两个来源均包含相同 ID 的线程
    When CLI 扫描 Amp 用量
    Then 该线程只计一次
    And 仅存在于一个来源的其他线程也计入用量

  Scenario: Linux 上层进程不回收孤儿时仍可同步
    Given 本地 Amp 命令留有同进程组的后台子进程
    And 容器的上层进程不回收孤儿
    When Amp 命令成功输出并退出
    Then CLI 清理该组并保留有效输出，不把已退出的子进程当作仍在运行

  Scenario: 手动强制同步可恢复已退出发现窗口的待办
    Given 一条已缓存线程的更新导出失败，此后不再出现在本次账号的发现结果中
    When 用户手动强制同步并成功导出全部可见线程
    Then 仅清除不可见线程的重试待办并报告数量，不请求旧 ID，不删除缓存文件
    And 以后重新发现该线程时重新导出，即使其列表时间没有变化
