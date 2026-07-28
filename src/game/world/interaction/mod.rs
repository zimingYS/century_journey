//! 管理玩家与世界方块、掉落物之间的权威交互规则。

mod breaking;
mod pickup;
mod plugin;

pub use breaking::execute_block_break;

pub(in crate::game::world) use plugin::WorldInteractionPlugin;
