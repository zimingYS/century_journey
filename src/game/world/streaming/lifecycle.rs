//! 根据玩家权威位置和朝向装载、排序并卸载区块实体。

use super::WorldStreamingConfig;
use crate::game::player::identity::Player;
use crate::game::save::world::chunk::model::SavedChunk;
use crate::game::save::{SaveConfig, SaveQueue};
use crate::game::world::chunk::{ChunkComponents, ChunkState};
use crate::game::world::state::ChunkRuntime;
use crate::game::world::state::WorldState;
use crate::game::world::streaming::cache::PlayerChunkCache;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Commands, Entity, EntityWorldMut, Query, Res, ResMut, Transform, With};

/// 单帧最多创建的区块实体数，避免流送窗口变化造成帧尖峰。
const MAX_SPAWN_PER_FRAME: u32 = 4;
/// 单帧最多卸载的区块实体数。
const MAX_UNLOAD_PER_FRAME: usize = 8;

/// 根据有序加载窗口创建或卸载区块实体，并遵守每帧处理预算。
/// 流送系统同时协调实体、权威区块和保存队列，资源借用保持显式。
#[allow(clippy::too_many_arguments)]
pub fn manage_chunks_system(
    mut commands: Commands,
    mut save_queue: ResMut<SaveQueue>,
    mut player_cache: ResMut<PlayerChunkCache>,
    mut chunk_runtime: ResMut<ChunkRuntime>,
    mut world_state: ResMut<WorldState>,
    chunk_query: Query<(Entity, &ChunkComponents)>,
    player_query: Query<&Transform, With<Player>>,
    save_config: Res<SaveConfig>,
    streaming_config: Res<WorldStreamingConfig>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_chunk_pos = WorldStreamingConfig::chunk_from_world(player_transform.translation);
    let view_forward_xz = view_forward_xz(player_transform);

    if player_cache.needs_rebuild(player_chunk_pos, &streaming_config) {
        player_cache.rebuild(&streaming_config, player_chunk_pos, view_forward_xz);
    }

    let mut spawned = 0u32;
    for &chunk_pos in player_cache.ordered_chunks() {
        if spawned >= MAX_SPAWN_PER_FRAME {
            break;
        }
        if chunk_runtime.contains_chunk_entity(chunk_pos) {
            continue;
        }

        let entity = commands
            .spawn((
                ChunkComponents {
                    position: chunk_pos,
                },
                ChunkState::Empty,
                Transform::from_translation(Vec3::new(
                    (chunk_pos.x * CHUNK_SIZE as i32) as f32,
                    (chunk_pos.y * CHUNK_SIZE as i32) as f32,
                    (chunk_pos.z * CHUNK_SIZE as i32) as f32,
                )),
            ))
            .id();
        chunk_runtime.register_chunk_entity(chunk_pos, entity);
        spawned += 1;
    }

    let mut unloaded = 0usize;
    for (entity, chunk_components) in chunk_query.iter() {
        if unloaded >= MAX_UNLOAD_PER_FRAME {
            break;
        }
        let pos = chunk_components.position;
        if player_cache.expects_chunk(pos) {
            continue;
        }

        let unloaded_snapshot = world_state.remove_chunk(pos);

        if save_config.save_on_unload
            && let Some(snapshot) = unloaded_snapshot
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            save_queue.enqueue(SavedChunk::from_world_snapshot(
                pos,
                snapshot,
                world_state.chunk_modified_time(pos).unwrap_or(now),
            ));
        }

        chunk_runtime.remove_generation_context(pos);
        chunk_runtime.remove_chunk_entity(pos);
        world_state.clear_chunk_modified(pos);
        commands
            .entity(entity)
            .queue_silenced(|entity: EntityWorldMut| {
                entity.despawn();
            });
        unloaded += 1;
    }
}

/// 返回玩家权威偏航在水平面的单位前向量。
fn view_forward_xz(player_transform: &Transform) -> Vec2 {
    let forward = player_transform.rotation * Vec3::NEG_Z;
    Vec2::new(forward.x, forward.z).normalize_or_zero()
}
