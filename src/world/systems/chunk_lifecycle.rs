use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crate::core::constant::world::*;
use crate::player::components::Player;
use crate::voxel::registry::BlockRegistry;
use crate::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::world::generation::context::ChunkGenContext;
use crate::world::generation::noise::CachedBlockIds;
use crate::world::generation::structure::StructureGenerator;
use crate::world::generation::WorldGenerator;
use crate::world::save;
use crate::world::save::format::SavedChunk;
use crate::world::save::system::{SaveConfig, SaveQueue};
use crate::world::storage::WorldStorage;
use crate::world::systems::{PlayerChunkCache, TerrainGenChannel, TerrainGenResult, DIRECTIONS};
use crate::world::systems::channel::{StructureGenChannel, StructureGenResult};

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
    let Ok(player_transform) = player_query.single() else { return };

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
            if spawned >= MAX_SPAWN_PER_FRAME { break; }
            spawned += 1;

            let entity = commands.spawn((
                ChunkComponents { position: chunk_pos },
                ChunkState::Empty,
                Transform::from_translation(Vec3::new(
                    (chunk_pos.x * CHUNK_SIZE as i32) as f32,
                    (chunk_pos.y * CHUNK_SIZE as i32) as f32,
                    (chunk_pos.z * CHUNK_SIZE as i32) as f32,
                )),
                Visibility::default(),
            )).id();
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
                        data: chunk_data.clone(),
                        modified_time: world_storage.chunk_modified_times.get(&pos).copied().unwrap_or(now),
                    });
                }
            }
            world_storage.chunk_entities.remove(&pos);
            world_storage.loaded_chunks.remove(&pos);
            world_storage.chunk_modified_times.remove(&pos);
            commands.entity(entity)
                .queue_silenced(|mut entity: EntityWorldMut| { entity.despawn(); });
        }
    }
}

/// 地形生成派发
pub fn spawn_terrain_gen_tasks(
    block_registry: Res<BlockRegistry>,
    channel: Res<TerrainGenChannel>,
    world_generator: Res<WorldGenerator>,
    cached_ids: Res<CachedBlockIds>,
    save_config: Res<SaveConfig>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    let mut spawned = 0u32;

    for (_entity, chunk_components, mut chunk_state) in &mut chunk_query {
        if *chunk_state != ChunkState::Empty { continue; }
        if spawned >= MAX_TERRAIN_TASKS_PER_FRAME { break; }

        let chunk_pos = chunk_components.position;

        if world_storage.loaded_chunks.contains_key(&chunk_pos) {
            *chunk_state = ChunkState::TerrainReady;
            spawned += 1;
            continue;
        }

        if let Some(mut saved) = save::system::try_load_chunk_from_disk(&save_config.world_name, chunk_pos) {
            if let Ok(level_data) = save::level::load_level(&save_config.world_name) {
                save::level::remap_chunk_block_ids(&mut saved.data, &level_data.block_id_map, &block_registry);
            }
            let _placeholder_ctx = ChunkGenContext::new(chunk_pos);
            world_storage.loaded_chunks.insert(chunk_pos, saved.data);
            *chunk_state = ChunkState::TerrainReady;
            spawned += 1;
            continue;
        }

        let sender = channel.sender.clone();
        let block_ids = cached_ids.0;
        let seed = world_generator.seed;
        let climate_config = world_generator.pipeline.climate_sampler.config.clone();
        let current_season = world_generator.pipeline.climate_sampler.current_season;
        let biome_registry = world_generator.pipeline.biome_registry.clone();

        AsyncComputeTaskPool::get().spawn(async move {
            let pipeline = crate::world::generation::pipeline::GenerationPipeline::rebuild_from_seed(
                seed, climate_config, current_season, biome_registry,
            );

            let ctx = crate::world::generation::terrain::TerrainGenerator::sample_context(
                &pipeline.noise_sampler,
                &pipeline.climate_sampler,
                &pipeline.biome_registry,
                chunk_pos,
            );

            let chunk_data = crate::world::generation::terrain::TerrainGenerator::generate_terrain(
                &ctx, &block_ids, &pipeline.biome_registry,
            );
            let _ = sender.send(TerrainGenResult { chunk_pos, chunk_data, gen_context: ctx });
        }).detach();

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
        let Ok(result) = receiver.try_recv() else { break };
        received += 1;

        let chunk_pos = result.chunk_pos;
        let mut chunk_data = result.chunk_data;
        let gen_ctx = result.gen_context;

        apply_pending_writes(chunk_pos, &mut chunk_data, &mut world_storage);
        world_storage.loaded_chunks.insert(chunk_pos, chunk_data);
        world_storage.gen_contexts.insert(chunk_pos, gen_ctx);

        for (chunk_components, mut chunk_state) in &mut chunk_query {
            if chunk_components.position == chunk_pos && *chunk_state == ChunkState::GeneratingTerrain {
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
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    let mut spawned = 0u32;

    for (chunk_components, mut chunk_state) in &mut chunk_query {
        if *chunk_state != ChunkState::TerrainReady { continue; }
        if spawned >= MAX_STRUCTURE_TASKS_PER_FRAME { break; }

        let chunk_pos = chunk_components.position;

        // 取当前区块数据
        let Some(chunk_data) = world_storage.loaded_chunks.get(&chunk_pos).cloned() else {
            continue;
        };

        // 取生成上下文（有缓存用缓存，没有就现场采样）
        let ctx = world_storage.gen_contexts.get(&chunk_pos).cloned()
            .unwrap_or_else(|| {
                crate::world::generation::terrain::TerrainGenerator::sample_context(
                    &world_generator.pipeline.noise_sampler,
                    &world_generator.pipeline.climate_sampler,
                    &world_generator.pipeline.biome_registry,
                    chunk_pos,
                )
            });

        // 收集邻居数据（跨区块树冠写入需要）
        let mut neighbor_data: HashMap<IVec3, ChunkData> = HashMap::new();
        for (dir, _) in &DIRECTIONS {
            let nbr_pos = chunk_pos + *dir;
            if let Some(data) = world_storage.loaded_chunks.get(&nbr_pos).cloned() {
                neighbor_data.insert(nbr_pos, data);
            }
        }

        let sender = channel.sender.clone();
        let block_ids = cached_ids.0;
        let biome_registry = world_generator.pipeline.biome_registry.clone();
        let seed = world_generator.seed;

        AsyncComputeTaskPool::get().spawn(async move {
            // 构建临时 WorldStorage，复用现有 generate_structures_world_aware
            let mut temp_storage = crate::world::storage::WorldStorage::default();
            temp_storage.loaded_chunks.insert(chunk_pos, chunk_data);
            for (pos, data) in neighbor_data {
                temp_storage.loaded_chunks.insert(pos, data);
            }

            StructureGenerator::generate_structures_world_aware(
                chunk_pos, &ctx, &block_ids, &biome_registry, seed, &mut temp_storage,
            );

            // 收集所有被修改的区块
            let modified_chunks: Vec<(IVec3, ChunkData)> =
                temp_storage.loaded_chunks.into_iter().collect();

            let _ = sender.send(StructureGenResult { chunk_pos, modified_chunks });
        }).detach();

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
        let Ok(result) = receiver.try_recv() else { break };
        received += 1;

        // 合并异步任务中修改的所有区块数据
        for (pos, data) in result.modified_chunks {
            if let Some(existing) = world_storage.loaded_chunks.get_mut(&pos) {
                // 只覆盖非空区块的变更（结构可能写入了邻居区块的树冠）
                *existing = data;
            } else {
                // 邻居区块还未加载到 loaded_chunks，暂存为 pending_writes
                // 或直接插入（如果该区块实体已存在但数据尚未就绪）
                if world_storage.chunk_entities.contains_key(&pos) {
                    world_storage.loaded_chunks.insert(pos, data);
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
fn apply_pending_writes(
    chunk_pos: IVec3,
    chunk: &mut ChunkData,
    storage: &mut WorldStorage,
) {
    if let Some(writes) = storage.pending_writes.writes.remove(&chunk_pos) {
        for write in writes {
            if chunk.get_voxel(write.local_x, write.local_y, write.local_z) == 0 {
                chunk.set_voxel(write.local_x, write.local_y, write.local_z, write.block_id);
            }
        }
    }
}
