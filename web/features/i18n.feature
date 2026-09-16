Feature: 界面语言切换
  用户需要在 English 与中文之间切换界面语言，以自己熟悉的语言使用产品。

  Rule: 语言切换按钮位于导航右上角主题切换的左侧

    # 覆盖: tests/e2e/i18n-locale.spec.ts「按钮位置与选项」。
    Scenario: 按钮位置与选项
      When 任意访客打开任意页面
      Then 导航右上角的主题切换按钮左侧有一个语言切换按钮
      And 点击后展开下拉选项
      And 选项恰好包含 "English" 和 "中文"

  Rule: 已包裹的导航文案跟随语言偏好

    Scenario: 中文语言偏好打开排行榜
      Given 访客的语言偏好为中文
      When 访客打开排行榜
      Then 页面的 lang 属性为 "zh-CN"
      And 导航显示 "排行榜"、"文档"、"团队榜"

    Scenario: English 语言偏好打开排行榜
      Given 访客的语言偏好为 English
      When 访客打开排行榜
      Then 页面的 lang 属性为 "en"
      And 导航显示 "Leaderboard"、"Docs"、"Teamboard"

    Scenario: 无法识别的语言偏好回退为 English
      Given 访客带有无法识别的语言偏好
      When 访客打开排行榜
      Then 页面按 English 渲染

  Rule: 用量数字跟随当前语言的 NumberFormat

    Scenario: 中文界面的用量数字按中文语言环境渲染
      Given 访客的语言偏好为中文
      When 访客打开排行榜
      Then 用量数字按中文语言环境的 NumberFormat 渲染

  Rule: 切换控件把选择写入偏好并刷新

    # 覆盖: tests/e2e/i18n-locale.spec.ts「切换到中文」。
    Scenario: 切换到中文
      Given 当前语言为 English
      When 访客在语言切换器中选择 "中文"
      Then 当前已包裹的导航文案变为中文
      And 用量数字按中文语言环境的 NumberFormat 渲染（紧凑如 123.5万，非紧凑货币如 US$1,234,567.89）

    # 覆盖: tests/e2e/i18n-locale.spec.ts「切换到 English」。
    Scenario: 切换到 English
      Given 当前语言为中文
      When 访客在语言切换器中选择 "English"
      Then 当前已包裹的导航文案变为英文

    # 覆盖: tests/e2e/i18n-locale.spec.ts「关闭浏览器后重新打开」。
    Scenario: 关闭浏览器后重新打开
      Given 访客已将语言切换为 "中文"
      When 访客关闭浏览器后重新打开网站
      Then 界面仍然为中文

  Rule: 已包裹的页面文案跟随语言偏好

    # 覆盖: tests/e2e/i18n-locale.spec.ts「切换后全部页面文案跟随」。
    Scenario: 切换后全部页面文案跟随
      Given 访客的语言偏好为中文
      When 访客依次打开排行榜、团队榜、团队、文档、本地查看器和认证页面
      Then 每个页面的标题和主要操作文案均为中文
      And 排行榜、文档和认证表单的已包裹说明文案均为中文

    # 覆盖: tests/e2e/i18n-locale.spec.ts「已登录设置页」。
    Scenario: 已登录用户打开设置页
      Given 用户已登录
      And 访客的语言偏好为中文
      When 访客打开设置页
      Then 页面标题为 "设置"
      And 页面说明文案为中文

    # 覆盖: tests/e2e/i18n-locale.spec.ts「隐私页面」。
    Scenario: 隐私页面
      Given 访客已将语言切换为 "中文"
      When 访客打开隐私政策页
      Then 页面为中文
      And 页面明确标注英文版本具有最终效力

    # 覆盖: tests/e2e/i18n-locale.spec.ts「条款页面」。
    Scenario: 条款页面
      Given 访客已将语言切换为 "中文"
      When 访客打开服务条款页
      Then 页面为中文
      And 页面明确标注英文版本具有最终效力

    # 覆盖: tests/e2e/i18n-locale.spec.ts「中文界面登录失败」。
    Scenario: 中文界面登录失败显示中文错误
      Given 访客的语言偏好为中文
      When 访客用错误密码提交登录
      Then 页面提示「邮箱或密码不正确」

