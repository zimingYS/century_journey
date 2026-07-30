//! 组织合成会话、交互和物品转移等运行时规则。

pub mod interaction;
pub mod lifecycle;
mod station;
pub mod transfer;

pub(super) use interaction::crafting_interaction_system;
pub(super) use lifecycle::return_crafting_on_close_system;
pub(super) use station::open_workbench_system;
