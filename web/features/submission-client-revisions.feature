Feature: 网站接收客户端用量与解析修正
  # Trace: web/src/lib/validation/submission.test.ts；提交校验与日用量合并契约，不运行生产数据库。

  Scenario Outline: 接收新客户端的第 1 版用量
    Given CLI 已为 <客户端> 生成第 1 版日用量
    When CLI 提交用量
    Then 网站接受该客户端标识与用量

    Examples:
      | 客户端   |
      | qoder    |
      | kwwk     |
      | trae-cli |

  Scenario Outline: 新解析版本纠正同一设备的日用量
    Given <客户端> 的第 1 版解析将 3000 个 token 记在同一天
    When 同一设备用第 2 版重新提交当天的 1000 个 token
    Then 当天该客户端的计数为 1000
    And 重放第 1 版不会恢复到 3000

    Examples:
      | 客户端 |
      | hermes |
      | amp    |

  Scenario Outline: 未识别的修订不能覆盖已存用量
    Given 网站最高支持 <客户端> 的第 2 版解析
    When CLI 提交第 3 版用量
    Then 网站拒绝提交

    Examples:
      | 客户端 |
      | hermes |
      | amp    |
