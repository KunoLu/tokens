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
      And 该命令从本仓库 GitHub 公开地址（钉住 commit，不跟随分支漂移）下载脚本，并以本站地址为参数执行
