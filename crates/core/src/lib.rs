//! # cj-core —— 项目公共语言
//!
//! 全项目共用的**纯类型与数学层**，被所有上层 crate 依赖。
//!
//! ## 硬纪律：零引擎依赖
//! 本 crate 不依赖任何游戏引擎（bevy / wgpu 等）。**这是不可破的纪律。**&#8203;
//! 一旦破了，`cj-sim` 的「可独立测试 / 可 CLI 批处理 / 可 GPU 平移」三项收益全部作废。
//! 验证：`cargo tree -p cj-core` 里不应出现 bevy。
//!
//! ## 放什么
//! 判断标准（**两条都满足**才放）：
//! 1. 多个层都要用
//! 2. 不依赖任何具体实现
//!    只满足一条的，放错了层。
//!    **不放**：ECS Component · Bevy Resource · 渲染代码 · 模拟算法 · 存档逻辑 · 游戏 UI
//!
//! ## 模块导航
//!
//! | 模块 | 内容 |
//! |---|---|
//! | [`voxel`]  | 方块身份与体素内容（`BlockId` / `ShapeId` / `Voxel`） |
//! | [`pos`]    | 四类坐标与换算（`BlockPos` / `ChunkPos` / `SectionPos` / `SectionLocalPos`） |
//! | [`spec`]   | 世界规格常量（尺寸、高度边界、海平面） |
//! | [`time`]   | 世界时间与模拟时钟 |
//! | [`field_ref`] | 可观测场引用（采样接口的输入） |
//! | [`rng`]    | 确定性位置哈希 |
//! | [`math`]   | 基础数学工具 |

pub mod field_ref;
pub mod math;
pub mod pos;
pub mod rng;
pub mod spec;
pub mod time;
pub mod voxel;
