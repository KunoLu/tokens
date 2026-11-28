Feature: CLI 按客户端来源与消息日期统计本地用量
  本地扫描不读取 Claude 网页、桌面、移动应用或 Projects 里的聊天。

  Scenario: 两个 Qoder 版本归入同一个客户端
    Given Qoder 和 Qoder CN 均记录了带计数的请求
    When CLI 扫描本地用量
    Then 这两个版本都归入 qoder
    And 同一请求出现在多个来源时不重复计数
    And 不同版本即使请求标识相同也分别计数

  Scenario: Qoder 多个来源对同一请求提供不同完整度的记录
    Given 一份记录只有用量和文件修改时间，另一份含请求实际发生时间
    When CLI 合并同一请求并按日计价
    Then 用量归到实际发生的日期，不随文件修改时间漂移
    And 明确报告的请求模型优先于会话的默认模型

  Scenario: 区分独立客户端及其记录格式
    Given 本地存在 kwwk 的 pi 格式用量和 TRAE CLI 会话
    When CLI 扫描本地用量
    Then kwwk 的用量归入 kwwk 而不是 pi
    And TRAE CLI 的用量归入 trae-cli 而不是 trae

  Scenario: kwwk 同毫秒的不同摘要不会因不完整副本而错位
    Given 同一会话有两次同毫秒摘要，分别消耗 20 和 3 个输入 token
    And 另一份副本将第一次摘要的用量记为零
    When CLI 同时读取两个副本
    Then 两次摘要的输入合计为 23 个 token

  Scenario Outline: 跨日会话按消息日期归账
    Given <客户端> 的一个会话含两个不同日期的带用量助手消息
    When CLI 汇总这两个日期的用量
    Then 各日只包含属于当天的消息用量

    Examples:
      | 客户端 |
      | Hermes |
      | Amp    |

  Scenario: 本地发现新的用量来源
    Given 本地存在带用量的 Copilot SDK session-state、Antigravity 桌面会话库及 T3 下的 Antigravity ACP 会话
    When CLI 扫描这些来源
    Then 每个来源的已记录用量计入对应客户端

  Scenario: Copilot 日志报告的输入已包含缓存
    Given SDK 将同一批输入的缓存读取和写入同时列在输入总量之内
    When CLI 汇总用量
    Then 每个输入 token 只计一次，缓存类别仍分别保留

  Scenario: 已取消的 Copilot 模型选择不再影响下一次请求
    Given 用户从指定模型切回自动选择
    When 后续请求没有报告具体模型
    Then CLI 不把该用量记到已取消的模型

  Scenario: pi 格式保留已上报速度
    Given 一条 pi 格式用量记录包含明确上报的推理速度
    When CLI 解析该记录
    Then 保留已上报速度且不改变总 token 数

  Scenario: 自动提交容忍调度抖动
    Given 上次自动提交已开始且正常结束
    When 下一次调度因轻微抖动提前到达允许窗口
    Then CLI 不会仅因上次提交的执行耗时而跳过本次提交
    And 相对日期筛选不会因为跨午夜的早醒而使用前一天
