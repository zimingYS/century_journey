//! 组织库存命令、快捷栏选择和槽位交互等固定步系统。

mod commands;
mod hotbar;
mod slot_interaction;

pub(super) use commands::handle_inventory_command_system;
pub(super) use hotbar::handle_hotbar_command_system;
pub(super) use slot_interaction::handle_slot_interaction_system;
