Feature: 邮箱密码认证
  访客用邮箱、用户名和密码注册并登录。GitHub OAuth 不再是登录方式。
  排行榜不再展示 verified 徽章。CLI 仍通过设备授权拿到 token。

  Scenario: 新用户用有效资料注册后处于登录状态
    Given 访客打开注册页
    When 访客提交未被占用的邮箱、合法用户名、以及符合规则的密码
    Then 系统创建账号并建立会话
    And 访客被带到登录后的页面
    And 系统向该邮箱发出验证邮件

  Scenario: 弱密码在注册时被拒绝
    Given 访客打开注册页
    When 访客提交少于 8 位、或不含大写、或不含小写、或不含特殊字符的密码
    Then 注册失败
    And 页面说明密码规则
    And 系统不创建账号、不建立会话

  Scenario: 已注册用户使用正确密码登录
    Given 用户已经用邮箱注册
    When 用户提交正确的邮箱和密码
    Then 系统建立会话
    And 页面显示当前登录状态

  Scenario: 错误密码不能登录
    Given 用户已经用邮箱注册
    When 用户提交正确邮箱和错误密码
    Then 登录失败
    And 系统不建立会话
    And 响应不区分「邮箱不存在」与「密码错误」

  Scenario: 忘记密码无论邮箱是否存在都同样成功
    Given 访客打开忘记密码页
    When 访客提交任意邮箱地址
    Then 页面显示成功
    And 仅当该邮箱对应未封禁账号时系统才发送重置邮件

  Scenario: 使用有效重置令牌设置新密码
    Given 用户已收到未过期的重置令牌
    When 用户提交该令牌和符合规则的新密码
    Then 密码更新成功
    And 该用户的全部网页会话失效

  Scenario: 使用有效验证令牌标记邮箱已验证
    Given 用户已收到未过期的邮箱验证令牌
    When 用户提交该令牌
    Then 该账号的邮箱被标记为已验证

  Scenario: 已登录用户可以重发验证邮件
    Given 用户已经注册并持有会话
    And 验证邮件丢失或从未送达
    When 用户在验证页请求重发
    Then 系统向该账号邮箱再发一封验证邮件
    And 忘记密码不会把邮箱标记为已验证

  Scenario: 未登录访客请求重发验证邮件被拒绝
    Given 访客未登录
    When 访客 POST 请求重发验证邮件接口
    Then 系统返回 401 未认证错误
    And 系统不发送任何验证邮件

  Scenario: 导航与个人页不再提供 GitHub 登录
    Given 访客未登录
    When 访客查看导航或需要登录的页面
    Then 登录入口指向邮箱登录页
    And 页面上没有 GitHub 登录按钮或 GitHub OAuth 链接
    And 个人页没有跳转到 GitHub 资料的 GitHub 按钮

  Scenario: 排行榜与个人页不再展示 verified 徽章
    Given 任意用户曾经拥有两个以上社交链接
    When 访客打开排行榜或该用户的个人页
    Then 页面不展示 verified 徽章

  @todo
  # 阻塞：需要本机 CLI credentials 与可登录的 web 会话；本切片不改 Rust CLI。
  Scenario: CLI 设备授权在邮箱登录后仍能拿到 token
    Given 用户已在网页用邮箱登录
    When 用户在终端运行 tokens login 并在设备页确认
    Then CLI 仍能拿到可用的 API token
    And tokens submit 契约不变
