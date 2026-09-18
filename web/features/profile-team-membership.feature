Feature: 个人主页的团队信息
  用户需要在自己的主页上看到团队与分组归属，并能从这里退出。

  Rule: 有归属时才展示团队区块

    Scenario: 同时有团队和分组
      Given "songlin" 属于团队 "Neon Ravens" 和分组 "frontend"
      When 任意访客打开 "songlin" 的个人主页
      Then 页面展示团队 "Neon Ravens"
      And 页面展示分组 "frontend"

    Scenario: 只有团队没有分组
      Given "lena" 属于团队 "Atlas Guild" 但不属于任何分组
      When 任意访客打开 "lena" 的个人主页
      Then 页面展示团队 "Atlas Guild"
      And 页面不展示分组信息

    Scenario: 没有任何归属
      Given "rin" 不属于任何团队
      When 任意访客打开 "rin" 的个人主页
      Then 页面不展示团队与分组区块

  Rule: 只有本人可以看到并执行退出操作

    Scenario: 本人查看自己的主页
      Given "songlin" 已登录且属于团队 "Neon Ravens" 和分组 "frontend"
      When "songlin" 打开自己的个人主页
      Then 页面展示"退出 Team"和"退出 Group"操作

    Scenario: 他人查看该主页
      Given "songlin" 属于团队 "Neon Ravens"
      When 另一位用户打开 "songlin" 的个人主页
      Then 页面不展示任何退出操作

    Scenario: 未登录访客查看该主页
      Given "songlin" 属于团队 "Neon Ravens"
      When 未登录访客打开 "songlin" 的个人主页
      Then 页面不展示任何退出操作

  Rule: 退出分组不影响团队归属

    Scenario: 退出分组
      Given "mikoto" 已登录且属于团队 "Neon Ravens" 和分组 "frontend"
      When "mikoto" 从个人主页退出分组
      Then "mikoto" 不再属于任何分组
      And "mikoto" 仍然属于团队 "Neon Ravens"
      And 排行榜上 "mikoto" 的 Group 列变为空

  Rule: 退出团队会同时退出分组

    Scenario: 普通成员退出团队
      Given "mikoto" 已登录且属于团队 "Neon Ravens" 和分组 "frontend"
      When "mikoto" 从个人主页退出团队
      Then "mikoto" 不再属于任何团队
      And "mikoto" 不再属于任何分组
      And 排行榜上 "mikoto" 的 Team 列和 Group 列都变为空

    Scenario: admin 退出团队被阻断
      Given "songlin" 已登录且是团队 "Neon Ravens" 的 admin
      When "songlin" 从个人主页尝试退出团队
      Then 操作被阻断并提示需要先移交 admin 或解散团队
      And "songlin" 仍然属于该团队
