Feature: 账号私密模式
  用户可以把自己的公开资料藏起来，同时继续提交用量。
  账号私密不是团队可见性。团队可见性仍只决定团队榜谁能打开。

  Rule: 私密默认关闭，公开面把私密账号当成不存在

    Scenario: 新账号默认仍出现在排行榜
      Given 开发者 "mira" 没有打开私密模式
      When 任意访客打开排行榜
      Then 排行榜可以出现 "mira"

    Scenario: 访客看不到私密账号的资料
      Given 开发者 "mira" 已打开私密模式
      When 未登录访客打开 "mira" 的资料页
      Then 页面显示找不到该用户
      And 页面不显示 "mira" 的用量

    Scenario: 主人仍能看到自己的私密资料
      Given 开发者 "mira" 已登录且已打开私密模式
      When "mira" 打开自己的资料页
      Then "mira" 看到自己的用量
      And 未登录访客仍看到找不到该用户

    Scenario: 私密账号不进入排行榜和全站人数
      Given 开发者 "mira" 已打开私密模式且有用量
      When 任意访客打开排行榜
      Then 列表中不出现 "mira"
      And 全站人数不把 "mira" 计算在内

    Scenario: 私密成员不出现在公开团队榜
      Given 开发者 "mira" 属于公开团队 "Neon Ravens" 且已打开私密模式
      When 任意访客在团队榜选择 "Neon Ravens"
      Then 列表中不出现 "mira"
      And 小组人数不把 "mira" 计算在内

    Scenario: 团队管理仍看得到私密成员
      Given 开发者 "mira" 属于团队 "Neon Ravens" 且已打开私密模式
      When 该团队的管理员查看成员名单
      Then 名单中仍有 "mira"

    Scenario: 用户名搜索找不到私密账号
      Given 开发者 "mira" 已打开私密模式
      When 团队管理员按用户名搜索 "mira"
      Then 搜索结果中不出现 "mira"

  Rule: 只有浏览器会话能把账号改回公开

    Scenario: 设置页可以打开和关闭私密模式
      Given 开发者 "mira" 已登录并打开设置页
      When "mira" 打开私密模式
      Then 设置页显示私密模式已打开
      When "mira" 关闭私密模式
      Then 设置页显示私密模式已关闭
      And 访客再次可以打开 "mira" 的资料页

    Scenario: 提交令牌不能把账号改回公开
      Given 开发者 "mira" 已打开私密模式
      When 有人使用 "mira" 的提交令牌请求改回公开
      Then 账号仍是私密
      And 访客仍看不到 "mira" 的资料

    Scenario: 登录时打开私密会给出一次性读令牌
      Given 开发者 "mira" 已在本机登录
      When "mira" 执行带私密开关的登录命令
      Then 命令提示私密模式已打开
      And 命令只这一次显示读令牌
      And 该读令牌不能用来提交用量

  Rule: 设备名对主人以外的人打码

    Scenario: 访客看到打码后的设备名
      Given 开发者 "mira" 的设备名是 "mira-macbook"
      When 未登录访客查看 "mira" 的公开资料
      Then 设备名显示为打码形式
      And 设备名不是 "mira-macbook"

    Scenario: 主人看到完整设备名
      Given 开发者 "mira" 已登录且设备名是 "mira-macbook"
      When "mira" 查看自己的资料
      Then 设备名显示 "mira-macbook"
