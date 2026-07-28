use crate::content::block::registry::BlockRegistry;
use crate::game::player::identity::Player;
use crate::game::save::config::{AutoSaveTimer, SaveConfig};
use crate::game::save::world::format::SavedChunk;
use crate::game::save::world::level;
use crate::game::save::world::queue::SaveQueue;
use crate::game::save::world::region::RegionManager;
use crate::game::world::generation::WorldGenerator;
use crate::game::world::state::WorldState;
use crate::game::world::time::WorldSimulationClock;
use bevy::prelude::*;

/// 区块卸载时自动保存系统
pub fn auto_save_on_unload_system(
    time: Res<Time>,
    mut auto_save_timer: ResMut<AutoSaveTimer>,
    save_config: Res<SaveConfig>,
    mut save_queue: ResMut<SaveQueue>,
    block_registry: Res<BlockRegistry>,
    world_generator: Res<WorldGenerator>,
    simulation_clock: Res<WorldSimulationClock>,
    player_query: Query<&Transform, With<Player>>,
    mut world_state: ResMut<WorldState>,
) {
    // 禁用自动保存则跳过
    if save_config.auto_save_interval <= 0.0 {
        return;
    }

    // 初始化计时器
    if auto_save_timer.timer.is_none() {
        auto_save_timer.timer = Some(Timer::from_seconds(
            save_config.auto_save_interval as f32,
            TimerMode::Repeating,
        ));
    }

    // 推进计时器
    let Some(ref mut timer) = auto_save_timer.timer else {
        return;
    };
    timer.tick(time.delta());

    if !timer.just_finished() {
        return;
    }

    // 获取玩家位置作为出生点
    let spawn_pos = player_query
        .single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    // 元数据很小，直接原子保存；真正修改过的区块交给后台保存队列。
    if let Err(error) = level::save_level(
        &save_config.world_name,
        world_generator.seed as u64,
        world_generator.generation_version,
        &simulation_clock,
        spawn_pos,
        &block_registry,
    ) {
        log::error!("[自动保存] 世界元数据保存失败: {error}");
        return;
    }

    for (position, modified_time) in world_state.take_modified_chunks() {
        let Some(data) = world_state.chunk(position) else {
            continue;
        };

        save_queue.enqueue(SavedChunk {
            position,
            data: data.as_ref().clone(),
            modified_time,
        });
    }
    log::trace!(
        "[自动保存] 元数据已保存，{} 个修改区块已进入后台队列",
        save_queue.queue.len()
    );
}

/// 保存整个世界
pub fn save_entire_world(
    world_name: &str,
    world_state: &WorldState,
    block_registry: &BlockRegistry,
    seed: u64,
    generation_version: u32,
    simulation_clock: &WorldSimulationClock,
    spawn_pos: Vec3,
) -> Result<(), super::region::SaveError> {
    // 保存世界数据到 level.dat
    level::save_level(
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

#[cfg(test)]
#[path = "../../../../tests/unit/game/save/world/write.rs"]
mod tests;
