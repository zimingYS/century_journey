//! 客户端声音反馈。
//!
//! 该模块只消费游戏消息和只读状态；声音播放不会改变世界、物品栏或动画状态。

mod plugin;
mod resources;
mod systems;

pub use plugin::ClientSoundPlugin;
