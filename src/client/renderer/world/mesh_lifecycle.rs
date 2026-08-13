//! 管理区块网格任务的派发、接收、实体更新和卸载生命周期。

use super::{
    BlockInfoSnapshot, CachedBlockInfo, DIRECTIONS, MeshBuildChannel, MeshBuildInput,
    build_greedy_mesh,
};
use crate::client::renderer::lighting::material::VoxelMaterial;
use crate::client::renderer::tex_atlas::BlockRenderAssets;

use crate::content::block::event::BlockChangedEvent;
use crate::content::block::registry::BlockRegistry;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::game::world::lighting::WorldLighting;
use crate::game::world::lighting::chunk_light::ChunkLight;
use crate::game::world::state::ChunkRuntime;
use crate::game::world::state::WorldState;
use crate::game::world::streaming::PlayerChunkCache;
use crate::game::world::streaming::WorldStreamingConfig;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

/// 单帧最多派发的客户端网格构建任务数。
const MAX_MESH_TASKS_PER_FRAME: u32 = 16;
/// 单帧最多接收的客户端网格构建结果数。
const MAX_MESH_RECEIVE_PER_FRAME: usize = 16;

#[derive(Clone, Copy)]
struct ActiveMeshRequest {
    entity: Entity,
    request_id: u64,
}

/// 每个区块最后一次网格请求的身份，用于严格拒绝乱序和跨实体旧结果。
#[derive(Resource, Default)]
pub struct MeshRequestTracker {
    next_request_id: u64,
    active: HashMap<IVec3, ActiveMeshRequest>,
}

impl MeshRequestTracker {
    fn begin(&mut self, position: IVec3, entity: Entity) -> u64 {
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        let request_id = self.next_request_id;
        self.active
            .insert(position, ActiveMeshRequest { entity, request_id });
        request_id
    }

    fn is_current(&self, position: IVec3, entity: Entity, request_id: u64) -> bool {
        self.active
            .get(&position)
            .is_some_and(|active| active.entity == entity && active.request_id == request_id)
    }

    fn finish(&mut self, position: IVec3, entity: Entity, request_id: u64) {
        if self.is_current(position, entity, request_id) {
            self.active.remove(&position);
        }
    }

    fn retain_runtime_entities(&mut self, runtime: &ChunkRuntime) {
        self.active
            .retain(|position, active| runtime.chunk_entity(*position) == Some(active.entity));
    }
}

/// 玩家方块编辑产生的高优先级网格目标；保持顺序并自动去重。
#[derive(Resource, Default)]
pub struct PriorityMeshQueue {
    ordered: VecDeque<IVec3>,
    contained: HashSet<IVec3>,
}

impl PriorityMeshQueue {
    fn prioritize(&mut self, position: IVec3) {
        if self.contained.contains(&position) {
            self.ordered.retain(|queued| *queued != position);
        } else {
            self.contained.insert(position);
        }
        self.ordered.push_front(position);
    }

    fn pop_front(&mut self) -> Option<IVec3> {
        let position = self.ordered.pop_front()?;
        self.contained.remove(&position);
        Some(position)
    }

    fn enqueue(&mut self, position: IVec3) {
        if self.contained.insert(position) {
            self.ordered.push_back(position);
        }
    }

    fn len(&self) -> usize {
        self.ordered.len()
    }

    fn is_empty(&self) -> bool {
        self.ordered.is_empty()
    }

    fn clear(&mut self) {
        self.ordered.clear();
        self.contained.clear();
    }
}

enum MeshSpawnAttempt {
    Spawned,
    Retry,
    Drop,
}

/// 注册网格生命周期所需的请求版本和交互优先队列。
pub fn register_mesh_lifecycle_resources(app: &mut App) {
    app.init_resource::<MeshRequestTracker>()
        .init_resource::<PriorityMeshQueue>();
}

/// 把方块编辑涉及的本区块和边界邻居加入交互优先队列。
pub fn collect_priority_mesh_rebuilds(
    mut changes: MessageReader<BlockChangedEvent>,
    mut queue: ResMut<PriorityMeshQueue>,
) {
    for change in changes.read() {
        let affected = affected_mesh_chunks(change.world_pos);
        for position in affected.into_iter().rev() {
            queue.prioritize(position);
        }
    }
}

/// 内容注册表变化时重建后台网格任务使用的属性快照。
pub fn rebuild_block_info_snapshot(
    registry: Res<BlockRegistry>,
    mut cached: ResMut<CachedBlockInfo>,
) {
    if registry.is_changed() {
        cached.0 = BlockInfoSnapshot::from_registry(&registry);
    }
}

/// 按可见窗口和帧预算派发区块网格后台任务。
/// 网格任务派发同时受流送、任务池和区块生命周期约束，资源访问保持显式。
#[allow(clippy::too_many_arguments)]
pub fn spawn_mesh_build_tasks(
    channel: Res<MeshBuildChannel>,
    registry: Option<Res<BlockRegistry>>,
    world_state: Res<WorldState>,
    cached_block_info: Res<CachedBlockInfo>,
    task: Res<TaskManager>,
    player_cache: Res<PlayerChunkCache>,
    streaming_config: Res<WorldStreamingConfig>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
    chunk_runtime: Res<ChunkRuntime>,
    mut request_tracker: ResMut<MeshRequestTracker>,
    mut priority_queue: ResMut<PriorityMeshQueue>,
    world_lighting: Option<Res<WorldLighting>>,
) {
    if registry.is_none() {
        return;
    }
    let Some(player_chunk_pos) = player_cache.player_chunk_pos() else {
        return;
    };

    let block_info = cached_block_info.0.clone();
    let mut spawned = 0u32;
    let worker_budget = task.worker_count().max(1);
    let max_in_flight = worker_budget.clamp(2, 8);
    request_tracker.retain_runtime_entities(&chunk_runtime);

    let urgent_attempts = priority_queue.len();
    for _ in 0..urgent_attempts {
        if spawned >= MAX_MESH_TASKS_PER_FRAME
            || channel.in_flight.load(Ordering::Relaxed) >= max_in_flight
            || task.pending_count() > worker_budget
        {
            break;
        }
        let Some(position) = priority_queue.pop_front() else {
            break;
        };
        match spawn_mesh_for_position(
            position,
            player_chunk_pos,
            true,
            &channel,
            &world_state,
            &block_info,
            &task,
            &streaming_config,
            &mut chunk_query,
            &chunk_runtime,
            world_lighting.as_deref(),
            &mut request_tracker,
        ) {
            MeshSpawnAttempt::Spawned => spawned += 1,
            MeshSpawnAttempt::Retry => priority_queue.enqueue(position),
            MeshSpawnAttempt::Drop => {}
        }
    }

    let background_budget = if priority_queue.is_empty() {
        worker_budget
    } else {
        worker_budget.saturating_sub(1).max(1)
    };
    for &position in player_cache.ordered_chunks() {
        if spawned >= MAX_MESH_TASKS_PER_FRAME
            || channel.in_flight.load(Ordering::Relaxed) >= max_in_flight
            || task.pending_count() >= background_budget
        {
            break;
        }
        if matches!(
            spawn_mesh_for_position(
                position,
                player_chunk_pos,
                false,
                &channel,
                &world_state,
                &block_info,
                &task,
                &streaming_config,
                &mut chunk_query,
                &chunk_runtime,
                world_lighting.as_deref(),
                &mut request_tracker,
            ),
            MeshSpawnAttempt::Spawned
        ) {
            spawned += 1;
        }
    }
}

// 调度边界必须同时校验流式、实体、光照和任务版本状态，拆成参数对象会模糊借用范围。
#[allow(clippy::too_many_arguments)]
fn spawn_mesh_for_position(
    position: IVec3,
    player_chunk: IVec3,
    allow_pending_neighbor_light: bool,
    channel: &MeshBuildChannel,
    world: &WorldState,
    block_info: &BlockInfoSnapshot,
    task: &TaskManager,
    streaming: &WorldStreamingConfig,
    chunk_query: &mut Query<(&ChunkComponents, &mut ChunkState)>,
    runtime: &ChunkRuntime,
    lighting: Option<&WorldLighting>,
    requests: &mut MeshRequestTracker,
) -> MeshSpawnAttempt {
    if !streaming.should_mesh_chunk(player_chunk, position) {
        return MeshSpawnAttempt::Drop;
    }
    let Some(chunk_entity) = runtime.chunk_entity(position) else {
        return MeshSpawnAttempt::Drop;
    };
    let Ok((components, mut state)) = chunk_query.get_mut(chunk_entity) else {
        return MeshSpawnAttempt::Drop;
    };
    if components.position != position {
        return MeshSpawnAttempt::Drop;
    }
    if *state != ChunkState::LightingReady {
        return MeshSpawnAttempt::Retry;
    }
    let Some(current_chunk_data) = world.chunk(position) else {
        return MeshSpawnAttempt::Retry;
    };
    let Some(lighting) = lighting else {
        return MeshSpawnAttempt::Retry;
    };
    if !lighting.is_chunk_light_current(position, current_chunk_data) {
        *state = ChunkState::LightingPending;
        return MeshSpawnAttempt::Retry;
    }
    if !voxel_neighbors_ready(world, position) {
        return MeshSpawnAttempt::Retry;
    }
    if !neighbor_lights_allow_mesh(
        allow_pending_neighbor_light,
        lighting,
        world,
        streaming,
        player_chunk,
        position,
    ) {
        return MeshSpawnAttempt::Retry;
    }

    let current_data = Arc::clone(current_chunk_data);
    let neighbors: [Option<Arc<ChunkData>>; 6] =
        DIRECTIONS.map(|(direction, _)| world.chunk(position + direction).map(Arc::clone));
    let light = current_light_snapshot(Some(lighting), position);
    let neighbor_lights = DIRECTIONS
        .map(|(direction, _)| current_light_snapshot(Some(lighting), position + direction));
    let request_id = requests.begin(position, chunk_entity);
    let sender = channel.sender.clone();
    let in_flight = Arc::clone(&channel.in_flight);
    let input = MeshBuildInput {
        chunk_pos: position,
        request_entity: chunk_entity,
        request_id,
        current_data,
        neighbors,
        block_info: block_info.clone(),
        light,
        neighbor_lights,
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
    MeshSpawnAttempt::Spawned
}

fn voxel_neighbors_ready(world: &WorldState, chunk_pos: IVec3) -> bool {
    DIRECTIONS
        .iter()
        .all(|(direction, _)| world.contains_chunk(chunk_pos + *direction))
}

/// 可见窗口内部的邻居光照必须先与权威体素快照同步，外圈数据邻居仅用于封边。
fn visible_neighbor_lights_ready(
    lighting: &WorldLighting,
    world: &WorldState,
    streaming: &WorldStreamingConfig,
    player_chunk: IVec3,
    position: IVec3,
) -> bool {
    DIRECTIONS.iter().all(|(direction, _)| {
        let neighbor = position + *direction;
        if !streaming.should_mesh_chunk(player_chunk, neighbor) {
            return true;
        }
        world
            .chunk(neighbor)
            .is_some_and(|data| lighting.is_chunk_light_current(neighbor, data))
    })
}

/// 编辑网格允许用中性临时光先渲染；后台流送仍等待可见邻居的权威光照。
fn neighbor_lights_allow_mesh(
    allow_pending_neighbor_light: bool,
    lighting: &WorldLighting,
    world: &WorldState,
    streaming: &WorldStreamingConfig,
    player_chunk: IVec3,
    position: IVec3,
) -> bool {
    allow_pending_neighbor_light
        || visible_neighbor_lights_ready(lighting, world, streaming, player_chunk, position)
}

fn current_light_snapshot(
    lighting: Option<&WorldLighting>,
    position: IVec3,
) -> Option<Arc<ChunkLight>> {
    let lighting = lighting?;
    lighting
        .chunk_lights
        .get(&position)
        .filter(|light| light.is_initialized())
        .cloned()
}

fn affected_mesh_chunks(world_pos: IVec3) -> Vec<IVec3> {
    let chunk = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE as i32),
        world_pos.y.div_euclid(CHUNK_SIZE as i32),
        world_pos.z.div_euclid(CHUNK_SIZE as i32),
    );
    let local = IVec3::new(
        world_pos.x.rem_euclid(CHUNK_SIZE as i32),
        world_pos.y.rem_euclid(CHUNK_SIZE as i32),
        world_pos.z.rem_euclid(CHUNK_SIZE as i32),
    );
    let mut affected = vec![chunk];
    let max = CHUNK_SIZE as i32 - 1;
    for (axis, negative, positive) in [
        (local.x, IVec3::NEG_X, IVec3::X),
        (local.y, IVec3::NEG_Y, IVec3::Y),
        (local.z, IVec3::NEG_Z, IVec3::Z),
    ] {
        if axis == 0 {
            affected.push(chunk + negative);
        } else if axis == max {
            affected.push(chunk + positive);
        }
    }
    affected
}

/// 离开世界时清空交互队列和飞行中请求身份；旧结果会因身份缺失被拒绝。
pub fn clear_mesh_lifecycle(
    mut queue: ResMut<PriorityMeshQueue>,
    mut requests: ResMut<MeshRequestTracker>,
) {
    queue.clear();
    requests.active.clear();
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/world/mesh_lifecycle.rs"]
mod tests;

/// 在渲染主线程接收网格结果并更新区块表现实体。
pub fn receive_mesh_results(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    channel: Res<MeshBuildChannel>,
    render_assets: Option<Res<BlockRenderAssets>>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
    chunk_runtime: Res<ChunkRuntime>,
    mut request_tracker: ResMut<MeshRequestTracker>,
) {
    let Some(render_assets) = render_assets else {
        return;
    };
    let opaque_mat = render_assets.voxel_opaque_material().clone();
    let cutout_mat = render_assets.voxel_cutout_material().clone();
    let water_base_mat = render_assets.water_base_material().clone();
    let water_effect_mat = render_assets.water_effect_material().clone();

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

        if !request_tracker.is_current(result.chunk_pos, result.request_entity, result.request_id) {
            continue;
        }
        let chunk_entity = result.request_entity;
        if chunk_runtime.chunk_entity(result.chunk_pos) != Some(chunk_entity) {
            request_tracker.finish(result.chunk_pos, chunk_entity, result.request_id);
            continue;
        }
        let Ok((_components, mut state)) = chunk_query.get_mut(chunk_entity) else {
            request_tracker.finish(result.chunk_pos, chunk_entity, result.request_id);
            continue;
        };
        if *state != ChunkState::GeneratingMesh {
            request_tracker.finish(result.chunk_pos, chunk_entity, result.request_id);
            continue;
        }

        commands
            .entity(chunk_entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity
                    .remove::<Mesh3d>()
                    .remove::<MeshMaterial3d<StandardMaterial>>()
                    .remove::<MeshMaterial3d<VoxelMaterial>>();
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
            let water_mesh = meshes.add(result.water.build_mesh_plain());
            let base_child = commands
                .spawn((
                    Mesh3d(water_mesh.clone()),
                    MeshMaterial3d(water_base_mat.clone()),
                    Transform::IDENTITY,
                    Visibility::default(),
                ))
                .id();
            let effect_child = commands
                .spawn((
                    Mesh3d(water_mesh),
                    MeshMaterial3d(water_effect_mat.clone()),
                    Transform::IDENTITY,
                    Visibility::default(),
                ))
                .id();
            commands
                .entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.add_child(base_child);
                    entity.add_child(effect_child);
                });
        }

        *state = ChunkState::Rendered;
        request_tracker.finish(result.chunk_pos, chunk_entity, result.request_id);
    }
}
