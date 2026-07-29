//! # Crafting
//!
//! 合成逻辑。
//!
//! 实现配方匹配及物品制作规则。

pub mod events;
pub mod grid;

mod plugin;
mod runtime;

pub use events::CraftingStationOpened;
pub use plugin::CraftingPlugin;
