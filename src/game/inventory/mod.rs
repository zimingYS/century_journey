//! 管理玩家与世界容器的权威库存状态、交互命令和运行时规则。

mod plugin;
mod runtime;

pub mod container;
pub mod events;
pub mod interaction;
pub mod item;
pub mod slot;
pub mod state;

pub use plugin::InventoryPlugin;
pub(crate) use plugin::InventorySet;
pub use state::{InventoryState, LocalInventory, LocalInventoryMut};
