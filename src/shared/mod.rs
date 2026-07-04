//! # Shared
//!
//! 共享层（Shared Layer）。
//!
//! `shared` 定义整个 CenturyJourney 共享的数据模型与基础概念，
//! 为 Client、Server、Game、Content 等所有上层模块提供统一的数据结构。
//!
//! Shared 不负责任何业务逻辑，它仅负责描述游戏中的"概念（Concept）"，
//! 而不是"行为（Behavior）"。
//!
//! ## 职责
//!
//! Shared 用于存放所有需要跨模块共享的数据定义，包括但不限于：
//!
//! - ECS Components
//! - ECS Resources
//! - ECS Events
//! - ECS Bundles
//! - 全局状态（State）
//! - 常量（Constants）
//! - 标识符（Identifier）
//! - 数学类型
//! - 坐标系统
//! - 方向定义
//! - 时间类型
//! - 通用工具
//!
//! Shared 提供统一的数据模型，使整个项目保持一致的数据表达方式。
//!
//! ## 设计原则
//!
//! Shared 描述的是"是什么（What）"，
//! 而不是"怎么做（How）"。
//!
//! Shared 中的数据应尽可能保持：
//!
//! - 简单
//! - 稳定
//! - 无副作用
//! - 可序列化
//! - 可复用
//!
//! Shared 应尽量避免依赖业务模块。
//!
//! ## 模块组织
//!
//! ```text
//! shared/
//! ├── components/      // ECS Components
//! ├── resources/       // ECS Resources
//! ├── events/          // ECS Events
//! ├── bundles/         // ECS Bundles
//! ├── states/          // 全局状态
//! ├── constants/       // 常量
//! ├── identifier/      // Identifier、UUID、ID 类型
//! ├── position/        // BlockPos、ChunkPos、WorldPos
//! ├── direction/       // Facing、Axis、Rotation
//! ├── math/            // 数学扩展
//! ├── time/            // Tick、GameTime
//! ├── registry/        // 通用注册接口
//! └── util/            // 无业务依赖的工具
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
//! Shared 位于 Engine 之上，Game、Content、Client、Server 之下，
//! 是整个项目共享数据模型的唯一来源。
//!
//! ## 核心设计理念
//!
//! Shared 提供的是 **Common Concepts（公共概念）**，
//! 而不是 **Gameplay（游戏玩法）**。
//!
//! 例如：
//!
//! - Shared 定义 `BlockPos`，而不是 `ChunkGenerator`。
//! - Shared 定义 `EntityId`，而不是 `MobAI`。
//! - Shared 定义 `Direction`，而不是 `PlayerLook`。
//! - Shared 定义 `GameTime`，而不是 `CropGrowthSystem`。
//! - Shared 定义 `InventorySlot`（数据结构），而不是 `InventoryManager`。
//!
//! ## 判断标准
//!
//! 当一个数据结构同时被多个上层模块使用，且不包含具体业务规则时，
//! 它应属于 Shared。
//!
//! 如果去掉 CenturyJourney 的游戏背景后，这个数据结构仍然具有通用意义，
//! 那么它通常应该放入 Shared；
//! 如果它依赖具体玩法规则，则应放入 Game 或其他业务模块。

pub mod bundles;
pub mod components;
pub mod constants;
pub mod direction;
pub mod errors;
pub mod events;
pub mod held_item;
pub mod identifier;
pub mod item_id;
pub mod math;
pub mod position;
pub mod registry;
pub mod resources;
pub mod states;
pub mod tag;
pub mod time;
pub mod ui_types;
pub mod util;
