//! # Server
//!
//! 服务器层（Server Layer）。
//!
//! `server` 负责驱动 CenturyJourney 的游戏模拟（Simulation）并维护权威世界状态
//! （Authoritative World State），为多人游戏提供网络、同步、存档和管理能力。
//!
//! Server 不负责实现游戏规则，它负责组织、调度并运行 Game。
//!
//! ## 职责
//!
//! Server 负责所有仅存在于服务器端的功能，包括但不限于：
//!
//! - 网络连接管理
//! - 玩家会话管理
//! - Packet 收发
//! - 权限校验
//! - 世界生命周期管理
//! - Tick 驱动
//! - 世界存档
//! - Chunk 加载与卸载
//! - 玩家同步
//! - 状态复制（Replication）
//! - 控制台命令
//! - Dedicated Server
//! - 插件加载
//! - 服务器配置
//!
//! Server 描述的是："游戏如何运行（How the Game Runs）"。
//!
//! ## 设计原则
//!
//! Server 是游戏世界的唯一权威（Authority）。
//!
//! 所有客户端请求都必须经过 Server 验证，
//! 再调用 Game 修改世界状态。
//!
//! Server 不直接实现游戏规则，
//! 而是驱动 Game 推进世界模拟。
//!
//! ## 模块组织
//!
//! ```text
//! server/
//! ├── network/          // 网络服务
//! ├── session/          // 玩家连接与会话
//! ├── replication/      // 状态同步
//! ├── world/            // 世界管理
//! ├── save/             // 世界存档
//! ├── command/          // 控制台命令
//! ├── permission/       // 权限系统
//! ├── config/           // 服务器配置
//! ├── plugin/           // 服务端插件
//! ├── console/          // 控制台
//! └── dedicated/        // Dedicated Server
//! ```
//!
//! ## 架构位置
//!
//! ```text
//!                App
//!              /     \
//!         Client     Server
//!             │         │
//!             └────┬────┘
//!                  │
//!              Protocol
//!                  │
//!                Game
//!               /    \
//!         Content   Shared
//!               \    /
//!               Engine
//! ```
//!
//! Server 位于架构最上层之一，
//! 负责驱动 Game，并通过 Protocol 与 Client 通信。
//!
//! Server 可以依赖：
//!
//! - Engine
//! - Shared
//! - Content
//! - Game
//! - Protocol
//!
//! Game 永远不能依赖 Server。
//!
//! ## 核心设计理念
//!
//! Server 提供的是 **Authority（权威）**，
//! 而不是 **Gameplay（游戏规则）**。
//!
//! 例如：
//!
//! - Server 验证放置请求，而不是实现放置逻辑。
//! - Server 驱动世界 Tick，而不是实现 Tick 行为。
//! - Server 同步实体状态，而不是计算实体 AI。
//! - Server 保存世界，而不是定义世界。
//!
//! ## 数据流
//!
//! ```text
//! Client
//!      │
//!      ▼
//! Packet
//!      │
//!      ▼
//! Server
//!      │
//!      ▼
//! Validate
//!      │
//!      ▼
//! Game
//!      │
//!      ▼
//! World State Changed
//!      │
//!      ▼
//! Replication
//!      │
//!      ▼
//! Client
//! ```
//!
//! Server 是世界状态的唯一修改入口。
//!
//! 所有世界变化最终都来源于 Game。
//!
//! ## 世界模拟
//!
//! Server 负责以固定 Tick 驱动世界模拟。
//!
//! 每个 Tick：
//!
//! - 接收客户端输入
//! - 校验请求
//! - 调用 Game
//! - 推进世界模拟
//! - 执行存档任务
//! - 执行同步任务
//!
//! Server 本身不关心具体规则，只负责调度。
//!
//! ## 状态同步
//!
//! Server 不同步整个世界。
//!
//! 而是根据世界状态变化，通过 Replication 系统向客户端发送：
//!
//! - Chunk 更新
//! - Entity 更新
//! - Inventory 更新
//! - World Event
//! - Game Event
//!
//! Server 负责"同步发生了什么"，
//! Client 负责"如何表现这些变化"。
//!
//! ## 判断标准
//!
//! 当一个功能回答的是：
//!
//! "如何管理服务器运行？"
//!
//! 那么它属于 Server。
//!
//! 如果回答的是：
//!
//! - 游戏规则是什么？→ Game
//! - 如何显示？→ Client
//! - 数据如何通信？→ Protocol
//! - 世界里有什么？→ Content
//! - 如何实现基础能力？→ Engine
//!
//! 那么它不属于 Server。
//!
//! ## 核心原则
//!
//! Server 永远不拥有独立的游戏规则。
//!
//! 它负责：
//!
//! - 驱动
//! - 校验
//! - 同步
//! - 存档
//! - 管理
//!
//! 游戏规则始终由 Game 实现，
//! Server 的职责是保证所有玩家共享同一个真实、可靠且一致的世界状态。

pub mod network;
pub mod simulation;
pub mod authority;
pub mod save;
pub mod chunk;
pub mod player;
pub mod command;
pub mod console;
pub mod plugin;
pub mod permission;
pub mod anti_cheat;
pub mod session;
pub mod replication;
pub mod scheduler;
pub mod metrics;
pub mod config;
pub mod whitelist;
pub mod ban;
pub mod logging;
pub mod watchdog;