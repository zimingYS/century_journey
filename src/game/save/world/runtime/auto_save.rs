//! 根据定时器和脏状态触发世界自动保存。

use crate::content::block::registry::BlockRegistry;
use crate::game::player::identity::Player;
use crate::game::save::world::chunk::model::SavedChunk;
use crate::game::save::world::metadata::io;
use crate::game::save::{AutoSaveTimer, SaveConfig, SaveQueue};
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::state::WorldState;
use crate::game::world::time::WorldSimulationClock;
use bevy::math::Vec3;
use bevy::prelude::{Query, Res, ResMut, Time, Timer, TimerMode, Transform, With};

/// 区块卸载时自动保存系统
/// 自动保存同时协调时钟、队列和脏区块，资源借用保持显式便于审查写入顺序。
#[allow(clippy::too_many_arguments)]
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
    if let Err(error) = io::save_level(
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
        let Some(snapshot) = world_state.chunk_snapshot(position) else {
            continue;
        };
        save_queue.enqueue(SavedChunk::from_world_snapshot(
            position,
            snapshot,
            modified_time,
        ));
    }
    log::trace!(
        "[自动保存] 元数据已保存，{} 个修改区块已进入后台队列",
        save_queue.queue.len()
    );
}
