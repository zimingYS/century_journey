use crate::content::constant::world::*;
use crate::engine::task::{TaskManager, TaskPriority, TaskResult};
use crate::game::player::components::Player;
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::generation::WorldGenerator;
use crate::game::world::generation::context::ChunkGenContext;
use crate::game::world::generation::noise::CachedBlockIds;
use crate::game::world::generation::structure::StructureGenerator;
use crate::game::world::save::format::SavedChunk;
use crate::game::world::save::region::RegionManager;
use crate::game::world::save::system::{CachedBlockIdRemap, SaveConfig, SaveQueue};
use crate::game::world::storage::WorldStorage;
use crate::game::world::systems::channel::{StructureGenChannel, StructureGenResult};
use crate::game::world::systems::{
    DIRECTIONS, PlayerChunkCache, TerrainGenChannel, TerrainGenResult,
};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 区块加载与卸载调度
pub fn manage_chunks_system(
    mut commands: Commands,
    mut save_queue: ResMut<SaveQueue>,
    mut world_storage: ResMut<WorldStorage>,
    mut player_cache: ResMut<PlayerChunkCache>,
    chunk_query: Query<(Entity, &ChunkComponents)>,
    player_query: Query<&Transform, With<Player>>,
    save_config: Res<SaveConfig>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let size_f32 = CHUNK_SIZE as f32;
    let render_distance = 8;
    let data_distance = render_distance + 1;

    let player_chunk_pos = IVec3::new(
        (player_pos.x / size_f32).floor() as i32,
        (player_pos.y / size_f32).floor() as i32,
        (player_pos.z / size_f32).floor() as i32,
    );

    let needs_rebuild = player_cache.last_chunk_pos != Some(player_chunk_pos);

    if needs_rebuild {
        player_cache.last_chunk_pos = Some(player_chunk_pos);
        let mut expected_chunks = HashSet::with_capacity_and_hasher(
            (data_distance * 2 + 1_i32).pow(3) as usize,
            Default::default(),
        );
        for x in -data_distance..=data_distance {
            for y in -data_distance..=data_distance {
                for z in -data_distance..=data_distance {
                    expected_chunks.insert(player_chunk_pos + IVec3::new(x, y, z));
                }
            }
        }
        player_cache.expected_chunks = expected_chunks;
    }

    let mut spawned = 0u32;
    for &chunk_pos in &player_cache.expected_chunks {
        if !world_storage.chunk_entities.contains_key(&chunk_pos) {
            if spawned >= MAX_SPAWN_PER_FRAME {
                break;
            }
            spawned += 1;

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
            world_storage.chunk_entities.insert(chunk_pos, entity);
        }
    }

    for (entity, chunk_components) in chunk_query.iter() {
        let pos = chunk_components.position;
        if !player_cache.expected_chunks.contains(&pos) {
            if save_config.save_on_unload {
                if let Some(chunk_data) = world_storage.loaded_chunks.get(&pos) {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs_f64();
                    save_queue.queue.push_back(SavedChunk {
                        position: pos,
                        data: chunk_data.as_ref().clone(),
                        modified_time: world_storage
                            .chunk_modified_times
                            .get(&pos)
                            .copied()
                            .unwrap_or(now),
                    });
                }
            }
            world_storage.chunk_entities.remove(&pos);
            world_storage.loaded_chunks.remove(&pos);
            world_storage.chunk_modified_times.remove(&pos);
            commands
                .entity(entity)
                .queue_silenced(|mut entity: EntityWorldMut| {
                    entity.despawn();
                });
        }
    }
}

/// 地形生成派发
pub fn spawn_terrain_gen_tasks(
    channel: Res<TerrainGenChannel>,
    world_generator: Res<WorldGenerator>,
    cached_ids: Res<CachedBlockIds>,
    save_config: Res<SaveConfig>,
    cached_remap: Res<CachedBlockIdRemap>,
    task: Res<TaskManager>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    let mut spawned = 0u32;

    for (_entity, chunk_components, mut chunk_state) in &mut chunk_query {
        if *chunk_state != ChunkState::Empty {
            continue;
        }
        if spawned >= MAX_TERRAIN_TASKS_PER_FRAME {
            break;
        }

        let chunk_pos = chunk_components.position;

        if world_storage.loaded_chunks.contains_key(&chunk_pos) {
            *chunk_state = ChunkState::TerrainReady;
            spawned += 1;
            continue;
        }

        let sender = channel.sender.clone();
        let world_name = save_config.world_name.clone();
        let remap = cached_remap.0.clone();
        let block_ids = cached_ids.0.clone();
        let biome_registry = Arc::clone(&world_generator.shared_biome);
        let noise_sampler = Arc::clone(&world_generator.shared_noise);
        let climate_sampler = Arc::clone(&world_generator.shared_climate);
        let current_season = world_generator.pipeline.climate_sampler.current_season;

        task.spawn_cpu(TaskPriority::Normal, move || {
            match RegionManager::read_chunk(&world_name, chunk_pos) {
                Ok(Some(mut saved)) => {
                    // 只在有映射表时才重映射
                    if !remap.is_empty() {
                        for voxel in saved.data.voxels.iter_mut() {
                            if let Some(&new_id) = remap.get(voxel) {
                                *voxel = new_id;
                            } else if *voxel != 0 {
                                *voxel = 0;
                            }
                        }
                    }
                    // 重映射已在上面完成,直接发送
                    let _ = sender.send(TerrainGenResult {
                        chunk_pos,
                        chunk_data: saved.data,
                        gen_context: ChunkGenContext::new(chunk_pos),
                    });
                }
                _ => {
                    // 磁盘未命中 → 程序化生成
                    let ctx =
                        crate::game::world::generation::terrain::TerrainGenerator::sample_context(
                            &noise_sampler,
                            &climate_sampler,
                            current_season,
                            &biome_registry,
                            chunk_pos,
                        );
                    let chunk_data =
                        crate::game::world::generation::terrain::TerrainGenerator::generate_terrain(
                            &ctx,
                            &block_ids,
                            &biome_registry,
                        );
                    let _ = sender.send(TerrainGenResult {
                        chunk_pos,
                        chunk_data,
                        gen_context: ctx,
                    });
                }
            }
            TaskResult::Success
        });

        *chunk_state = ChunkState::GeneratingTerrain;
        spawned += 1;
    }
}

/// 地形生成接收
pub fn receive_terrain_results(
    mut world_storage: ResMut<WorldStorage>,
    channel: Res<TerrainGenChannel>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    let receiver = channel.receiver.lock().unwrap();
    let mut received = 0usize;

    while received < MAX_TERRAIN_RECEIVE_PER_FRAME {
        let Ok(result) = receiver.try_recv() else {
            break;
        };
        received += 1;

        let chunk_pos = result.chunk_pos;
        let mut chunk_data = result.chunk_data;
        let gen_ctx = result.gen_context;

        apply_pending_writes(chunk_pos, &mut chunk_data, &mut world_storage);
        world_storage
            .loaded_chunks
            .insert(chunk_pos, Arc::from(chunk_data));
        if !gen_ctx.columns.is_empty() {
            world_storage.gen_contexts.insert(chunk_pos, gen_ctx);
        }

        for (chunk_components, mut chunk_state) in &mut chunk_query {
            if chunk_components.position == chunk_pos
                && *chunk_state == ChunkState::GeneratingTerrain
            {
                *chunk_state = ChunkState::TerrainReady;
            }
        }
    }
}

/// 结构生成（主线程，有严格时间预算）
pub fn generate_structures_system(
    world_storage: Res<WorldStorage>,
    channel: Res<StructureGenChannel>,
    world_generator: Res<WorldGenerator>,
    cached_ids: Res<CachedBlockIds>,
    task: Res<TaskManager>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    let mut spawned = 0u32;

    for (chunk_components, mut chunk_state) in &mut chunk_query {
        if *chunk_state != ChunkState::TerrainReady {
            continue;
        }
        if spawned >= MAX_STRUCTURE_TASKS_PER_FRAME {
            break;
        }

        let chunk_pos = chunk_components.position;

        // 取当前区块数据
        let Some(chunk_data) = world_storage.loaded_chunks.get(&chunk_pos).cloned() else {
            continue;
        };

        // 取生成上下文（有缓存用缓存，没有就现场采样）
        let ctx = world_storage
            .gen_contexts
            .get(&chunk_pos)
            .cloned()
            .unwrap_or_else(|| {
                crate::game::world::generation::terrain::TerrainGenerator::sample_context(
                    &world_generator.pipeline.noise_sampler,
                    &world_generator.pipeline.climate_sampler,
                    world_generator.pipeline.climate_sampler.current_season,
                    &world_generator.pipeline.biome_registry,
                    chunk_pos,
                )
            });

        // 收集邻居数据（跨区块树冠写入需要）
        let mut neighbor_data: HashMap<IVec3, ChunkData> = HashMap::new();
        for (dir, _) in &DIRECTIONS {
            let nbr_pos = chunk_pos + *dir;
            if let Some(data) = world_storage.loaded_chunks.get(&nbr_pos).cloned() {
                neighbor_data.insert(nbr_pos, data.as_ref().clone());
            }
        }

        let sender = channel.sender.clone();
        let block_ids = cached_ids.0.clone();
        let biome_registry = world_generator.pipeline.biome_registry.clone();
        let seed = world_generator.seed;

        task.spawn_cpu(TaskPriority::Normal, move || {
            // 构建临时 WorldStorage，复用现有 generate_structures_world_aware
            let mut temp_storage = crate::game::world::storage::WorldStorage::default();
            temp_storage.loaded_chunks.insert(chunk_pos, chunk_data);
            for (pos, data) in neighbor_data {
                temp_storage.loaded_chunks.insert(pos, Arc::from(data));
            }

            StructureGenerator::generate_structures_world_aware(
                chunk_pos,
                &ctx,
                &block_ids,
                &biome_registry,
                seed,
                &mut temp_storage,
            );

            // 收集所有被修改的区块
            let modified_chunks: Vec<(IVec3, ChunkData)> = temp_storage
                .loaded_chunks
                .into_iter()
                .map(|(pos, arc)| (pos, Arc::unwrap_or_clone(arc)))
                .collect();

            let _ = sender.send(StructureGenResult {
                chunk_pos,
                modified_chunks,
            });
            TaskResult::Success
        });

        *chunk_state = ChunkState::GeneratingStructure;
        spawned += 1;
    }
}

pub fn receive_structure_results(
    mut world_storage: ResMut<WorldStorage>,
    channel: Res<StructureGenChannel>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    let receiver = channel.receiver.lock().unwrap();
    let mut received = 0usize;

    while received < MAX_STRUCTURE_RECEIVE_PER_FRAME {
        let Ok(result) = receiver.try_recv() else {
            break;
        };
        received += 1;

        // 合并异步任务中修改的所有区块数据
        for (pos, data) in result.modified_chunks {
            if let Some(existing) = world_storage.loaded_chunks.get_mut(&pos) {
                // 只覆盖非空区块的变更（结构可能写入了邻居区块的树冠）
                *existing = Arc::from(data);
            } else {
                // 邻居区块还未加载到 loaded_chunks，暂存为 pending_writes
                // 或直接插入（如果该区块实体已存在但数据尚未就绪）
                if world_storage.chunk_entities.contains_key(&pos) {
                    world_storage.loaded_chunks.insert(pos, Arc::from(data));
                }
            }
        }

        // 清除已使用的生成上下文缓存
        world_storage.gen_contexts.remove(&result.chunk_pos);

        // 更新区块状态
        for (chunk_components, mut chunk_state) in &mut chunk_query {
            if chunk_components.position == result.chunk_pos
                && *chunk_state == ChunkState::GeneratingStructure
            {
                *chunk_state = ChunkState::StructureReady;
            }
        }
    }
}

/// 延迟写入
fn apply_pending_writes(chunk_pos: IVec3, chunk: &mut ChunkData, storage: &mut WorldStorage) {
    if let Some(writes) = storage.pending_writes.writes.remove(&chunk_pos) {
        for write in writes {
            if chunk.get_voxel(write.local_x, write.local_y, write.local_z) == 0 {
                chunk.set_voxel(write.local_x, write.local_y, write.local_z, write.block_id);
            }
        }
    }
}
