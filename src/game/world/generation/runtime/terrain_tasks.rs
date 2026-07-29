use crate::content::constant::world::{MAX_TERRAIN_RECEIVE_PER_FRAME, MAX_TERRAIN_TASKS_PER_FRAME};
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::save::world::chunk::region::RegionManager;
use crate::game::save::{CachedBlockIdRemap, SaveConfig};
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::generation::block_ids::CachedBlockIds;
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::generation::runtime::{TerrainGenChannel, TerrainGenResult};
use crate::game::world::generation::terrain::context::ChunkGenContext;
use crate::game::world::state::WorldState;
use crate::game::world::state::ChunkRuntime;
use crate::game::world::streaming::PlayerChunkCache;
use bevy::math::IVec3;
use bevy::prelude::{Query, Res, ResMut};
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub fn spawn_terrain_gen_tasks(
    channel: Res<TerrainGenChannel>,
    world_generator: Res<WorldGenerator>,
    cached_ids: Res<CachedBlockIds>,
    save_config: Res<SaveConfig>,
    cached_remap: Res<CachedBlockIdRemap>,
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
            *chunk_state = ChunkState::TerrainReady;
            continue;
        }

        let sender = channel.sender.clone();
        let world_name = save_config.world_name.clone();
        let remap = cached_remap.0.clone();
        let block_ids = cached_ids.0.clone();
        let pipeline = world_generator.pipeline.clone();
        let in_flight = Arc::clone(&channel.in_flight);

        channel.in_flight.fetch_add(1, Ordering::Relaxed);
        task.spawn_cpu(move || {
            let result = match RegionManager::read_chunk(&world_name, chunk_pos) {
                Ok(Some(mut saved)) => {
                    if !remap.is_empty() {
                        for voxel in saved.data.voxels.iter_mut() {
                            if let Some(&new_id) = remap.get(voxel) {
                                *voxel = new_id;
                            } else if *voxel != 0 {
                                *voxel = 0;
                            }
                        }
                    }
                    sender.send(TerrainGenResult {
                        chunk_pos,
                        chunk_data: saved.data,
                        gen_context: ChunkGenContext::new(chunk_pos),
                    })
                }
                _ => {
                    let (chunk_data, ctx) = pipeline.generate_base_chunk(chunk_pos, &block_ids);
                    sender.send(TerrainGenResult {
                        chunk_pos,
                        chunk_data,
                        gen_context: ctx,
                    })
                }
            };
            if result.is_err() {
                in_flight.fetch_sub(1, Ordering::Relaxed);
            }
            TaskResult::Success
        });

        *chunk_state = ChunkState::GeneratingTerrain;
        spawned += 1;
    }
}

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
        let mut chunk_data = result.chunk_data;
        let gen_ctx = result.gen_context;

        let Some(entity) = chunk_runtime.chunk_entity(chunk_pos) else {
            continue;
        };
        let Ok((chunk_components, mut chunk_state)) = chunk_query.get_mut(entity) else {
            continue;
        };
        if chunk_components.position != chunk_pos || *chunk_state != ChunkState::GeneratingTerrain {
            continue;
        }

        apply_pending_writes(chunk_pos, &mut chunk_data, &mut world_state);
        world_state.insert_chunk(chunk_pos, Arc::from(chunk_data));
        if !gen_ctx.columns.is_empty() {
            chunk_runtime.cache_generation_context(chunk_pos, gen_ctx);
        }

        *chunk_state = ChunkState::TerrainReady;
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
