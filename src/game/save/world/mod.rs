//! 管理世界元数据、区块数据及其持久化流程。

pub mod format;
pub mod level;
pub mod region;

pub(super) mod load;
mod plugin;
pub(super) mod queue;
pub(super) mod write;

pub use load::{
    CachedBlockIdRemap, LoadQueue, load_entire_world, load_world_metadata, try_load_chunk_from_disk,
};
pub(super) use plugin::WorldSavePlugin;
pub use queue::{SaveQueue, SaveWorker, flush_save_queue};
pub use write::save_entire_world;
