//! # Client
//!
//! 客户端层（Client Layer）。
//!
//! `client` 负责 CenturyJourney 的所有本地表现（Presentation）与交互（Interaction），
//! 将 Game 产生的世界状态转换为玩家可见、可听、可操作的游戏体验。
//!
//! Client 不负责定义游戏规则，它只负责表现游戏规则。
//!
//! ## 职责
//!
//! Client 负责所有仅存在于客户端的功能，包括但不限于：
//!
//! - 世界渲染（Renderer）
//! - 摄像机（Camera）
//! - 玩家第一人称模型（ViewModel）
//! - UI
//! - HUD
//! - 动画
//! - 粒子
//! - 音效
//! - 天空系统
//! - 后处理效果
//! - 本地输入采集
//! - 本地预测（Client Prediction）
//! - 插值（Interpolation）
//! - 调试可视化
//!
//! Client 描述的是："游戏如何呈现给玩家（How the Game Is Presented）"。
//!
//! ## 设计原则
//!
//! Client 是世界状态的消费者（Consumer），
//! 而不是世界状态的生产者（Producer）。
//!
//! Client 应根据 Game 当前状态进行表现，
//! 而不是主动维护另一套游戏状态。
//!
//! 当需要执行游戏行为时：
//!
//! - 单机模式：调用 Game。
//! - 联机模式：发送请求，由 Server 驱动 Game。
//!
//! Client 自身不决定游戏规则是否成立。
//!
//! ## 模块组织
//!
//! ```text
//! client/
//! ├── renderer/         // 世界渲染
//! ├── camera/           // 摄像机
//! ├── viewmodel/        // 第一人称模型
//! ├── animation/        // 动画
//! ├── ui/               // UI
//! ├── hud/              // HUD
//! ├── sky/              // 天空系统
//! ├── effect/           // 特效
//! ├── audio/            // 客户端音频
//! ├── input/            // 输入采集
//! ├── prediction/       // 客户端预测
//! ├── interpolation/    // 插值
//! └── debug/            // 调试工具
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
//! Client 位于架构最上层之一，
//! 负责将 Game 当前状态转换为玩家可见的表现。
//!
//! Client 可以依赖：
//!
//! - Engine
//! - Shared
//! - Content
//! - Game
//! - Protocol
//!
//! 但 Game 永远不能依赖 Client。
//!
//! ## 核心设计理念
//!
//! Client 提供的是 **Presentation（表现）**，
//! 而不是 **Simulation（模拟）**。
//!
//! 例如：
//!
//! - Client 渲染方块，而不是生成方块。
//! - Client 播放攻击动画，而不是计算伤害。
//! - Client 显示生命值，而不是修改生命值。
//! - Client 打开背包界面，而不是管理背包数据。
//! - Client 播放音效，而不是决定何时掉落物品。
//!
//! ## 数据流
//!
//! ```text
//! Content
//!      │
//!      ▼
//! Game
//!      │
//!      ▼
//! World State
//!      │
//!      ▼
//! Client
//!      │
//!      ▼
//! Renderer / UI / Audio / Animation
//! ```
//!
//! Client 始终读取 Game 当前状态，
//! 根据状态决定如何渲染和交互。
//!
//! ## 客户端预测
//!
//! 为提升交互体验，
//! Client 可以实现：
//!
//! - Client Prediction
//! - Entity Interpolation
//! - Entity Extrapolation
//! - Input Buffer
//!
//! 这些仅影响本地表现，
//! 不改变游戏规则的最终结果。
//!
//! 最终世界状态始终以 Game（单机）或 Server（联机）为准。
//!
//! ## 判断标准
//!
//! 当一个功能回答的是：
//!
//! "玩家应该看到什么？"
//!
//! 那么它属于 Client。
//!
//! 如果回答的是：
//!
//! - 世界里有什么？→ Content
//! - 游戏规则是什么？→ Game
//! - 如何实现？→ Engine
//! - 如何通信？→ Protocol
//! - 如何运行服务器？→ Server
//!
//! 那么它不属于 Client。
//!
//! ## 核心原则
//!
//! Client 永远不拥有游戏规则。
//!
//! 它只负责：
//!
//! - 输入
//! - 展示
//! - 动画
//! - 渲染
//! - 音频
//! - 本地体验优化
//!
//! 游戏规则始终来自 Game，
//! Client 的职责是将这些规则以自然、流畅、直观的方式呈现给玩家。

pub mod network;
pub mod renderer;
pub mod viewmodel;
pub mod camera;
pub mod animation;
pub mod ui;
pub mod hud;
pub mod sky;
pub mod particle;
pub mod effect;
pub mod sound;
pub mod debug;
pub mod input;
pub mod localization;
pub mod asset;
pub mod font;
pub mod postprocess;
pub mod overlay;
