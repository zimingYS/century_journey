use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::*;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::storage::WorldStorage;
use crate::game::world::systems::{PlayerChunkCache, WorldStreamingConfig};
use bevy::prelude::*;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use super::{
    BlockInfoSnapshot, CachedBlockInfo, DIRECTIONS, MeshBuildChannel, MeshBuildInput,
    build_greedy_mesh,
};

pub fn rebuild_block_info_snapshot(
    registry: Res<BlockRegistry>,
    mut cached: ResMut<CachedBlockInfo>,
) {
    if registry.is_changed() {
        cached.0 = BlockInfoSnapshot::from_registry(&registry);
    }
}

pub fn spawn_mesh_build_tasks(
    channel: Res<MeshBuildChannel>,
    registry: Option<Res<BlockRegistry>>,
    world_storage: Res<WorldStorage>,
    cached_block_info: Res<CachedBlockInfo>,
    task: Res<TaskManager>,
    player_cache: Res<PlayerChunkCache>,
    streaming_config: Res<WorldStreamingConfig>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    if registry.is_none() {
        return;
    }
    let Some(player_chunk_pos) = player_cache.last_chunk_pos else {
        return;
    };

    let block_info = cached_block_info.0.clone();
    let mut spawned = 0u32;
    let max_in_flight = (task.worker_count().max(1) * 2).clamp(2, 8);

    for &current_chunk_pos in &player_cache.ordered_chunks {
        if spawned >= MAX_MESH_TASKS_PER_FRAME
            || channel.in_flight.load(Ordering::Relaxed) >= max_in_flight
        {
            break;
        }
        if !streaming_config.should_mesh_chunk(player_chunk_pos, current_chunk_pos) {
            continue;
        }
        let Some(&chunk_entity) = world_storage.chunk_entities.get(&current_chunk_pos) else {
            continue;
        };
        let Ok((chunk_components, mut state)) = chunk_query.get_mut(chunk_entity) else {
            continue;
        };
        if chunk_components.position != current_chunk_pos || *state != ChunkState::StructureReady {
            continue;
        }

        let neighbors_ready = DIRECTIONS.iter().all(|(dir, _)| {
            world_storage
                .loaded_chunks
                .contains_key(&(current_chunk_pos + *dir))
        });
        if !neighbors_ready {
            continue;
        }

        let Some(current_chunk_data) = world_storage.loaded_chunks.get(&current_chunk_pos) else {
            continue;
        };

        let current_data = Arc::clone(current_chunk_data);
        let neighbors: [Option<Arc<ChunkData>>; 6] = DIRECTIONS.map(|(dir, _)| {
            world_storage
                .loaded_chunks
                .get(&(current_chunk_pos + dir))
                .map(Arc::clone)
        });

        let sender = channel.sender.clone();
        let in_flight = Arc::clone(&channel.in_flight);
        let input = MeshBuildInput {
            chunk_pos: current_chunk_pos,
            current_data,
            neighbors,
            block_info: block_info.clone(),
        };

        channel.in_flight.fetch_add(1, Ordering::Relaxed);
        task.spawn_cpu(move || {
            let result = build_greedy_mesh(input);
            if sender.send(result).is_err() {
                in_flight.fetch_sub(1, Ordering::Relaxed);
            }
            TaskResult::Success
        });

        *state = ChunkState::GeneratingMesh;
        spawned += 1;
    }
}

pub fn receive_mesh_results(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    channel: Res<MeshBuildChannel>,
    render_assets: Option<Res<BlockRenderAssets>>,
    world_storage: Res<WorldStorage>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    let Some(render_assets) = render_assets else {
        return;
    };
    let opaque_mat = render_assets.opaque_material().clone();
    let cutout_mat = render_assets.cutout_material().clone();
    let transparent_mat = render_assets.transparent_material().clone();

    let receiver = channel.receiver.lock().unwrap();
    let mut received = 0usize;
    let frame_start = Instant::now();
    const RECEIVE_BUDGET_MS: f64 = 2.0;

    while received < MAX_MESH_RECEIVE_PER_FRAME {
        if received > 0 && frame_start.elapsed().as_secs_f64() * 1000.0 >= RECEIVE_BUDGET_MS {
            break;
        }
        let Ok(result) = receiver.try_recv() else {
            break;
        };
        channel.in_flight.fetch_sub(1, Ordering::Relaxed);
        received += 1;

        let Some(&chunk_entity) = world_storage.chunk_entities.get(&result.chunk_pos) else {
            continue;
        };
        let Ok((_components, mut state)) = chunk_query.get_mut(chunk_entity) else {
            continue;
        };
        if *state != ChunkState::GeneratingMesh {
            continue;
        }

        commands
            .entity(chunk_entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity
                    .remove::<Mesh3d>()
                    .remove::<MeshMaterial3d<StandardMaterial>>();
            });
        commands
            .entity(chunk_entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity.despawn_related::<Children>();
            });

        if !result.opaque.is_empty() {
            let opaque_mesh = meshes.add(result.opaque.build_mesh());
            let mat = opaque_mat.clone();
            commands
                .entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.insert((Mesh3d(opaque_mesh), MeshMaterial3d(mat)));
                });
        }

        if !result.cutout.is_empty() {
            let cutout_mesh = meshes.add(result.cutout.build_mesh());
            let mat = cutout_mat.clone();
            let child = commands
                .spawn((
                    Mesh3d(cutout_mesh),
                    MeshMaterial3d(mat),
                    Transform::IDENTITY,
                    Visibility::default(),
                ))
                .id();
            commands
                .entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.add_child(child);
                });
        }

        if !result.water.is_empty() {
            let water_mesh = meshes.add(result.water.build_mesh());
            let mat = transparent_mat.clone();
            let child = commands
                .spawn((
                    Mesh3d(water_mesh),
                    MeshMaterial3d(mat),
                    Transform::IDENTITY,
                    Visibility::default(),
                ))
                .id();
            commands
                .entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.add_child(child);
                });
        }

        *state = ChunkState::Rendered;
    }
}
