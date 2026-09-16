Feature: 定时维护任务
  自建部署没有 Cloudflare Workers 定时器，运维通过受密钥保护的 HTTP 接口触发每日清理，
  并依据响应状态决定是否需要重试。

  Rule: 接口必须携带有效密钥

    # 覆盖: tests/e2e/cron.spec.ts「未携带密钥被拒绝」。
    Scenario: 未携带密钥被拒绝
      When 运维不带密钥调用定时维护接口
      Then 接口返回 401
      And 不执行任何清理

  Rule: 所有清理完成后接口才返回成功

    # @todo refreshAllSocialLinks 对每个用户都请求 GitHub（无 github_id 过滤），且会改写共享
    # 开发库的社交链接快照：正式 e2e 会依赖外部网络并污染数据。200 路径由部署验收中生产定时
    # 任务的首次运行验证；本任务内对无 GitHub 关联账号的开发库手动验证一次。
    @todo
    Scenario: 全部清理成功
      Given 服务端已配置定时任务密钥
      When 运维携带正确密钥调用定时维护接口
      Then 接口返回 200
      And 过期邮箱验证令牌已被删除
      And 过期团队邀请已被标记为 expired

    # @todo 需要可注入的单任务故障才能自动化；手动验证：停库后携带密钥调用返回非 2xx，调度器据此重试。
    @todo
    Scenario: 任一清理失败时返回可重试的失败状态
      Given 服务端已配置定时任务密钥
      And 其中一项清理无法完成
      When 运维携带正确密钥调用定时维护接口
      Then 接口返回非 2xx 状态
