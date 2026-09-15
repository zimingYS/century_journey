//! cj-sim —— 世界规律层
//!
//! 规则（§6.6.1）：
//! - 单 crate，内部按域分组（field 容器 + op 纯算子 + domain 领域）
//! - 不依赖 bevy / ECS / 渲染 / UI
//! - 不持有存储（存储归 cj-world）

pub mod climate;
pub mod eco;
pub mod event;
pub mod field;
pub mod geo;
pub mod hydro;
pub mod structure;
pub mod worldgen;
// 模拟算子签名约定：`fn(input, params, dt) -> output`
//
// 不读全局状态、不读 ECS、不依赖全局时间。
