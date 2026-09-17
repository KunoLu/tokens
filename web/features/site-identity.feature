Feature: 站点身份
  站点所有用户可见的品牌字样、链接与联系邮箱都来自部署自身配置
  （NEXT_PUBLIC_URL / CONTACT_EMAIL），不残留 tokens.ci 硬编码。

  Rule: 页面与图像文案不出现 tokens.ci，品牌字样显示本站 host

    # 覆盖: tests/e2e/site-identity.spec.ts「公共页面 HTML 无 tokens.ci 且品牌字样为本站 host」。
    Scenario: 公共页面不包含 tokens.ci
      When 访客打开 /privacy、/terms、/docs、/leaderboard
      Then 页面 HTML 不包含 tokens.ci
      And 品牌链接与 og:url 指向本站 host

    # 覆盖: tests/e2e/site-identity.spec.ts「OG 图与 embed SVG 不含 tokens.ci」。
    Scenario: OG 与 embed 图像文案不残留上游域名
      When 请求 /api/og 与 /api/embed/<username>/svg
      Then 响应内容不包含 tokens.ci
      And embed SVG 的品牌字样为本站 host

  Rule: 联系邮箱由 CONTACT_EMAIL 环境变量驱动

    # 覆盖: tests/e2e/site-identity.spec.ts「未配置 CONTACT_EMAIL 时法律页无 mailto 链接」。
    Scenario: 未配置 CONTACT_EMAIL 时法律页无 mailto 链接
      Given 部署未配置 CONTACT_EMAIL
      When 访客打开 /privacy 或 /terms
      Then 页面不出现 mailto: 链接
      And 联系区块保留本站链接
    # 手工验证（2026-09-17）：一次性 dev server 配 CONTACT_EMAIL=ops@localhost.dev，
    # privacy 10 个 mailto、terms 7 个、无 mailto:undefined。e2e 栈不配置 CONTACT_EMAIL，此场景未自动化。
    Scenario: 配置 CONTACT_EMAIL 后法律页渲染邮箱链接
      Given 部署配置了 CONTACT_EMAIL
      When 访客打开 /privacy 或 /terms
      Then 联系区块渲染指向该邮箱的 mailto 链接
