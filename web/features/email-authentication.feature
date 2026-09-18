Feature: 邮箱注册与登录
  用户需要用自有账号进入产品，以便上报用量并参与团队协作。
  账号体系不再依赖 GitHub。

  Rule: 注册必须同时提供邮箱、用户名和符合强度要求的密码

    Scenario: 使用合规信息成功注册
      Given 访客打开注册页
      When 访客填写未被占用的邮箱、未被占用的用户名和密码 "Abcdef1!"
      And 访客提交注册
      Then 访客进入已登录状态
      And 系统向该邮箱发送验证邮件

    Scenario: 密码长度不足被拒绝
      Given 访客打开注册页
      When 访客填写密码 "Ab1!"
      Then 页面提示密码至少需要 8 位
      And 注册按钮保持不可用

    Scenario: 密码缺少特殊字符被拒绝
      Given 访客打开注册页
      When 访客填写密码 "Abcdefg1"
      Then 页面提示密码需要至少 1 个特殊字符
      And 注册按钮保持不可用

    Scenario: 密码缺少大写字母被拒绝
      Given 访客打开注册页
      When 访客填写密码 "abcdef1!"
      Then 页面提示密码需要包含大写字母
      And 注册按钮保持不可用

    Scenario: 邮箱已被注册
      Given 已存在使用邮箱 "taken@example.com" 的账号
      When 访客用同一邮箱提交注册
      Then 页面提示该邮箱已被注册
      And 系统不创建新账号

    Scenario: 用户名已被占用且不区分大小写
      Given 已存在用户名为 "songlin" 的账号
      When 访客用用户名 "SongLin" 提交注册
      Then 页面提示该用户名已被占用

  Rule: 登录只接受邮箱与密码

    Scenario: 使用正确密码登录
      Given 存在一个已注册账号
      When 该用户提交正确的邮箱和密码
      Then 该用户进入自己的已登录状态

    Scenario: 密码错误
      Given 存在一个已注册账号
      When 该用户提交错误的密码
      Then 页面提示邮箱或密码不正确
      And 提示不透露该邮箱是否已注册

    Scenario: 被封禁的账号无法登录
      Given 存在一个已被封禁的账号
      When 该账号提交正确的邮箱和密码
      Then 登录被拒绝并说明账号已被封禁

  Rule: 产品中不再存在任何 GitHub 登录入口

    Scenario: 导航右上角没有 GitHub 图标
      When 任意访客打开排行榜页
      Then 页面顶部导航不展示 GitHub 图标

    Scenario: 个人主页没有 GitHub 按钮
      Given 存在一个已注册账号
      When 任意访客打开该用户的个人主页
      Then 页面右上角不展示 GitHub 按钮

  Rule: 命令行工具的登录方式保持不变

    Scenario: CLI 通过设备码完成登录
      Given 用户已在浏览器中登录
      When 用户在命令行执行登录并在浏览器输入设备码完成授权
      Then 命令行获得可用的 API token
      And 用户可以用该 token 成功提交用量数据
