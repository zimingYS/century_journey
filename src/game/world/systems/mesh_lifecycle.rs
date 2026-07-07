use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::*;
use crate::engine::task::{TaskManager, TaskPriority, TaskResult};
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::storage::WorldStorage;
use crate::game::world::systems::{
    BlockInfoSnapshot, CachedBlockInfo, DIRECTIONS, MeshBuildChannel, MeshBuildInput,
    build_greedy_mesh,
};
use bevy::prelude::*;
use std::sync::Arc;

/// 在BlockRegistry变化时重建缓存的系统
pub fn rebuild_block_info_snapshot(
    registry: Res<BlockRegistry>,
    mut cached: ResMut<CachedBlockInfo>,
) {
    // 自动处理
    if registry.is_changed() {
        cached.0 = BlockInfoSnapshot::from_registry(&registry);
    }
}

/// Mesh 构建派发
pub fn spawn_mesh_build_tasks(
    channel: Res<MeshBuildChannel>,
    registry: Option<Res<BlockRegistry>>,
    world_storage: Res<WorldStorage>,
    cached_block_info: Res<CachedBlockInfo>,
    task: Res<TaskManager>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    if registry.is_none() {
        return;
    }

    let ready_count = chunk_query
        .iter()
        .filter(|(_, c, s)| {
            **s == ChunkState::StructureReady
                && world_storage.chunk_entities.contains_key(&c.position)
        })
        .count();
    if ready_count == 0 {
        return;
    }

    let block_info = cached_block_info.0.clone();
    let mut spawned = 0u32;

    for (_chunk_entity, chunk_components, mut state) in &mut chunk_query {
        if !world_storage
            .chunk_entities
            .contains_key(&chunk_components.position)
        {
            continue;
        }
        if *state != ChunkState::StructureReady {
            continue;
        }
        if spawned >= MAX_MESH_TASKS_PER_FRAME {
            break;
        }

        let current_chunk_pos = chunk_components.position;

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
        let input = MeshBuildInput {
            chunk_pos: current_chunk_pos,
            current_data,
            neighbors,
            block_info: block_info.clone(),
        };

        task.spawn_cpu(TaskPriority::Normal, move || {
            let result = build_greedy_mesh(input);
            let _ = sender.send(result);
            TaskResult::Success
        });

        *state = ChunkState::GeneratingMesh;
        spawned += 1;
    }
}

/// 接收 Mesh 构建结果
pub fn receive_mesh_results(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    channel: Res<MeshBuildChannel>,
    render_assets: Option<Res<BlockRenderAssets>>,
    world_storage: Res<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    let Some(render_assets) = render_assets else {
        return;
    };
    let opaque_mat = render_assets.opaque_material().clone();
    let cutout_mat = render_assets.cutout_material().clone();
    let transparent_mat = render_assets.transparent_material().clone();

    let receiver = channel.receiver.lock().unwrap();
    let mut received = 0usize;

    while received < MAX_MESH_RECEIVE_PER_FRAME {
        let Ok(result) = receiver.try_recv() else {
            break;
        };
        received += 1;

        let Some(&chunk_entity) = world_storage.chunk_entities.get(&result.chunk_pos) else {
            continue;
        };
        let Ok((_entity, _components, mut state)) = chunk_query.get_mut(chunk_entity) else {
            continue;
        };
        if *state != ChunkState::GeneratingMesh {
            continue;
        }

        // 清除旧 Mesh
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

        // 不透明
        if !result.opaque.is_empty() {
            let opaque_mesh = meshes.add(result.opaque.build_mesh());
            let mat = opaque_mat.clone();
            commands
                .entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.insert((Mesh3d(opaque_mesh), MeshMaterial3d(mat)));
                });
        }

        // 镂空
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

        // 水
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
