Feature: 文档页的安装入口
  访客需要在文档页直接看到把 CLI 接到本站的安装方式，不需要离开站点去别处找脚本。

  Rule: 安装命令使用本站自身的地址

    # 覆盖: tests/e2e/t9-acceptance.spec.ts「Docs 页安装入口指向本站」。
    Scenario: 安装命令不指向第三方站点
      When 访客打开文档页
      Then Linux 安装命令从本站下载安装脚本
      And 文档页不出现 tokens.ci 的链接

    Scenario: 文档页提供预安装脚本入口
      When 访客打开文档页
      Then 每个平台都有一条预安装脚本命令
      And 该命令从本仓库 GitHub 公开地址（钉住 tag 或 commit，不跟随分支漂移）下载脚本，并以本站地址为参数执行
      And 安装命令排在各平台命令列表的第一位，预安装命令在其后，登录命令在预安装之后
      And macOS 的 serve 说明写明 TOKENS_API_URL 是本站源地址，并动态显示本站实际地址
      And macOS 的 serve 说明文字带「CLI 手册」跳转链接
      And macOS 平台不出现 brew services（常驻服务读不到 shell alias）
      And Windows 的登录与提交使用预安装脚本装好的 tokens 命令（不用 bunx 直跑，确保指向本站）
      And 各平台在命令列表下方始终展示「重开终端（Windows 可执行 `. $PROFILE`）后再登录」的说明（含移动端视口）

    Scenario: 文档页提供常驻自动化脚本入口
      When 访客打开文档页
      Then Linux 与 Windows 的命令列表分别包含常驻/定时自动化脚本命令，且从本站地址下载执行
      And 说明写明这些脚本只配置常驻提交与站点指向，不替代登录
