Feature: 界面语言切换
  用户需要在 English、简体中文与 日本語 之间切换界面语言，以自己熟悉的语言使用产品。
  语言由 URL 前缀表达（/zh-CN/...、/ja/...，无前缀即 English）；tt_locale cookie 只记住选择，供根路径 / 重定向使用。

  Rule: 语言切换按钮位于导航右上角主题切换的左侧

    # 覆盖: tests/e2e/i18n-locale.spec.ts「按钮位置与选项」。
    Scenario: 按钮位置与选项
      When 任意访客打开任意页面
      Then 导航右上角的主题切换按钮左侧有一个语言切换按钮
      And 点击后展开下拉选项
      And 选项恰好包含 "English"、"简体中文"、"日本語"

  Rule: 语言前缀决定页面语言

    Scenario: 打开带 zh-CN 前缀的排行榜
      When 访客打开 "/zh-CN/leaderboard"
      Then 页面的 lang 属性为 "zh-CN"
      And 导航显示 "排行榜"、"文档"、"团队榜"

    Scenario: 打开带 ja 前缀的排行榜
      When 访客打开 "/ja/leaderboard"
      Then 页面的 lang 属性为 "ja"
      And 导航显示 "リーダーボード"

    Scenario: 打开无前缀的排行榜
      When 访客打开 "/leaderboard"
      Then 页面的 lang 属性为 "en"
      And 导航显示 "Leaderboard"、"Docs"、"Teamboard"

  Rule: 旧无前缀中文链接仍然可用

    # 方案甲切换前，中文界面由 cookie 驱动、URL 不带前缀；这些旧链接不得 404。
    Scenario: 旧无前缀地址正常打开
      When 访客打开 "/leaderboard"
      Then 页面按 English 正常渲染，状态为 200
    Scenario: 根路径按记忆的语言重定向
      Given 访客带有 "tt_locale=zh-CN" cookie
      When 访客打开 "/"
      Then 访客被 307 重定向到 "/zh-CN/leaderboard"
    Scenario: 旧版 tt_locale=zh cookie 按 zh-CN 处理
      Given 访客带有旧版 "tt_locale=zh" cookie
      When 访客打开 "/"
      Then 访客被 307 重定向到 "/zh-CN/leaderboard"

  Rule: 切换控件改写 URL 前缀并记住选择

    # 覆盖: tests/e2e/i18n-locale.spec.ts「切换到简体中文」。
    Scenario: 切换到简体中文
      Given 访客在 "/leaderboard"
      When 访客在语言切换器中选择 "简体中文"
      Then 地址变为 "/zh-CN/leaderboard"
      And 页面以简体中文渲染
      And "tt_locale" cookie 的值为 "zh-CN"

    # 覆盖: tests/e2e/i18n-locale.spec.ts「切换到 English」。
    Scenario: 切换到 English
      Given 访客在 "/zh-CN/leaderboard"
      When 访客在语言切换器中选择 "English"
      Then 地址变为 "/leaderboard"
      And 页面以 English 渲染

    # 覆盖: tests/e2e/i18n-locale.spec.ts「从 / 回到记忆语言」。
    Scenario: 从 / 回到记忆语言
      Given 访客已把语言切换为 "简体中文"
      When 访客之后打开 "/"
      Then 访客被重定向到 "/zh-CN/leaderboard"

  Rule: 已包裹的页面文案跟随语言前缀

    Scenario: 设置页未登录重定向保留前缀
      When 未登录访客打开 "/zh-CN/settings"
      Then 访客被重定向到 "/login?returnTo=/zh-CN/settings"

    Scenario: 隐私页面
      When 访客打开 "/zh-CN/privacy"
      Then 页面为中文
      And 页面明确标注英文版本具有最终效力

    Scenario: 条款页面
      When 访客打开 "/ja/terms"
      Then 页面为日文
      And 页面明确标注英文版本具有最终效力

    # 覆盖: tests/e2e/i18n-locale.spec.ts「中文界面登录失败」。
    Scenario: 中文界面登录失败显示中文错误
      Given 访客在 "/zh-CN/login"
      When 访客用错误密码提交登录
      Then 页面提示「邮箱或密码不正确」
