Feature: 团队管理
  用户需要把开发者组织成团队，以便按团队维度查看用量并管理成员。
  创建团队的人是该团队的 admin。

  Background:
    Given 用户 "songlin" 已登录

  Rule: 创建者成为团队的 admin

    Scenario: 创建团队
      When "songlin" 创建名为 "Neon Ravens" 的团队
      Then "Neon Ravens" 创建成功且状态为 active
      And "songlin" 在该团队中的角色是 admin
      And 该团队的成员数为 1

    Scenario: 一个用户只能属于一个团队
      Given "songlin" 已属于团队 "Neon Ravens"
      When "songlin" 尝试创建另一个团队
      Then 操作被拒绝并提示需要先退出当前团队

  Rule: 只有 admin 和 subadmin 可以修改团队资料与邀请成员

    Scenario Outline: 按角色区分资料修改权限
      Given "songlin" 在团队 "Neon Ravens" 中的角色是 <角色>
      When "songlin" 修改团队名称为 "Neon Ravens 2"
      Then 该操作<结果>

      Examples:
        | 角色     | 结果   |
        | admin    | 成功   |
        | subadmin | 成功   |
        | member   | 被拒绝 |

    Scenario: 普通成员不能邀请他人
      Given "songlin" 在团队 "Neon Ravens" 中的角色是 member
      When "songlin" 尝试邀请他人加入该团队
      Then 该操作被拒绝

  Rule: 团队可见性由 admin 在建团时选择，admin 与 subadmin 可修改

    Scenario: 建团时选择公开
      Given "songlin" 在建团表单中把可见性选为 "public"
      When "songlin" 提交表单
      Then 该团队的可见性是 "public"
      And 该团队出现在其他人的团队榜筛选项中

    Scenario: 未选择可见性时按私有落库
      Given "songlin" 提交建团表单时没有选择可见性
      When 团队被创建
      Then 该团队的可见性是 "private"

    Scenario: subadmin 可以修改可见性
      Given "mika" 是团队 "Neon Ravens" 的 subadmin
      When "mika" 把可见性改为 "private"
      Then 该团队的可见性是 "private"

    Scenario: 普通成员不能修改可见性
      Given "rin" 是团队 "Neon Ravens" 的普通成员
      When "rin" 尝试修改团队可见性
      Then 操作被拒绝
      And 该团队的可见性保持不变

  Rule: 邀请通过可搜索的下拉多选完成

    已注册用户从下拉列表中勾选，每个选项左侧有勾选框，选中即计数、可再点取消。
    未注册用户仍可通过输入完整邮箱地址来邀请。归一化、去重、回填的服务端逻辑
    不因交互方式改变而变化。
    下拉只展示用户名与显示名，不回显绑定邮箱；输入完整邮箱时按邮箱精确匹配，命中已注册用户也按其用户名展示，不回显邮箱。

    Scenario: 从下拉列表勾选一位已注册用户
      Given "songlin" 是 "Neon Ravens" 的 admin
      And 存在用户名为 "mikoto" 的账号
      When "songlin" 打开邀请对话框并勾选 "mikoto"
      Then 已选计数变为 1
      And 提交后 "mikoto" 收到一条待处理邀请

    Scenario: 输入内容实时筛选下拉选项
      Given "songlin" 是 "Neon Ravens" 的 admin
      And 存在用户名为 "mikoto" 的账号
      When "songlin" 在邀请对话框中输入 "mik"
      Then 下拉选项中显示 "mikoto"

    Scenario: 下拉列表不展示成员的绑定邮箱
      Given "songlin" 是 "Neon Ravens" 的 admin
      When "songlin" 在邀请对话框中输入 "mik"
      Then 下拉选项只显示用户名与显示名
      And 不显示任何成员的绑定邮箱

    Scenario: 输入完整邮箱匹配到已注册用户时按用户名展示
      Given "mikoto" 的绑定邮箱是 "mikoto@example.com"
      And "songlin" 是 "Neon Ravens" 的 admin
      When "songlin" 在邀请对话框中输入完整邮箱 "mikoto@example.com"
      Then 下拉中显示用户 "mikoto"
      And 不回显其绑定邮箱

    Scenario: 勾选多位成员后一次提交
      Given "songlin" 是 "Neon Ravens" 的 admin
      When "songlin" 在下拉中勾选 3 位成员
      Then 已选计数变为 3
      And 提交后系统创建 3 条待处理邀请

    Scenario: 取消勾选后计数减少
      Given "songlin" 已勾选 2 位成员
      When "songlin" 取消勾选其中 1 位
      Then 已选计数变为 1

    Scenario: 输入完整邮箱且未匹配到任何账户时可邀请该邮箱
      Given "songlin" 是 "Neon Ravens" 的 admin
      When "songlin" 在邀请对话框中输入 "chen.wei@example.com"
      And 没有任何已注册账户匹配该输入
      Then 列表底部出现「邀请该邮箱」选项
      And 选择该选项后系统为 "chen.wei@example.com" 创建待处理邀请

    Scenario: 该邮箱随后完成注册
      Given "songlin" 已邀请邮箱 "chen.wei@example.com" 且该邀请仍待处理
      When 该邮箱完成注册
      Then 新账号可以看到这条待处理邀请

    Scenario: 对同一人重复勾选不会重复创建邀请
      Given "songlin" 已邀请 "mikoto" 且该邀请仍待处理
      When "songlin" 再次通过下拉勾选 "mikoto" 并提交
      Then "mikoto" 在此团队下仍然只有 1 条待处理邀请

    Scenario: 同时勾选用户名路径与邮箱路径指向同一人时只产生一条
      Given "songlin" 是 "Neon Ravens" 的 admin
      And "mikoto" 是已注册用户，其绑定邮箱是 "mikoto@example.com"
      When "songlin" 在下拉中勾选 "mikoto" 并通过「邀请该邮箱」选择 "mikoto@example.com"
      Then 系统只创建 1 条待处理邀请

    Scenario: 输入一个不存在的用户名时下拉显示无匹配
      Given "songlin" 是 "Neon Ravens" 的 admin
      When "songlin" 在邀请对话框中输入 "nobody-here"
      Then 下拉显示无匹配账户
      And 不出现可提交的选项

    Scenario: 普通成员不能打开邀请对话框
      Given "songlin" 在 "Neon Ravens" 中的角色是 member
      When "songlin" 打开团队管理页
      Then 邀请成员按钮不可用或不可见

    Scenario: 非管理员不能调用用户搜索接口
      Given "rin" 在任何团队中都不是 admin 或 subadmin
      When "rin" 直接请求用户搜索接口
      Then 请求返回 403

    Scenario: 邀请过期后不再可用
      Given 存在一条 7 天前发出且未被处理的邀请
      When 受邀人尝试接受该邀请
      Then 操作被拒绝并提示邀请已过期

    Scenario: 接受邀请只会生效一次
      Given 存在一条待处理邀请
      When 受邀人连续两次提交接受
      Then 该用户只被加入团队一次

  Rule: 邮箱行与用户行指向同一个人时，保留创建时间较早的一条

    同一团队下理论上可能同时存在一条未关联用户的邮箱邀请，和一条指向该用户的用户邀请
    （并发提交，或日后引入换绑邮箱功能）。归一化必须给出确定的胜者，且置 superseded 要
    发生在回填之前，否则回填会先撞上 (team_id, invited_user_id) 的部分唯一索引。

    Scenario: 邮箱那条更早时保留邮箱那条并回填
      Given "Neon Ravens" 下存在一条指向邮箱 "chen.wei@example.com" 的待处理邀请，创建于 3 月 1 日
      And 同一团队下存在一条指向用户 "chenwei" 的待处理邀请，创建于 3 月 5 日
      And 邮箱 "chen.wei@example.com" 归属于用户 "chenwei"
      When 系统对该团队执行邀请归一化
      Then 3 月 5 日那条被置为 "superseded"
      And 3 月 1 日那条被回填 invited_user_id 指向 "chenwei"
      And "chenwei" 在此团队下只有 1 条待处理邀请

    Scenario: 用户那条更早时保留用户那条且不做回填
      Given "Neon Ravens" 下存在一条指向用户 "chenwei" 的待处理邀请，创建于 3 月 1 日
      And 同一团队下存在一条指向邮箱 "chen.wei@example.com" 的待处理邀请，创建于 3 月 5 日
      And 邮箱 "chen.wei@example.com" 归属于用户 "chenwei"
      When 系统对该团队执行邀请归一化
      Then 3 月 5 日那条被置为 "superseded"
      And 3 月 1 日那条保持不变，仍指向 "chenwei"
      And "chenwei" 在此团队下只有 1 条待处理邀请

    Scenario: 两条创建时间完全相同时按 id 较小者取胜
      Given "Neon Ravens" 下的邮箱行与用户行指向同一个人且创建时间完全相同
      When 系统对该团队执行邀请归一化
      Then id 较小的那条胜出，另一条被置为 "superseded"

    Scenario: 归一化不会因唯一索引冲突而失败
      Given "Neon Ravens" 下的邮箱行与用户行指向同一个人
      When 系统对该团队执行邀请归一化
      Then 归一化成功完成，不产生唯一索引冲突错误

    Scenario: 回填尚未完成时受邀人仍能看到邀请
      Given 邮箱 "chen.wei@example.com" 有一条待处理邀请且 invited_user_id 仍为空
      And "chen.wei@example.com" 已注册并验证了该邮箱
      When 她查看自己的待处理邀请列表
      Then 该条邀请出现在列表中

    Scenario: 归一化可以重复执行
      Given "Neon Ravens" 下的邀请已经归一化过一次
      When 系统再次对该团队执行邀请归一化
      Then 结果与第一次完全相同

  Rule: admin 最多指派 2 个 subadmin

    Scenario: 指派第一个 subadmin
      Given "Neon Ravens" 当前没有 subadmin
      When admin 把一名成员指派为 subadmin
      Then 该成员的角色变为 subadmin

    Scenario: 指派第三个 subadmin 被拒绝
      Given "Neon Ravens" 已有 2 个 subadmin
      When admin 尝试把第三名成员指派为 subadmin
      Then 操作被拒绝并提示 subadmin 名额已满

    Scenario: 并发指派不会突破名额上限
      Given "Neon Ravens" 已有 1 个 subadmin
      When admin 同时对两名成员发起指派
      Then 该团队的 subadmin 总数不超过 2

    Scenario: subadmin 不能指派其他 subadmin
      Given "songlin" 在 "Neon Ravens" 中的角色是 subadmin
      When "songlin" 尝试把一名成员指派为 subadmin
      Then 该操作被拒绝

  Rule: 团队必须先解散才能删除

    Scenario: 未解散的团队不能删除
      Given "Neon Ravens" 状态为 active 且有 8 名成员
      When admin 尝试删除该团队
      Then 操作被拒绝并提示需要先解散团队

    Scenario: 解散会清空全部成员
      Given "Neon Ravens" 有 8 名成员
      When admin 解散该团队
      Then 该团队状态变为 disbanded
      And 该团队的成员数为 0
      And 原先的 8 名成员都不再属于任何团队

    Scenario: 解散会同时清掉 admin 自己的成员身份
      Given "songlin" 是 "Neon Ravens" 的 admin
      When "songlin" 解散该团队
      Then "songlin" 不再属于任何团队
      And 该团队没有任何 admin 成员行

    Scenario: 解散后创建者仍可删除团队
      Given "Neon Ravens" 已解散且成员数为 0
      And 该团队已经没有任何成员行可用于角色鉴权
      When 创建者删除该团队
      Then 该团队被删除

    Scenario: subadmin 不能解散团队
      Given "songlin" 在 "Neon Ravens" 中的角色是 subadmin
      When "songlin" 尝试解散该团队
      Then 该操作被拒绝

    Scenario: subadmin 不能删除团队
      Given "Neon Ravens" 已解散且成员数为 0
      And "songlin" 曾是该团队的 subadmin
      When "songlin" 尝试删除该团队
      Then 该操作被拒绝

  Rule: 未解散的团队必须恰有一个 admin

    Scenario: admin 不能直接退出团队
      Given "songlin" 是 "Neon Ravens" 的 admin
      When "songlin" 尝试退出该团队
      Then 操作被拒绝并提示需要先移交 admin 或解散团队

    Scenario: 移交 admin 后原 admin 可以退出
      Given "songlin" 是 "Neon Ravens" 的 admin
      When "songlin" 把 admin 移交给另一名成员
      Then "songlin" 的角色变为 member
      And "songlin" 可以退出该团队
