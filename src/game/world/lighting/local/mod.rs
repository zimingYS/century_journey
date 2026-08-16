//! 为可见区块和方块编辑区域提供高优先级局部光照重建。

mod channel;
mod constants;
mod helpers;
mod systems;

pub(super) use channel::register_resources;
pub(super) use systems::{
    clear_local_lighting, prune_unloaded_lighting, queue_pending_chunk_lighting,
    receive_local_lighting_results, schedule_local_lighting_rebuild, sync_changed_block_sources,
};

#[cfg(test)]
#[path = "../../../../../tests/unit/game/world/lighting/local/mod.rs"]
mod tests;
