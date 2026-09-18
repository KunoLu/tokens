Feature: 团队内分组
  团队需要把成员进一步分组，以便按分组筛选成员和查看用量。
  分组隶属于团队，其生命周期规则与团队一致。

  Background:
    Given 存在团队 "Neon Ravens"
    And "songlin" 是该团队的 admin

  Rule: 只有 admin 和 subadmin 可以管理分组

    Scenario Outline: 按角色区分建组权限
      Given "aki" 在 "Neon Ravens" 中的角色是 <角色>
      When "aki" 创建分组 "frontend"
      Then 该操作<结果>

      Examples:
        | 角色     | 结果   |
        | admin    | 成功   |
        | subadmin | 成功   |
        | member   | 被拒绝 |

    Scenario: subadmin 对分组拥有与 admin 相同的权限
      Given "aki" 在 "Neon Ravens" 中的角色是 subadmin
      And 存在分组 "frontend"
      Then "aki" 可以修改该分组名称
      And "aki" 可以调整该分组的成员
      And "aki" 可以解散该分组
      And "aki" 可以在解散后删除该分组

  Rule: 分组成员必须先是团队成员

    Scenario: 把团队成员加入分组
      Given "mikoto" 是 "Neon Ravens" 的成员
      And 存在分组 "frontend"
      When admin 把 "mikoto" 加入 "frontend"
      Then "mikoto" 属于分组 "frontend"

    Scenario: 非团队成员不能被加入分组
      Given "outsider" 不属于 "Neon Ravens"
      When admin 尝试把 "outsider" 加入 "frontend"
      Then 该操作被拒绝

    Scenario: 一名成员在团队内只属于一个分组
      Given "mikoto" 已属于分组 "frontend"
      When admin 把 "mikoto" 加入分组 "backend"
      Then "mikoto" 只属于分组 "backend"
      And "mikoto" 不再属于分组 "frontend"

    Scenario: 同一团队内分组名称唯一
      Given 存在分组 "frontend"
      When admin 再次创建名为 "frontend" 的分组
      Then 操作被拒绝并提示名称已存在

  Rule: 分组必须先解散才能删除

    Scenario: 未解散的分组不能删除
      Given 分组 "frontend" 有 3 名成员
      When admin 尝试删除该分组
      Then 操作被拒绝并提示需要先解散分组

    Scenario: 解散会清空分组成员
      Given 分组 "frontend" 有 3 名成员
      When admin 解散该分组
      Then 该分组状态变为 disbanded
      And 该分组的成员数为 0
      And 原先的 3 名成员仍然属于团队 "Neon Ravens"

    Scenario: 解散后可以删除分组
      Given 分组 "frontend" 已解散且成员数为 0
      When admin 删除该分组
      Then 该分组被删除

  Rule: 点击分组可以递进筛选成员

    Scenario: 按分组筛选团队成员
      Given "Neon Ravens" 有 8 名成员
      And 分组 "infra" 有 2 名成员
      When 查看者在团队管理页点击分组 "infra"
      Then 成员列表只展示这 2 名成员

    Scenario: 取消筛选后恢复全部成员
      Given 查看者已按分组 "infra" 筛选
      When 查看者点击"全部成员"
      Then 成员列表展示全部 8 名成员

  Rule: 退出团队时自动退出分组

    Scenario: 成员退出团队
      Given "mikoto" 属于团队 "Neon Ravens" 和分组 "frontend"
      When "mikoto" 退出团队
      Then "mikoto" 不再属于任何团队
      And "mikoto" 不再属于任何分组
