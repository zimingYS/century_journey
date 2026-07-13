# 项目结构

本文档描述当前仓库中真实存在并参与编译的模块，不把规划功能写成已实现能力。

## 运行链路

    main
      -> app
      -> client::plugin_group
         -> engine
         -> content
         -> game
         -> client

App 负责选择运行模式和装配插件；Content 提供定义；Game 执行规则；
Client 负责输入与表现；Engine 和 Shared 提供底层能力与共享类型。

新世界进入游戏时，App 发出 ContentReloadRequested；Content 先重建数据注册表，
随后 Game 刷新玩法缓存，Client 刷新材质、HUD 等表现资源。底层模块不读取 App
内部的会话资源。

## 源码目录

    src/
    ├── app/       应用入口、配置、状态和菜单流程
    ├── client/    输入、UI、渲染、音频、粒子和本地玩家表现
    ├── content/   方块、物品、生物群系、配方、掉落表和标签定义
    ├── engine/    资源系统、常量和异步任务门面
    ├── game/      世界、玩家、物品栏、合成与玩法规则
    ├── shared/    跨层共享组件、标识符、状态和时间类型
    ├── editor/    编辑器规划边界，当前未实现
    ├── protocol/  联机协议规划边界，当前未实现
    └── server/    专用服务端规划边界，当前未实现

## 依赖方向

- Engine 不依赖任何玩法模块。
- Shared 只保存跨层数据类型，不实现玩法。
- Content 可以依赖 Engine 和 Shared，不依赖 Game 或 Client。
- Game 消费 Content 定义，不依赖 Client。
- Client 可以消费 Game、Content 和 Shared，但只负责输入与表现。
- App 位于装配层，不复制下层业务逻辑。

区块网格任务、贪心网格构建和材质挂载归 Client::renderer 所有；Game 只维护
区块数据与生成阶段。掉落物的物理、合并和生命周期归 Game 所有，模型生成归
Client::renderer 所有。

## 自动边界检查

tests/architecture_boundaries.rs 会递归扫描 Rust 源码并拒绝以下依赖：

- Engine 依赖 App、Client、Content 或 Game。
- Shared 依赖其他项目层。
- Content 依赖 App、Client 或 Game。
- Game 依赖 App 或 Client。

新增跨层引用前应先确认数据或事件的真实所有者，不应绕过该测试。

## 模块整理规则

- 没有实现的功能写入文档，不创建空的多层模块树。
- 已被新实现替代的代码应删除，不保留第二套未接入编译的版本。
- 公共类型放在其真实所有者模块中，通过有限重导出提供稳定入口。
- 数据定义、游戏规则和客户端表现必须保持单向依赖。

## 中文注释规范

- 模块文档说明职责、边界和当前实现状态。
- 公共类型和非直观算法使用中文文档注释。
- 注释解释原因、约束和数据流，不逐行复述代码。
- Rust、Bevy、ECS、JSON、ID 等专有名词保留原文。
- 未实现功能应明确写成“尚未接入”，不使用含糊的 TODO 占位。
