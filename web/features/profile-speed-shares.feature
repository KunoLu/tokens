# UI 层 manual-blocked：无数据库和浏览器 E2E；在有真实 profile 数据的环境逐场景补验。
# 当前仅执行下列算法／格式化 trace，不将其作为页面渲染通过。
@manual
Feature: 个人用量占比与推理速度
  占比解释 token 构成，速度档位不改变模型总量或费用。

  # Trace: format percentage smoke；UI 渲染因无数据库环境未运行。
  Scenario: 非零微小用量不显示为零占比
    Given 某 token 类别占总量的 0.01%
    When 用户查看概览、当天或 token 构成图例
    Then 该类别显示小于 0.1% 而非 0.0%
    And 文案使用当前中文或英文语言

  # Trace: speedUsage.test.ts — tier rows
  Scenario: 有部分速度证据时显示未知余量
    Given 模型共使用 100 tokens 且仅 60 tokens 有已知速度
    When 用户查看模型速度明细
    Then 已知档位显示 60 tokens
    And 未知档位显示剩余 40 tokens
    And 提示区分服务端报告与用户选择

  # Trace: speedUsage.test.ts — no known speed
  Scenario: 历史数据没有速度证据
    Given 模型没有任何速度证据
    When 用户查看当天、日详情或周期模型表
    Then 不显示速度档位行

  # Trace: ProfileModels 源码与 typecheck；真实周期切换、分母标示及中文换行需人工补验。
  Scenario: 周期模型表区分速度占比与费用占比
    Given 模型用量含已知速度且周期数据没有消息总数
    When 用户查看周期模型表
    Then 速度占比以该模型的 tokens 为分母并嵌套于模型单元格
    And 原费用占比列含义不变
    And 速度行不显示消息数
