//! # App
//!
//! 应用层（Application Layer）。
//!
//! `app` 是整个 CenturyJourney 的最高层模块，负责组织和启动整个游戏，
//! 它不包含任何具体的游戏逻辑，仅负责协调各个子系统之间的关系。
//!
//! ## 职责
//!
//! - 创建并初始化 `bevy::App`
//! - 注册所有 Plugin
//! - 配置 ECS Schedule
//! - 初始化窗口、渲染与运行环境
//! - 初始化全局配置
//! - 管理应用生命周期
//! - 管理全局游戏状态（State）
//! - 定义程序启动流程
//!
//! ## 设计原则
//!
//! App 是整个项目的组合层（Composition Root）。
//!
//! 所有模块均通过 Plugin 注册到 App，
//! 模块之间不应直接依赖 App，App 负责依赖组织而非业务实现。
//!
//! ## 目录结构
//!
//! ```text
//! app/
//! ├── mod.rs         // 模块导出
//! ├── plugin.rs      // 注册所有一级 Plugin
//! ├── startup.rs     // 启动流程
//! ├── schedule.rs    // ECS 调度配置
//! ├── state.rs       // 应用状态
//! ├── config.rs      // 全局配置
//! └── exit.rs        // 程序退出流程
//! ```
//!
//! ## 架构位置
//!
//! ```text
//!                 App
//!               /     \
//!          Client     Server
//!              │         │
//!              └────┬────┘
//!                   │
//!               Protocol
//!                   │
//!                 Game
//!                /    \
//!          Content   Shared
//!                \    /
//!                Engine
//! ```
//!
//! App 位于整个架构最顶层，仅负责组织各层，不参与任何底层实现。

pub mod plugin;
pub mod startup;
pub mod schedule;
pub mod state;
pub mod config;
pub mod exit;
pub mod launcher;
pub mod mode;
pub mod application;
pub mod input_block;

pub use launcher::launch;