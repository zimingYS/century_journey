use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crate::core::constant::world::*;
use crate::voxel::registry::BlockRegistry;
use crate::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::world::storage::WorldStorage;
use crate::world::systems::{build_greedy_mesh, BlockInfoSnapshot, MeshBuildChannel, MeshBuildInput, DIRECTIONS};

/// Mesh 构建派发
pub fn spawn_mesh_build_tasks(
    channel: Res<MeshBuildChannel>,
    registry: Option<Res<BlockRegistry>>,
    world_storage: Res<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    let Some(reg) = registry else { return };

    let ready_count = chunk_query.iter()
        .filter(|(_, c, s)| **s == ChunkState::StructureReady
            && world_storage.chunk_entities.contains_key(&c.position))
        .count();
    if ready_count == 0 { return; }

    let block_info = BlockInfoSnapshot::from_registry(&reg);
    let mut spawned = 0u32;

    for (_chunk_entity, chunk_components, mut state) in &mut chunk_query {
        if !world_storage.chunk_entities.contains_key(&chunk_components.position) { continue; }
        if *state != ChunkState::StructureReady { continue; }
        if spawned >= MAX_MESH_TASKS_PER_FRAME { break; }

        let current_chunk_pos = chunk_components.position;

        let neighbors_ready = DIRECTIONS.iter()
            .all(|(dir, _)| world_storage.loaded_chunks.contains_key(&(current_chunk_pos + *dir)));
        if !neighbors_ready { continue; }

        let Some(current_chunk_data) = world_storage.loaded_chunks.get(&current_chunk_pos) else { continue };

        let current_data = current_chunk_data.clone();
        let neighbors: [Option<ChunkData>; 6] = DIRECTIONS.map(|(dir, _)| {
            world_storage.loaded_chunks.get(&(current_chunk_pos + dir)).cloned()
        });

        let sender = channel.sender.clone();
        let input = MeshBuildInput {
            chunk_pos: current_chunk_pos,
            current_data,
            neighbors,
            block_info: block_info.clone(),
        };

        AsyncComputeTaskPool::get().spawn(async move {
            let result = build_greedy_mesh(input);
            let _ = sender.send(result);
        }).detach();

        *state = ChunkState::GeneratingMesh;
        spawned += 1;
    }
}

/// 接收 Mesh 构建结果
pub fn receive_mesh_results(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    channel: Res<MeshBuildChannel>,
    registry: Option<Res<BlockRegistry>>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    let Some(reg) = registry else { return };
    let opaque_mat = reg.opaque_material.clone();
    let cutout_mat = reg.cutout_material.clone();
    let transparent_mat = reg.transparent_material.clone();

    let receiver = channel.receiver.lock().unwrap();
    let mut received = 0usize;

    while received < MAX_MESH_RECEIVE_PER_FRAME {
        let Ok(result) = receiver.try_recv() else { break };
        received += 1;

        let Some((chunk_entity, _, mut state)) = chunk_query
            .iter_mut()
            .find(|(_, c, _)| c.position == result.chunk_pos)
        else { continue };

        if *state != ChunkState::GeneratingMesh { continue; }

        // 清除旧 Mesh
        commands.entity(chunk_entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity.remove::<Mesh3d>().remove::<MeshMaterial3d<StandardMaterial>>();
            });
        commands.entity(chunk_entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity.despawn_related::<Children>();
            });

        // 不透明
        if !result.opaque.is_empty() {
            let opaque_mesh = meshes.add(result.opaque.build_mesh());
            let mat = opaque_mat.clone();
            commands.entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.insert((Mesh3d(opaque_mesh), MeshMaterial3d(mat)));
                });
        }

        // 镂空
        if !result.cutout.is_empty() {
            let cutout_mesh = meshes.add(result.cutout.build_mesh());
            let mat = cutout_mat.clone();
            let child = commands.spawn((
                Mesh3d(cutout_mesh), MeshMaterial3d(mat),
                Transform::IDENTITY, Visibility::default(),
            )).id();
            commands.entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| { entity.add_child(child); });
        }

        // 水
        if !result.water.is_empty() {
            let water_mesh = meshes.add(result.water.build_mesh());
            let mat = transparent_mat.clone();
            let child = commands.spawn((
                Mesh3d(water_mesh), MeshMaterial3d(mat),
                Transform::IDENTITY, Visibility::default(),
            )).id();
            commands.entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| { entity.add_child(child); });
        }

        *state = ChunkState::Rendered;
    }
}