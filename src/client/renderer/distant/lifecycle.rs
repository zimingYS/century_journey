//! 管理远景真实方块 LOD 瓦片的计划、异步构建、上传与世界会话清理。

use super::block_mesh::build_distant_block_mesh;
use super::channel::{DistantTerrainBuildChannel, DistantTerrainBuildResult};
use super::config::DistantTerrainConfig;
use super::planner::{DistantTerrainTileKey, DistantTerrainTileSpec, build_distant_terrain_plan};
use crate::client::camera::FpsCamera;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::renderer::world::CachedBlockInfo;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::world::generation::pipeline::TerrainSurfaceSampler;
use crate::game::world::streaming::{PlayerChunkCache, WorldStreamingConfig};
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::pbr::AtmosphereSettings;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// 单帧最多派发的远景方块 LOD 任务数。
///
/// 远景永远让位给权威区块加载和近景网格，因此这个预算显著低于近景网格任务预算。
const MAX_DISTANT_TERRAIN_TASKS_PER_FRAME: usize = 2;
/// 单帧最多接收并上传的远景方块网格数，避免大量任务同时完成时造成渲染帧尖峰。
const MAX_DISTANT_TERRAIN_RESULTS_PER_FRAME: usize = 4;

/// 已上传远景方块 LOD 瓦片的 ECS 标记。
///
/// 它只标识 Client 的临时表现实体，不参与方块、碰撞、光照、保存或任何权威规则。
#[derive(Component)]
struct DistantTerrainTile;

/// 远景瓦片运行时的请求身份和可见实体索引。
///
/// `session_generation` 在每次世界会话切换时递增，使已经离开世界后才完成的后台
/// 任务无法提交到新世界；暂停恢复保留同一会话，`request_id` 则处理同一瓦片键的
/// 乱序结果。
#[derive(Resource, Default)]
pub(crate) struct DistantTerrainRuntime {
    session_generation: u64,
    next_request_id: u64,
    expected_keys: HashSet<DistantTerrainTileKey>,
    ordered_plan: Vec<DistantTerrainTileSpec>,
    active_requests: HashMap<DistantTerrainTileKey, u64>,
    tile_entities: HashMap<DistantTerrainTileKey, Entity>,
    /// 每个已构建瓦片当前使用的覆盖位图；玩家移动导致位图变化时原地更新网格，
    /// 而不是销毁并重建瓦片实体。
    tile_masks: HashMap<DistantTerrainTileKey, [u64; 4]>,
    last_player_chunk: Option<IVec3>,
    last_near_radius_chunks: Option<i32>,
    last_config: Option<DistantTerrainConfig>,
}

impl DistantTerrainRuntime {
    fn begin_request(&mut self, key: DistantTerrainTileKey) -> u64 {
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        let request_id = self.next_request_id;
        self.active_requests.insert(key, request_id);
        request_id
    }

    fn accepts(&self, result: &DistantTerrainBuildResult) -> bool {
        result.session_generation == self.session_generation
            && self.expected_keys.contains(&result.key)
            && self.active_requests.get(&result.key) == Some(&result.request_id)
    }

    fn clear_plan(&mut self) {
        self.expected_keys.clear();
        self.ordered_plan.clear();
        self.active_requests.clear();
        self.tile_masks.clear();
        self.last_player_chunk = None;
        self.last_near_radius_chunks = None;
        self.last_config = None;
    }

    /// 推进世界会话版本并撤销所有旧瓦片请求。
    ///
    /// 共享材质不随世界切换销毁；只有请求身份必须失效，避免后台任务把旧种子的
    /// 真实方块 LOD 提交到新世界。
    fn advance_session(&mut self) {
        self.session_generation = self.session_generation.wrapping_add(1).max(1);
        self.clear_plan();
    }
}

/// 确保远景渲染器拥有共享材质并初始化首个世界会话。
pub(crate) fn initialize_distant_terrain_system(mut runtime: ResMut<DistantTerrainRuntime>) {
    // 暂停恢复同样会进入 `InGame`，但必须保留已经完成的远景瓦片和请求身份。
    if runtime.session_generation == 0 {
        runtime.advance_session();
    }
}

/// 根据权威近景网格半径更新远景方块瓦片计划。
///
/// 该系统运行在普通渲染帧，并显式排在区块流送之后；它只读取玩家区块缓存，绝不
/// 向 `WorldState` 请求额外区块，因此相机移动不会扩大存档或模拟窗口。
pub(crate) fn sync_distant_terrain_plan_system(
    mut commands: Commands,
    player_cache: Res<PlayerChunkCache>,
    streaming: Res<WorldStreamingConfig>,
    config: Res<DistantTerrainConfig>,
    sampler: Res<TerrainSurfaceSampler>,
    mut runtime: ResMut<DistantTerrainRuntime>,
) {
    let Some(player_chunk) = player_cache.player_chunk_pos() else {
        return;
    };
    let near_radius_chunks = streaming.mesh_horizontal_radius.max(1);
    let sampler_changed = sampler.is_changed();
    if !sampler_changed
        && runtime.last_player_chunk == Some(player_chunk)
        && runtime.last_near_radius_chunks == Some(near_radius_chunks)
        && runtime.last_config.as_ref() == Some(config.as_ref())
    {
        return;
    }

    if sampler_changed {
        for entity in runtime.tile_entities.drain().map(|(_, entity)| entity) {
            commands
                .entity(entity)
                .despawn_related::<Children>()
                .despawn();
        }
        runtime.advance_session();
    }

    let ordered_plan = build_distant_terrain_plan(player_chunk, near_radius_chunks, &config);
    let expected_keys = ordered_plan
        .iter()
        .map(|spec| spec.key)
        .collect::<HashSet<_>>();
    let removed_keys = runtime
        .expected_keys
        .difference(&expected_keys)
        .copied()
        .collect::<Vec<_>>();
    for key in removed_keys {
        runtime.active_requests.remove(&key);
        runtime.tile_masks.remove(&key);
        if let Some(entity) = runtime.tile_entities.remove(&key) {
            commands
                .entity(entity)
                .despawn_related::<Children>()
                .despawn();
        }
    }

    runtime.expected_keys = expected_keys;
    runtime.ordered_plan = ordered_plan;
    runtime.last_player_chunk = Some(player_chunk);
    runtime.last_near_radius_chunks = Some(near_radius_chunks);
    runtime.last_config = Some(config.clone());
}

/// 在近景任务预算空闲时派发远景方块网格构建。
pub(crate) fn spawn_distant_terrain_tasks_system(
    channel: Res<DistantTerrainBuildChannel>,
    sampler: Res<TerrainSurfaceSampler>,
    block_info: Res<CachedBlockInfo>,
    task: Res<TaskManager>,
    mut runtime: ResMut<DistantTerrainRuntime>,
) {
    if !sampler.is_ready() || block_info.0.max_id == 0 {
        return;
    }

    let worker_count = task.worker_count().max(1);
    let max_in_flight = worker_count.saturating_sub(1).clamp(1, 2);
    let planned_specs = runtime.ordered_plan.clone();
    let mut spawned = 0usize;

    for spec in planned_specs {
        if spawned >= MAX_DISTANT_TERRAIN_TASKS_PER_FRAME
            || channel.in_flight.load(Ordering::Relaxed) >= max_in_flight
            || task.pending_count() >= worker_count
        {
            break;
        }
        if !runtime.expected_keys.contains(&spec.key)
            || runtime.active_requests.contains_key(&spec.key)
        {
            continue;
        }
        // 已有实体且覆盖位图未变化时无需重建；位图变化则原地更新网格而非销毁瓦片。
        if runtime.tile_entities.contains_key(&spec.key)
            && runtime.tile_masks.get(&spec.key) == Some(&spec.coverage_mask)
        {
            continue;
        }

        let request_id = runtime.begin_request(spec.key);
        let session_generation = runtime.session_generation;
        let coverage_mask = spec.coverage_mask;
        let sampler = sampler.clone();
        let block_info = block_info.0.clone();
        let sender = channel.sender.clone();
        let in_flight = Arc::clone(&channel.in_flight);
        channel.in_flight.fetch_add(1, Ordering::Relaxed);
        task.spawn_cpu(move || {
            let result = DistantTerrainBuildResult {
                session_generation,
                request_id,
                key: spec.key,
                coverage_mask,
                mesh: build_distant_block_mesh(&sampler, &block_info, spec),
            };
            if sender.send(result).is_err() {
                in_flight.fetch_sub(1, Ordering::Relaxed);
            }
            TaskResult::Success
        });
        spawned += 1;
    }
}

/// 回收仍属于当前会话的后台结果并创建远景方块表现实体。
pub(crate) fn receive_distant_terrain_results_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    render_assets: Option<Res<BlockRenderAssets>>,
    channel: Res<DistantTerrainBuildChannel>,
    mut runtime: ResMut<DistantTerrainRuntime>,
) {
    let Some(render_assets) = render_assets else {
        return;
    };
    let opaque_material = render_assets.voxel_opaque_material().clone();
    let water_base_material = render_assets.water_base_material().clone();
    let water_effect_material = render_assets.water_effect_material().clone();
    let receiver = channel.receiver.lock().unwrap();
    let mut received = 0usize;

    while received < MAX_DISTANT_TERRAIN_RESULTS_PER_FRAME {
        let Ok(result) = receiver.try_recv() else {
            break;
        };
        channel.in_flight.fetch_sub(1, Ordering::Relaxed);
        received += 1;

        if !runtime.accepts(&result) {
            continue;
        }
        runtime.active_requests.remove(&result.key);
        runtime.tile_masks.insert(result.key, result.coverage_mask);
        if let Some(previous) = runtime.tile_entities.remove(&result.key) {
            commands
                .entity(previous)
                .despawn_related::<Children>()
                .despawn();
        }

        let opaque_mesh =
            (!result.mesh.opaque.is_empty()).then(|| meshes.add(result.mesh.opaque.build_mesh()));
        let water_mesh = (!result.mesh.water.is_empty())
            .then(|| meshes.add(result.mesh.water.build_mesh_plain()));
        let key = result.key;
        let entity = commands
            .spawn((
                DistantTerrainTile,
                Name::new(format!(
                    "DistantTerrainLod{} ({}, {})",
                    key.lod_level, key.origin_chunk_x, key.origin_chunk_z
                )),
                NotShadowCaster,
                NotShadowReceiver,
                Visibility::default(),
                Transform::from_xyz(
                    key.origin_chunk_x as f32 * crate::shared::voxel::CHUNK_SIZE as f32,
                    0.0,
                    key.origin_chunk_z as f32 * crate::shared::voxel::CHUNK_SIZE as f32,
                ),
            ))
            .id();
        if let Some(mesh) = opaque_mesh {
            commands
                .entity(entity)
                .insert((Mesh3d(mesh), MeshMaterial3d(opaque_material.clone())));
        }
        if let Some(mesh) = water_mesh {
            let base_child = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(water_base_material.clone()),
                    Transform::IDENTITY,
                    Visibility::default(),
                ))
                .id();
            let effect_child = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(water_effect_material.clone()),
                    Transform::IDENTITY,
                    Visibility::default(),
                ))
                .id();
            commands
                .entity(entity)
                .add_children(&[base_child, effect_child]);
        }
        runtime.tile_entities.insert(key, entity);
    }
}

/// 扩展主相机裁剪与大气 LUT 距离，使远景网格不会在雾效前被裁掉。
pub(crate) fn sync_distant_terrain_camera_range_system(
    config: Res<DistantTerrainConfig>,
    streaming: Res<WorldStreamingConfig>,
    mut camera_query: Query<(&mut Projection, Option<&mut AtmosphereSettings>), With<FpsCamera>>,
) {
    let visible_distance = config.view_distance_blocks(streaming.mesh_horizontal_radius);
    for (mut projection, atmosphere) in &mut camera_query {
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.far = visible_distance;
        }
        if let Some(mut atmosphere) = atmosphere {
            atmosphere.aerial_view_lut_max_distance = visible_distance;
        }
    }
}

/// 切换或结束世界会话时销毁远景表现实体并使飞行中结果失效。
///
/// 这个系统不能绑定在 `OnExit(InGame)`：打开暂停菜单也会触发该状态退出，而暂停
/// 不应丢弃已经构建好的远景瓦片。
pub(crate) fn clear_distant_terrain_system(
    mut commands: Commands,
    mut runtime: ResMut<DistantTerrainRuntime>,
) {
    for entity in runtime.tile_entities.drain().map(|(_, entity)| entity) {
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .despawn();
    }
    runtime.advance_session();
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/distant/lifecycle.rs"]
mod tests;
