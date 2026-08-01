//! 在任务池恢复或生成基础地形，并在主线程按请求实体提交权威结果。

use crate::engine::task::{TaskManager, TaskResult};
use crate::game::save::world::chunk::region::RegionManager;
use crate::game::save::{
    CachedBlockIdRemap, SaveConfig, SaveQueue, SaveWorker, latest_snapshot_for_load,
};
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::generation::block_ids::CachedBlockIds;
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::generation::runtime::{
    TerrainGenChannel, TerrainGenOutcome, TerrainGenResult,
};
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::streaming::PlayerChunkCache;
use bevy::math::IVec3;
use bevy::prelude::{Query, Res, ResMut};
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// 单帧最多派发的地形生成任务数。
const MAX_TERRAIN_TASKS_PER_FRAME: u32 = 4;
/// 单帧最多接收的地形生成结果数。
const MAX_TERRAIN_RECEIVE_PER_FRAME: usize = 4;

/// 按流送优先级派发加载或生成任务。
///
/// 读取顺序固定为待写快照、飞行中快照、磁盘、首次生成；内存快照使用当前运行时
/// 方块 ID，只有磁盘快照需要经过内容 ID 重映射。
#[allow(clippy::too_many_arguments)]
pub fn spawn_terrain_gen_tasks(
    channel: Res<TerrainGenChannel>,
    world_generator: Res<WorldGenerator>,
    cached_ids: Res<CachedBlockIds>,
    save_config: Res<SaveConfig>,
    cached_remap: Res<CachedBlockIdRemap>,
    save_queue: Res<SaveQueue>,
    save_worker: Res<SaveWorker>,
    task: Res<TaskManager>,
    world_state: Res<WorldState>,
    player_cache: Res<PlayerChunkCache>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
    chunk_runtime: Res<ChunkRuntime>,
) {
    let mut spawned = 0u32;
    let max_in_flight = task.worker_count().max(1);

    for &chunk_pos in player_cache.ordered_chunks() {
        if spawned >= MAX_TERRAIN_TASKS_PER_FRAME
            || channel.in_flight.load(Ordering::Relaxed) >= max_in_flight
        {
            break;
        }
        let Some(entity) = chunk_runtime.chunk_entity(chunk_pos) else {
            continue;
        };
        let Ok((chunk_components, mut chunk_state)) = chunk_query.get_mut(entity) else {
            continue;
        };
        if chunk_components.position != chunk_pos || *chunk_state != ChunkState::Empty {
            continue;
        }

        if world_state.contains_chunk(chunk_pos) {
            *chunk_state = ChunkState::StructureReady;
            continue;
        }

        let memory_snapshot = latest_snapshot_for_load(&save_queue, &save_worker, chunk_pos);
        let sender = channel.sender.clone();
        let world_name = save_config.world_name.clone();
        let remap = cached_remap.0.clone();
        let block_ids = cached_ids.0.clone();
        let pipeline = world_generator.pipeline.clone();
        let in_flight = Arc::clone(&channel.in_flight);

        channel.in_flight.fetch_add(1, Ordering::Relaxed);
        task.spawn_cpu(move || {
            let outcome = match memory_snapshot {
                Some(saved) => TerrainGenOutcome::Restored {
                    chunk_data: Box::new(saved.data),
                    tree_instances: saved.tree_instances,
                },
                None => match RegionManager::read_chunk(&world_name, chunk_pos) {
                    Ok(Some(mut saved)) => {
                        if !remap.is_empty() {
                            for voxel in &mut saved.data.voxels {
                                if let Some(&new_id) = remap.get(voxel) {
                                    *voxel = new_id;
                                } else if *voxel != 0 {
                                    *voxel = 0;
                                }
                            }
                        }
                        TerrainGenOutcome::Restored {
                            chunk_data: Box::new(saved.data),
                            tree_instances: saved.tree_instances,
                        }
                    }
                    Ok(None) => {
                        let (chunk_data, gen_context) =
                            pipeline.generate_base_chunk(chunk_pos, &block_ids);
                        TerrainGenOutcome::Generated {
                            chunk_data: Box::new(chunk_data),
                            gen_context,
                        }
                    }
                    Err(error) => TerrainGenOutcome::LoadFailed(error.to_string()),
                },
            };
            let task_result = match &outcome {
                TerrainGenOutcome::Restored { .. } | TerrainGenOutcome::Generated { .. } => {
                    TaskResult::Success
                }
                TerrainGenOutcome::LoadFailed(error) => TaskResult::Failed(error.clone()),
            };
            let result = sender.send(TerrainGenResult {
                chunk_pos,
                request_entity: entity,
                outcome,
            });
            if result.is_err() {
                in_flight.fetch_sub(1, Ordering::Relaxed);
            }
            task_result
        });

        *chunk_state = ChunkState::GeneratingTerrain;
        spawned += 1;
    }
}

/// 在主线程接收地形结果，拒绝已经卸载或被同坐标新实体替代的旧任务。
pub fn receive_terrain_results(
    mut world_state: ResMut<WorldState>,
    channel: Res<TerrainGenChannel>,
    mut chunk_runtime: ResMut<ChunkRuntime>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    let receiver = channel.receiver.lock().unwrap();
    let mut received = 0usize;

    while received < MAX_TERRAIN_RECEIVE_PER_FRAME {
        let Ok(result) = receiver.try_recv() else {
            break;
        };
        channel.in_flight.fetch_sub(1, Ordering::Relaxed);
        received += 1;

        let chunk_pos = result.chunk_pos;
        if chunk_runtime.chunk_entity(chunk_pos) != Some(result.request_entity) {
            continue;
        }
        let Ok((chunk_components, mut chunk_state)) = chunk_query.get_mut(result.request_entity)
        else {
            continue;
        };
        if chunk_components.position != chunk_pos || *chunk_state != ChunkState::GeneratingTerrain {
            continue;
        }

        let (mut chunk_data, tree_instances, gen_context, next_state) = match result.outcome {
            TerrainGenOutcome::Restored {
                chunk_data,
                tree_instances,
            } => (
                *chunk_data,
                tree_instances,
                None,
                ChunkState::StructureReady,
            ),
            TerrainGenOutcome::Generated {
                chunk_data,
                gen_context,
            } => (
                *chunk_data,
                Vec::new(),
                Some(gen_context),
                ChunkState::TerrainReady,
            ),
            TerrainGenOutcome::LoadFailed(error) => {
                log::error!("[存档系统] 区块 {chunk_pos:?} 读取失败，已阻止重新生成: {error}");
                *chunk_state = ChunkState::LoadFailed;
                continue;
            }
        };

        apply_pending_writes(chunk_pos, &mut chunk_data, &mut world_state);
        if let Err(error) =
            world_state.insert_restored_chunk(chunk_pos, Arc::from(chunk_data), tree_instances)
        {
            log::error!("[存档系统] 区块 {chunk_pos:?} 树实例恢复失败: {error}");
            *chunk_state = ChunkState::LoadFailed;
            continue;
        }
        if let Some(gen_context) = gen_context {
            chunk_runtime.cache_generation_context(chunk_pos, gen_context);
        } else {
            chunk_runtime.remove_generation_context(chunk_pos);
        }

        *chunk_state = next_state;
    }
}

fn apply_pending_writes(chunk_pos: IVec3, chunk: &mut ChunkData, world_state: &mut WorldState) {
    if let Some(writes) = world_state.take_pending_writes(chunk_pos) {
        for write in writes {
            if chunk.get_voxel(write.local_x, write.local_y, write.local_z) == 0 {
                chunk.set_voxel(write.local_x, write.local_y, write.local_z, write.block_id);
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/world/generation/runtime/terrain_tasks.rs"]
mod tests;
