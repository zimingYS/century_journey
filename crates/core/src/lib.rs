//! # cj-core —— 项目公共语言
//!
//! 全项目共用的纯类型与数学层。
//!
//! ## 硬纪律：零引擎依赖
//! 不依赖任何游戏引擎（bevy / wgpu 等）。
//! 验证：`cargo tree -p cj-core` 里不应出现 bevy。
//!
//! ## 模块导航
//!
//! | 模块 | 内容 |
//! |---|---|
//! | [`voxel`]  | 方块身份与体素内容 |
//! | [`pos`]    | 四类坐标与换算 |
//! | [`spec`]   | 世界规格常量 |
//! | [`time`]   | 世界时间与模拟时钟 |
//! | [`field_ref`] | 可观测场引用 |
//! | [`rng`]    | 确定性位置哈希 |
//! | [`math`]   | 基础数学工具 |

pub mod block;
pub mod field_ref;
pub mod math;
pub mod pos;
pub mod rng;
pub mod spec;
pub mod time;
pub mod voxel;
