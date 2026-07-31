//! 组织玩家与世界存档领域，并统一管理会话、脏状态和写入策略。

pub mod dirty;
pub mod player;
pub mod world;

mod config;
mod debug_controls;
mod path;
mod plugin;

pub use config::{AutoSaveTimer, SaveConfig};
pub use debug_controls::SaveDebugCommand;
pub use plugin::GameSavePlugin;
pub(in crate::game) use world::latest_snapshot_for_load;
pub use world::runtime::world_load::CachedBlockIdRemap;
pub use world::runtime::world_save::save_entire_world;
pub use world::{LoadQueue, SaveQueue, SaveWorker, flush_save_queue};
