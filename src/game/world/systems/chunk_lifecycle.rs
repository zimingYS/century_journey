use crate::content::constant::world::*;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::player::identity::Player;
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::generation::WorldGenerator;
use crate::game::world::generation::context::ChunkGenContext;
use crate::game::world::generation::noise::CachedBlockIds;
use crate::game::world::generation::structure::{StructureGenerationWorkspace, StructureGenerator};
use crate::game::world::save::format::SavedChunk;
use crate::game::world::save::region::RegionManager;
use crate::game::world::save::system::{CachedBlockIdRemap, SaveConfig, SaveQueue};
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::systems::channel::{StructureGenChannel, StructureGenResult};
use crate::game::world::systems::{
    PlayerChunkCache, TerrainGenChannel, TerrainGenResult, WorldStreamingConfig,
};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

const CHUNK_NEIGHBOR_OFFSETS: [IVec3; 6] = [
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(1, 0, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];
const MAX_UNLOAD_PER_FRAME: usize = 8;

pub fn manage_chunks_system(
    mut commands: Commands,
    mut save_queue: ResMut<SaveQueue>,
    mut player_cache: ResMut<PlayerChunkCache>,
    mut chunk_runtime: ResMut<ChunkRuntime>,
    mut world_state: ResMut<WorldState>,
    chunk_query: Query<(Entity, &ChunkComponents)>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&GlobalTransform, With<crate::shared::components::FpsCamera>>,
    save_config: Res<SaveConfig>,
    streaming_config: Res<WorldStreamingConfig>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_chunk_pos = WorldStreamingConfig::chunk_from_world(player_transform.translation);
    let view_forward_xz = view_forward_xz(player_transform, &camera_query);
    let needs_rebuild = player_cache.last_chunk_pos != Some(player_chunk_pos)
        || player_cache.last_streaming_config.as_ref() != Some(&*streaming_config);

    if needs_rebuild {
        player_cache.last_chunk_pos = Some(player_chunk_pos);
        player_cache.last_streaming_config = Some(streaming_config.clone());
        let (ordered_chunks, expected_chunks) =
            streaming_config.rebuild_expected_chunks(player_chunk_pos, view_forward_xz);
        player_cache.ordered_chunks = ordered_chunks;
        player_cache.expected_chunks = expected_chunks;
    }

    let mut spawned = 0u32;
    for &chunk_pos in &player_cache.ordered_chunks {
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
                Visibility::default(),
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
        if player_cache.expected_chunks.contains(&pos) {
            continue;
        }

        let unloaded_chunk = world_state.remove_chunk(pos);

        if save_config.save_on_unload
            && let Some(chunk_data) = unloaded_chunk
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            save_queue.enqueue(SavedChunk {
                position: pos,
                data: chunk_data.as_ref().clone(),
                modified_time: world_state.chunk_modified_time(pos).unwrap_or(now),
            });
        }

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

    for &chunk_pos in &player_cache.ordered_chunks {
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
            chunk_runtime.gen_contexts.insert(chunk_pos, gen_ctx);
        }

        *chunk_state = ChunkState::TerrainReady;
    }
}

pub fn generate_structures_system(
    world_state: Res<WorldState>,
    channel: Res<StructureGenChannel>,
    world_generator: Res<WorldGenerator>,
    cached_ids: Res<CachedBlockIds>,
    task: Res<TaskManager>,
    player_cache: Res<PlayerChunkCache>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
    chunk_runtime: Res<ChunkRuntime>,
) {
    let mut spawned = 0u32;

    for &chunk_pos in &player_cache.ordered_chunks {
        if spawned >= MAX_STRUCTURE_TASKS_PER_FRAME
            || channel.in_flight.load(Ordering::Relaxed) >= 1
        {
            break;
        }
        let Some(entity) = chunk_runtime.chunk_entity(chunk_pos) else {
            continue;
        };
        let Ok((chunk_components, mut chunk_state)) = chunk_query.get_mut(entity) else {
            continue;
        };
        if chunk_components.position != chunk_pos || *chunk_state != ChunkState::TerrainReady {
            continue;
        }

        let Some(chunk_data) = world_state.chunk(chunk_pos).cloned() else {
            continue;
        };

        let ctx = chunk_runtime
            .gen_contexts
            .get(&chunk_pos)
            .cloned()
            .unwrap_or_else(|| world_generator.pipeline.sample_context(chunk_pos));

        let mut input_chunks: HashMap<IVec3, Arc<ChunkData>> = HashMap::new();
        input_chunks.insert(chunk_pos, chunk_data);
        for direction in CHUNK_NEIGHBOR_OFFSETS {
            let nbr_pos = chunk_pos + direction;
            if let Some(data) = world_state.chunk(nbr_pos).cloned() {
                input_chunks.insert(nbr_pos, data);
            }
        }
        let original_chunks = input_chunks.clone();

        let sender = channel.sender.clone();
        let in_flight = Arc::clone(&channel.in_flight);
        let block_ids = cached_ids.0.clone();
        let biome_registry = Arc::clone(&world_generator.pipeline.biome_registry);
        let seed = world_generator.seed;

        channel.in_flight.fetch_add(1, Ordering::Relaxed);
        task.spawn_cpu(move || {
            let mut workspace = StructureGenerationWorkspace::new(input_chunks);

            StructureGenerator::generate_structures_world_aware(
                chunk_pos,
                &ctx,
                &block_ids,
                &biome_registry,
                seed,
                &mut workspace,
            );

            let (generated_chunks, pending_writes) = workspace.into_parts();

            let modified_chunks = generated_chunks
                .into_iter()
                .filter_map(|(pos, arc)| {
                    let changed = original_chunks
                        .get(&pos)
                        .is_none_or(|original| !Arc::ptr_eq(original, &arc));
                    changed.then(|| (pos, Arc::unwrap_or_clone(arc)))
                })
                .collect();

            let result = sender.send(StructureGenResult {
                chunk_pos,
                modified_chunks,
                pending_writes,
            });
            if result.is_err() {
                in_flight.fetch_sub(1, Ordering::Relaxed);
            }
            TaskResult::Success
        });

        *chunk_state = ChunkState::GeneratingStructure;
        spawned += 1;
    }
}

pub fn receive_structure_results(
    mut world_state: ResMut<WorldState>,
    channel: Res<StructureGenChannel>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
    mut chunk_runtime: ResMut<ChunkRuntime>,
) {
    let receiver = channel.receiver.lock().unwrap();
    let mut received = 0usize;

    while received < MAX_STRUCTURE_RECEIVE_PER_FRAME {
        let Ok(result) = receiver.try_recv() else {
            break;
        };
        channel.in_flight.fetch_sub(1, Ordering::Relaxed);
        received += 1;

        let Some(result_entity) = chunk_runtime.chunk_entity(result.chunk_pos) else {
            continue;
        };
        let Ok((result_components, result_state)) = chunk_query.get(result_entity) else {
            continue;
        };
        if result_components.position != result.chunk_pos
            || *result_state != ChunkState::GeneratingStructure
        {
            continue;
        }

        for (pos, data) in result.modified_chunks {
            if let Some(existing) = world_state.chunk_mut(pos) {
                *existing = Arc::from(data);
            } else if chunk_runtime.contains_chunk_entity(pos) {
                world_state.insert_chunk(pos, Arc::from(data));
            }
            if let Some(entity) = chunk_runtime.chunk_entity(pos)
                && let Ok((_, mut state)) = chunk_query.get_mut(entity)
                && matches!(*state, ChunkState::Rendered | ChunkState::GeneratingMesh)
            {
                *state = ChunkState::StructureReady;
            }
        }
        for (pos, writes) in result.pending_writes {
            world_state.queue_pending_writes(pos, writes);
        }

        chunk_runtime.gen_contexts.remove(&result.chunk_pos);

        if let Some(entity) = chunk_runtime.chunk_entity(result.chunk_pos)
            && let Ok((chunk_components, mut chunk_state)) = chunk_query.get_mut(entity)
            && chunk_components.position == result.chunk_pos
            && *chunk_state == ChunkState::GeneratingStructure
        {
            *chunk_state = ChunkState::StructureReady;
        }
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

fn view_forward_xz(
    player_transform: &Transform,
    camera_query: &Query<&GlobalTransform, With<crate::shared::components::FpsCamera>>,
) -> Vec2 {
    let forward = camera_query
        .single()
        .map(|camera_transform| camera_transform.compute_transform().rotation * Vec3::NEG_Z)
        .unwrap_or_else(|_| player_transform.rotation * Vec3::NEG_Z);
    Vec2::new(forward.x, forward.z).normalize_or_zero()
}
