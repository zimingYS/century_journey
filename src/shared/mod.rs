//! 跨层共享数据定义。
//!
//! Shared 保存多个上层模块共同使用且不包含业务副作用的类型，包括组件、
//! 标识符、物品 ID、应用状态、标签、时间和 UI 槽位类型。

pub mod components;
pub mod held_item;
pub mod identifier;
pub mod item_id;
pub mod states;
pub mod tag;
pub mod time;
pub mod ui_types;
