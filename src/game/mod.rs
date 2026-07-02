//! # Game
//!
//! 游戏层（Game Layer）。
//!
//! `game` 是 CenturyJourney 的核心领域（Core Game Domain），
//! 负责实现所有游戏规则（Game Rules）与世界模拟（Simulation）。
//!
//! Game 是整个游戏行为的唯一权威来源（Single Source of Truth），
//! 所有游戏规则都应在此实现，而不是分散在 Client、Server 或其他模块。
//!
//! ## 职责
//!
//! Game 负责实现游戏世界中的所有业务逻辑，包括但不限于：
//!
//! - 世界模拟（World Simulation）
//! - 玩家逻辑（Player）
//! - 实体逻辑（Entity）
//! - 背包（Inventory）
//! - 方块交互（Interaction）
//! - 战斗（Combat）
//! - 合成（Crafting）
//! - 游戏规则（Game Rules）
//! - 世界 Tick
//! - Chunk Simulation
//! - AI
//!
//! Game 描述的是："游戏世界如何运行（How the Game Works）"。
//!
//! ## 设计原则
//!
//! Game 是整个游戏规则的唯一实现。
//!
//! 无论运行环境是：
//!
//! - 单机
//! - Client Prediction
//! - Dedicated Server
//! - Replay
//! - AI Simulation
//!
//! 都应调用同一套 Game 逻辑。
//!
//! Client 负责展示结果。
//!
//! Server 负责驱动 Game。
//!
//! Game 不关心结果最终如何显示，也不关心数据如何通过网络同步。
//!
//! ## 模块组织
//!
//! ```text
//! game/
//! ├── world/             // 世界模拟
//! ├── player/            // 玩家逻辑
//! ├── inventory/         // 背包系统
//! ├── interaction/       // 方块、物品交互
//! ├── combat/            // 战斗系统
//! ├── crafting/          // 合成系统
//! ├── entity/            // 实体系统
//! └── rules/             // 游戏规则
//! ```
//!
//! 每个 Feature 应保持统一结构：
//!
//! ```text
//! feature/
//! ├── plugin.rs
//! ├── components/
//! ├── resources/
//! ├── events/
//! ├── systems/
//! ├── commands/
//! ├── queries/
//! ├── config/
//! └── util/
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
//! Game 位于整个架构中心。
//!
//! 上层模块通过调用 Game 实现游戏行为；
//! 下层模块为 Game 提供运行所需的数据和基础设施。
//!
//! ## 核心设计理念
//!
//! Game 提供的是 **Game Rules（游戏规则）**，
//! 而不是 **Game Data（游戏数据）** 或 **Game Presentation（游戏表现）**。
//!
//! 例如：
//!
//! - Game 实现放置方块，而不是定义 Block。
//! - Game 实现挖掘逻辑，而不是定义 Pickaxe。
//! - Game 实现掉落规则，而不是定义 LootTable。
//! - Game 实现玩家移动，而不是处理键盘输入。
//! - Game 实现生命值变化，而不是绘制 HUD。
//!
//! ## 世界模拟原则
//!
//! Game 应以固定 Tick 推进世界模拟。
//!
//! 每个 Tick：
//!
//! - 更新玩家状态
//! - 更新实体状态
//! - 执行世界规则
//! - 处理事件
//! - 推进定时任务
//! - 更新游戏状态
//!
//! Game 负责"模拟世界"，
//! Client 与 Server 仅负责驱动模拟。
//!
//! ## 数据流
//!
//! Game 的数据来源于 Content。
//!
//! 例如：
//!
//! ```text
//! Content
//!      │
//!      ▼
//! BlockDefinition
//!      │
//!      ▼
//! Game
//!      │
//!      ▼
//! Block Placement
//!      │
//!      ▼
//! World State
//! ```
//!
//! Game 永远读取 Content，而不是修改 Content。
//!
//! ## 判断标准
//!
//! 当一个功能回答的是：
//!
//! "游戏规则是什么？"
//!
//! 那么它属于 Game。
//!
//! 如果回答的是：
//!
//! - 世界里有什么？→ Content
//! - 如何实现？→ Engine
//! - 如何显示？→ Client
//! - 如何通信？→ Protocol
//! - 如何组织？→ App
//! - 如何运行服务器？→ Server
//!
//! 那么它不应属于 Game。
//!
//! ## 核心原则
//!
//! **整个项目只能存在一份游戏规则。**
//!
//! Client 不实现游戏规则。
//!
//! Server 不实现游戏规则。
//!
//! Replay 不实现游戏规则。
//!
//! AI 不实现游戏规则。
//!
//! 它们全部共享同一套 Game，实现真正的单一权威逻辑（Single Source of Truth）。

pub mod advancement;
pub mod block;
pub mod combat;
pub mod command;
pub mod constant;
pub mod crafting;
pub mod effect;
pub mod entity;
pub mod event;
pub mod gameplay;
pub mod interaction;
pub mod inventory;
pub mod physics;
pub mod player;
pub mod rules;
pub mod statistics;
pub mod world;
