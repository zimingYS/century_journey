//! 组织本地玩家模型、第一人称全身表现和手持物品视图。

pub mod full_body;
pub mod model;

mod plugin;
mod systems;

pub use plugin::ClientPlayerPlugin;
