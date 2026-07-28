//! 管理玩家周围区块的加载窗口、优先级和生命周期。

mod config;
mod lifecycle;
mod plugin;

pub use config::{PlayerChunkCache, WorldStreamingConfig};

pub(in crate::game::world) use lifecycle::manage_chunks_system;
pub(in crate::game::world) use plugin::WorldStreamingPlugin;
