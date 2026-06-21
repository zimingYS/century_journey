//! # Engine
//!
//! 引擎层（Engine Layer）。
//!
//! `engine` 为整个 CenturyJourney 提供与具体游戏无关的底层基础能力，
//! 它不包含任何游戏规则，也不了解任何游戏内容，仅负责提供通用运行环境。
//!
//! Engine 是整个项目架构的最底层，所有上层模块（Shared、Content、Game、
//! Client、Server）均可依赖 Engine，但 Engine 本身不能依赖任何业务模块。
//!
//! ## 职责
//!
//! Engine 负责提供所有通用基础设施，包括但不限于：
//!
//! - ECS 扩展
//! - 渲染基础设施
//! - Mesh 构建工具
//! - Texture 管理
//! - Material 管理
//! - Asset 加载
//! - 异步任务调度
//! - 多线程支持
//! - 文件系统
//! - 序列化
//! - 存档读写
//! - 数学工具
//! - 输入封装
//! - 音频基础
//! - 时间管理
//! - 随机数
//! - 调试工具
//! - 性能分析
//! - 通用工具函数
//!
//! Engine 负责的是"能力（Capability）"，而不是"玩法（Gameplay）"。
//!
//! ## 设计原则
//!
//! Engine 只提供通用能力，不参与任何业务逻辑。
//!
//! 上层模块通过组合 Engine 提供的能力构建游戏，而不是修改 Engine 来实现玩法。
//!
//! Engine 应尽可能保持平台无关、玩法无关、业务无关。
//!
//! ## 模块组织
//!
//! ```text
//! engine/
//! ├── asset/             // 资源加载与管理
//! ├── ecs/               // ECS 扩展
//! ├── render/            // 渲染基础设施
//! ├── graphics/          // GPU 抽象
//! ├── mesh/              // Mesh 工具
//! ├── texture/           // Texture 管理
//! ├── material/          // Material 管理
//! ├── physics/           // 通用物理工具
//! ├── math/              // 数学工具
//! ├── input/             // 输入封装
//! ├── audio/             // 音频基础
//! ├── task/              // 异步任务系统
//! ├── threading/         // 多线程工具
//! ├── filesystem/        // 文件系统
//! ├── serialization/     // 序列化
//! ├── save/              // 通用存档接口
//! ├── diagnostics/       // 调试与日志
//! ├── profiling/         // 性能分析
//! ├── random/            // 随机数工具
//! └── util/              // 通用工具
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
//! Engine 位于整个架构最底层，是所有模块共同依赖的基础设施。
//! 它应始终保持稳定、通用、可复用，并避免引入任何游戏领域知识。
//!
//! ## 核心设计理念
//!
//! Engine 提供的是 **How（如何实现）**，
//! 而不是 **What（实现什么）**。
//!
//! 例如：
//!
//! - Engine 提供 MeshBuilder，而不是 BlockMesh。
//! - Engine 提供 AssetLoader，而不是 BlockRegistry。
//! - Engine 提供 TaskPool，而不是 ChunkGenerator。
//! - Engine 提供 SaveSystem，而不是 WorldSave。
//!
//! 当一个功能能够脱离 CenturyJourney，直接应用于其他游戏时，
//! 它应属于 Engine；否则，应放入更高层的模块（Shared、Content、Game、Client 或 Server）。

pub mod asset;
pub mod ecs;
pub mod render;
pub mod graphics;
pub mod mesh;
pub mod texture;
pub mod material;
pub mod physics;
pub mod math;
pub mod input;
pub mod audio;
pub mod task;
pub mod threading;
pub mod save;
pub mod serialization;
pub mod filesystem;
pub mod diagnostics;
pub mod profiling;
pub mod random;
pub mod util;
pub mod constant;