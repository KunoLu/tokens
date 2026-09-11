Feature: 界面语言切换
  用户需要在 English 与中文之间切换界面语言，以自己熟悉的语言使用产品。

  Rule: 语言切换按钮位于导航右上角主题切换的左侧

    Scenario: 按钮位置与选项
      When 任意访客打开任意页面
      Then 导航右上角的主题切换按钮左侧有一个语言切换按钮
      And 点击后展开下拉选项
      And 选项恰好包含 "English" 和 "中文"

  Rule: 切换后全部页面文案跟随

    Scenario: 切换到中文
      Given 当前语言为 English
      When 访客在语言切换器中选择 "中文"
      Then 当前页面所有文案变为中文
      And 导航、页面标题、按钮、表单标签、提示、空态、页脚均为中文
      And 数字与日期格式按中文语言环境渲染

    Scenario: 切换到 English
      Given 当前语言为中文
      When 访客在语言切换器中选择 "English"
      Then 当前页面所有文案变为英文

    Scenario: 切换后进入其他页面仍然生效
      Given 访客已将语言切换为 "中文"
      When 访客导航到排行榜页
      Then 排行榜页的文案为中文

  Rule: 选择跨会话持久化

    Scenario: 关闭浏览器后重新打开
      Given 访客已将语言切换为 "中文"
      When 访客关闭浏览器后重新打开网站
      Then 界面仍然为中文

  Rule: 页面的语言属性跟随选择

    Scenario: HTML lang 属性
      Given 访客已将语言切换为 "中文"
      When 访客打开任意页面
      Then 页面的 lang 属性为 "zh-CN"

  Rule: 隐私与条款页的中文版本标注英文为准

    Scenario: 隐私页面
      Given 访客已将语言切换为 "中文"
      When 访客打开隐私政策页
      Then 页面为中文
      And 页面明确标注英文版本具有最终效力

    Scenario: 条款页面
      Given 访客已将语言切换为 "中文"
      When 访客打开服务条款页
      Then 页面为中文
      And 页面明确标注英文版本具有最终效力
