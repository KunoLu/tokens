Feature: 排行榜与团队榜
  查看者需要在排行榜上看到开发者的团队归属，并能按团队和分组查看排名。

  Rule: Leaderboard 在 Developer 之后展示 Team 与 Group 两列

    Scenario: 列顺序
      When 任意访客打开排行榜页
      Then 表格的列从左到右依次是 "#"、"Developer"、"Team"、"Group"、"Tokens"、"Cost"

    Scenario: 有团队归属的开发者
      Given 开发者 "songlin" 属于团队 "Neon Ravens" 和分组 "frontend"
      When 任意访客打开排行榜页
      Then "songlin" 所在行的 Team 列显示 "Neon Ravens"
      And "songlin" 所在行的 Group 列显示 "frontend"

    Scenario: 没有团队归属的开发者留空
      Given 开发者 "rin" 不属于任何团队
      When 任意访客打开排行榜页
      Then "rin" 所在行的 Team 列为空
      And "rin" 所在行的 Group 列为空

    Scenario: 有团队但没有分组的开发者只留空 Group
      Given 开发者 "lena" 属于团队 "Atlas Guild" 但不属于任何分组
      When 任意访客打开排行榜页
      Then "lena" 所在行的 Team 列显示 "Atlas Guild"
      And "lena" 所在行的 Group 列为空

    Scenario: 已解散团队的成员不再显示团队名
      Given 开发者 "songlin" 所属的团队已解散
      When 任意访客打开排行榜页
      Then "songlin" 所在行的 Team 列为空

  Rule: 新增列不改变既有的排序与数字格式行为

    Scenario: 按 Cost 排序仍然可用
      Given 访客打开排行榜页
      When 访客把排序切换为 "Cost"
      Then 列表按 Cost 从高到低重新排列

    Scenario: Tokens 表头切换的是数字格式而非排序
      Given 访客打开排行榜页
      When 访客点击 "Tokens" 表头
      Then Tokens 列在缩写与精确数字之间切换
      And 列表的排列顺序保持不变

  # 团队可见范围见 docs PRD §13 D-2：teams.visibility 取 public / private，
  # 建团时由 admin 显式选择，DB 默认 private。

  Rule: Teamboard 只能选到公开团队与自己所属的团队

    Scenario: 访客可以浏览公开团队
      Given 团队 "Neon Ravens" 的可见性是 "public"
      And 未登录访客打开团队榜页
      When 访客选择团队 "Neon Ravens"
      Then 列表展示该团队的成员

    Scenario: 私有团队不出现在筛选器中
      Given 团队 "Atlas Guild" 的可见性是 "private"
      And "songlin" 不属于 "Atlas Guild"
      When "songlin" 展开团队筛选项
      Then 候选项中不出现 "Atlas Guild"

    Scenario: 直接请求他人的私有团队返回 404
      Given 团队 "Atlas Guild" 的可见性是 "private"
      And "songlin" 不属于 "Atlas Guild"
      When "songlin" 用 "Atlas Guild" 的标识直接请求团队榜数据
      Then 请求返回 404
      And 不返回任何成员数据

    Scenario: 成员始终能浏览自己所属的私有团队
      Given 团队 "Atlas Guild" 的可见性是 "private"
      And "lena" 属于 "Atlas Guild"
      When "lena" 打开团队榜页
      Then 团队筛选项显示 "Atlas Guild"
      And 列表展示该团队的成员

    Scenario: 团队转为私有后非成员立即不可见
      Given 访客正在查看公开团队 "Neon Ravens"
      When 该团队的 admin 把可见性改为 "private"
      And 访客刷新页面
      Then 访客看不到 "Neon Ravens" 的成员数据

    Scenario: 登录用户默认选中自己的团队
      Given "songlin" 已登录且属于团队 "Neon Ravens"
      When "songlin" 打开团队榜页
      Then 团队筛选项显示 "Neon Ravens"
      And 列表展示该团队的成员

    Scenario: 尚未选择团队
      Given 未登录访客打开团队榜页
      Then 页面提示先选择一个团队
      And 不展示成员列表

    Scenario: 团队筛选是单选
      Given 访客已选中公开团队 "Neon Ravens"
      When 访客选择公开团队 "Kite Works"
      Then 列表只展示 "Kite Works" 的成员

    Scenario: 列中不包含 Team
      When 访客在团队榜页选中任意团队
      Then 表格的列从左到右依次是 "#"、"Developer"、"Group"、"Tokens"、"Cost"

  Rule: Teamboard 的分组筛选是多选

    Scenario: 选择单个分组
      Given "songlin" 正在查看团队 "Neon Ravens"
      When "songlin" 勾选分组 "frontend"
      Then 列表只展示 "frontend" 的成员

    Scenario: 同时选择多个分组
      Given "songlin" 正在查看团队 "Neon Ravens"
      And 分组 "frontend" 有 3 名成员
      And 分组 "infra" 有 2 名成员
      When "songlin" 同时勾选 "frontend" 和 "infra"
      Then 列表展示这 5 名成员
      And 名次按用量重新从 1 开始编号

    Scenario: 不选分组时展示团队全部成员
      Given "songlin" 正在查看团队 "Neon Ravens"
      When "songlin" 不勾选任何分组
      Then 列表展示该团队的全部成员

    Scenario: 切换团队会清空已选分组
      Given 访客已选中公开团队 "Neon Ravens" 并勾选了分组 "frontend"
      When 访客切换到公开团队 "Kite Works"
      Then 分组筛选被清空

    Scenario: 公开团队的筛选条件可以通过链接分享
      Given "songlin" 在公开团队 "Neon Ravens" 上勾选了两个分组并复制了当前页面链接
      When 任意访客打开该链接
      Then 访客看到相同的团队与分组筛选结果

    Scenario: 私有团队的链接不会泄露给非成员
      Given "lena" 在私有团队 "Atlas Guild" 上勾选了两个分组并复制了当前页面链接
      When 不属于该团队的 "rin" 打开该链接
      Then "rin" 看不到该团队的成员数据

  Rule: 封禁用户不出现在任何榜单

    Scenario: 被封禁的团队成员
      Given 团队 "Neon Ravens" 中有一名成员已被封禁
      When 访客在团队榜页选中 "Neon Ravens"
      Then 该成员不出现在列表中
