//! 跨层共享数据定义。
//!
//! Shared 只保存多个上层模块共同使用且不包含业务副作用的稳定协议类型。

pub mod identifier;
pub mod item_id;
pub mod random;
pub mod states;
pub mod tag;
pub mod voxel;
