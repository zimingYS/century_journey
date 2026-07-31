//! 调度结构生成并合并跨区块延迟写入，保持结果可复现。

use crate::engine::task::{TaskManager, TaskResult};
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::generation::block_ids::CachedBlockIds;
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::generation::runtime::{StructureGenChannel, StructureGenResult};
use crate::game::world::generation::structure::placement::{
    StructureGenerationWorkspace, StructureGenerator,
};
use crate::game::world::state::ChunkRuntime;
use crate::game::world::state::WorldState;
use crate::game::world::streaming::PlayerChunkCache;
use bevy::math::IVec3;
use bevy::prelude::{Query, Res, ResMut};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// 单帧最多派发的结构生成任务数。
const MAX_STRUCTURE_TASKS_PER_FRAME: u32 = 4;
/// 单帧最多接收的结构生成结果数。
const MAX_STRUCTURE_RECEIVE_PER_FRAME: usize = 4;

const CHUNK_NEIGHBOR_OFFSETS: [IVec3; 6] = [
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(1, 0, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// 按流送优先级派发结构任务，并为任务提供相邻区块快照。
/// 任务派发同时受流送窗口、任务池和区块状态约束，资源访问保持显式。
#[allow(clippy::too_many_arguments)]
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

    for &chunk_pos in player_cache.ordered_chunks() {
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
            .generation_context(chunk_pos)
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
                request_entity: entity,
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

/// 在主线程合并结构修改和延迟写入，并推进区块生命周期状态。
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

        if chunk_runtime.chunk_entity(result.chunk_pos) != Some(result.request_entity) {
            continue;
        }
        let Ok((result_components, result_state)) = chunk_query.get(result.request_entity) else {
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

        chunk_runtime.remove_generation_context(result.chunk_pos);

        if chunk_runtime.chunk_entity(result.chunk_pos) == Some(result.request_entity)
            && let Ok((chunk_components, mut chunk_state)) =
                chunk_query.get_mut(result.request_entity)
            && chunk_components.position == result.chunk_pos
            && *chunk_state == ChunkState::GeneratingStructure
        {
            *chunk_state = ChunkState::StructureReady;
        }
    }
}
