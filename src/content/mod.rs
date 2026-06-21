//! # Content
//!
//! 内容层（Content Layer）。
//!
//! `content` 定义 CenturyJourney 世界中的所有游戏内容（Game Content），
//! 包括方块、物品、生物群系、配方等数据资源。
//!
//! Content 负责描述"世界里有什么（What Exists）"，
//! 而不是"这些内容如何运行（How It Behaves）"。
//!
//! Content 是整个项目的数据定义中心，也是数据驱动架构（Data-Driven Architecture）的核心。
//!
//! ## 职责
//!
//! Content 负责管理所有游戏内容定义，包括但不限于：
//!
//! - Block
//! - Item
//! - Biome
//! - Structure
//! - Recipe
//! - Loot Table
//! - Entity Definition
//! - Particle
//! - Sound
//! - Language
//! - Tag
//! - Registry
//!
//! 每一种内容都应包含：
//!
//! - Definition（定义）
//! - Registry（注册）
//! - Loader（加载）
//! - Tag（标签）
//! - Model（模型）
//! - Texture（纹理）
//!
//! Content 是游戏世界的数据来源，而不是行为来源。
//!
//! ## 设计原则
//!
//! Content 只描述数据，不描述行为。
//!
//! 所有内容应尽可能采用数据驱动方式进行定义，
//! 避免将游戏数据硬编码到业务逻辑中。
//!
//! 游戏规则应读取 Content 提供的数据，而不是反过来依赖业务模块。
//!
//! ## 模块组织
//!
//! ```text
//! content/
//! ├── block/
//! │   ├── definition/
//! │   ├── registry/
//! │   ├── loader/
//! │   ├── model/
//! │   ├── texture/
//! │   ├── tag/
//! │   └── behavior/
//! │
//! ├── item/
//! │   ├── definition/
//! │   ├── registry/
//! │   ├── loader/
//! │   ├── model/
//! │   ├── texture/
//! │   └── tag/
//! │
//! ├── biome/
//! ├── structure/
//! ├── recipe/
//! ├── loot/
//! ├── entity/
//! ├── particle/
//! ├── sound/
//! ├── language/
//! └── localization/
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
//! Content 位于 Shared 之上、Game 之下，
//! 为整个游戏提供统一的内容定义。
//!
//! Game、Client、Server 均通过 Registry 查询和使用 Content，
//! 而不是直接维护游戏内容。
//!
//! ## 核心设计理念
//!
//! Content 提供的是 **Game Definitions（游戏内容定义）**，
//! 而不是 **Game Logic（游戏规则）**。
//!
//! 例如：
//!
//! - Content 定义 Block，而不是 Block Update。
//! - Content 定义 Item，而不是 Item Use。
//! - Content 定义 Recipe，而不是 Crafting。
//! - Content 定义 LootTable，而不是 Drop Logic。
//! - Content 定义 Entity，而不是 AI。
//!
//! ## 数据驱动原则
//!
//! 游戏中的大部分内容应由外部资源定义，例如：
//!
//! - JSON
//! - RON
//! - TOML
//! - YAML
//! - 二进制资源
//!
//! 程序启动时由 Loader 加载并注册到 Registry，
//! 游戏逻辑始终通过 Registry 查询内容定义，而不是直接依赖具体实现。
//!
//! ## 判断标准
//!
//! 如果一个对象是在描述"游戏世界中存在什么"，
//! 它通常属于 Content。
//!
//! 如果一个对象是在描述"这些内容如何运行"，
//! 它应属于 Game。
//!
//! 简单来说：
//!
//! - Content 决定世界里"有什么"。
//! - Game 决定世界里的东西"怎么工作"。
//! - Client 决定它们"如何表现"。
//! - Server 决定它们"如何同步"。

pub mod block;
pub mod item;
pub mod biome;
pub mod structure;
pub mod recipe;
pub mod loot;
pub mod entity;
pub mod particle;
pub mod sound;
pub mod language;
pub mod localization;