//! 管理世界元数据、区块数据及其持久化流程。

pub mod chunk;
pub mod metadata;
mod plugin;
pub mod runtime;

pub use chunk::load::{LoadQueue, try_load_chunk_from_disk};
pub use chunk::queue::{SaveQueue, SaveWorker, flush_save_queue};
pub(super) use plugin::WorldSavePlugin;
pub use runtime::world_load::CachedBlockIdRemap;
pub use runtime::world_load::load_entire_world;
pub use runtime::world_save::save_entire_world;
