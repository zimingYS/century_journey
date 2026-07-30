# 项目结构

本文档描述当前仓库中真实存在并参与编译的模块，不把规划功能写成已实现能力。

## 运行链路

    main
      -> app::launch
      -> ClientApplication
      -> app::runtime::ClientRuntimePluginGroup
         -> EnginePluginGroup
         -> ContentPluginGroup
         -> GamePluginGroup
         -> AppPluginGroup
         -> ClientPluginGroup

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
    ├── engine/    资产、持久化和异步任务等通用基础设施
    ├── game/      世界、玩家、物品栏、合成与玩法规则
    ├── shared/    稳定标识、顶层状态、输入上下文和体素尺寸等窄契约
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

世界时间的权威时钟、日历和事件归 `Game::world::time` 所有；天空与光照对时间
的可视化归 `Client::presentation::time` 和 `Client::sky` 所有。相机组件和玩家
骨架只属于 Client，Shared 不保存带明确运行时所有者的表现类型。

存档是独立的 Game 领域：`GameSavePlugin` 组装 `PlayerSavePlugin` 与
`WorldSavePlugin`。世界模块不再拥有存档插件，只通过窄数据接口向存档领域提供
权威区块和时钟快照。

## 自动边界检查

`tests/architecture_boundaries.rs` 会递归扫描 Rust 源码并拒绝以下问题：

- Engine 依赖 App、Client、Content 或 Game。
- Shared 依赖其他项目层。
- Content 依赖 App、Client 或 Game。
- Game 依赖 App 或 Client。
- 生产源码文件超过 800 行。
- 源码模块缺少文件级 `//!` 职责说明。
- 公共类型或函数缺少中文 `///` 职责说明。
- 使用 crate/模块级 lint 豁免，或局部豁免缺少中文原因说明。
- Game 权威规则使用未标明语义的 `Time`，而不是显式的 `Time<Fixed>`。
- 已删除的物品渲染、库存、玩家或导航兼容接口被重新引入。
- 白盒测试没有镜像到 `tests/unit/`、出现孤立测试文件或被重复声明。

新增跨层引用前应先确认数据或事件的真实所有者，不应绕过该测试。

## 模块整理规则

- 没有实现的功能写入文档，不创建空的多层模块树。
- 已被新实现替代的代码应删除，不保留第二套未接入编译的版本。
- 公共类型放在其真实所有者模块中，通过有限重导出提供稳定入口。
- 数据定义、游戏规则和客户端表现必须保持单向依赖。
- 每个有多个子领域的层级使用自己的聚合插件；App 运行时只组装各层聚合入口。
- 游戏规则读取 Client 输入转换后的命令或消息，不直接读取键盘、鼠标或界面资源。

## 中文注释规范

- 模块文档说明职责、边界和当前实现状态。
- 公共类型和非直观算法使用中文文档注释。
- 注释解释原因、约束和数据流，不逐行复述代码。
- Rust、Bevy、ECS、JSON、ID 等专有名词保留原文。
- 未实现功能应明确写成“尚未接入”，不使用含糊的 TODO 占位。

## 植被与树形边界

`Content::vegetation` 拥有树种、生长节奏和树形尺寸定义，并把稳定方块标识解析为运行时
ID。`Game::world::structure` 只生成无副作用的确定性体素蓝图，供世界生成和运行时玩法共同
消费。`Game::world::vegetation` 在固定步的 Environment 阶段维护已加载区块内的稀疏树苗
索引，完成支撑、空间与区块生命周期预检后，通过统一方块写入接口提交结果。

运行时植被不得依赖 Client，也不得使用生成期 `pending_writes`。跨入未加载区块或遇到非空气
方块时，本次生长整体延后，不能覆盖玩家建筑或留下残缺结构。
