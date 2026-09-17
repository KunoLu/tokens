Feature: 站点页脚
  页脚只保留站点身份、法律链接和来源署名，不展示与本部署无关的托管方或赞助方信息。

  Rule: 页脚只保留版权与署名一行

    # 覆盖: tests/e2e/t9-acceptance.spec.ts「页脚只保留版权与署名行」。
    Scenario: 页脚不包含托管与赞助信息
      When 任意访客打开任意页面
      Then 页脚居中展示版权与来源署名行
      And 页脚不出现 "Workers"、"V.PS"、"Neon"
