use crate::content::block::registry::BlockRegistry;
use crate::game::save::world::chunk::model::SavedChunk;
use crate::game::save::world::chunk::region::{RegionManager, SaveError};
use crate::game::save::world::metadata::io;
use crate::game::world::state::authoritative::WorldState;
use crate::game::world::time::WorldSimulationClock;
use bevy::math::Vec3;
use bevy::prelude;

/// 保存整个世界
pub fn save_entire_world(
    world_name: &str,
    world_state: &WorldState,
    block_registry: &BlockRegistry,
    seed: u64,
    generation_version: u32,
    simulation_clock: &WorldSimulationClock,
    spawn_pos: Vec3,
) -> prelude::Result<(), SaveError> {
    // 保存世界数据到 level.dat
    io::save_level(
        world_name,
        seed,
        generation_version,
        simulation_clock,
        spawn_pos,
        block_registry,
    )?;

    // 获取当前时间戳
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    // 批量保存所有区块
    let chunks: Vec<SavedChunk> = world_state
        .chunks()
        .map(|(position, data)| SavedChunk {
            position,
            data: data.as_ref().clone(),
            modified_time: world_state.chunk_modified_time(position).unwrap_or(now),
        })
        .collect();

    RegionManager::write_chunks_batch(world_name, &chunks)?;

    log::info!("[存档系统] 世界已保存: {} 个区块", chunks.len());
    Ok(())
}
