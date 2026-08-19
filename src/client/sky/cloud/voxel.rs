//! Voxel 块状云渲染：把 `generation` 生成的几何场落成真实实体。
//!
//! 几何生成（云团分级、密度场、三级角色）在 `super::generation`，本模块只
//! 负责渲染侧：三级尺寸共享 Mesh、四档冷白/灰材质、实体 spawn 与清理。
//!
//! 与原 raymarching 系统并行存在但不注册：cloud/material.rs、cloud/systems.rs
//! 的原版代码未被 plugin 引用，因此球体不被 spawn、WGSL 不被加载。

use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

use super::components::CloudWeatherState;
use super::generation::{
    BOTTOM_SHADE_BAND, CLOUD_CELL_SIZE, CLOUD_RANGE, CORE_CELL_SIZE, CloudCluster, CloudTier,
    EDGE_CELL_SIZE, EDGE_DENSITY_THRESHOLD, ENVELOPE_SCALE, VoxelRole, cluster_envelopes,
    density_at, generate_clusters, sample_voxel,
};
use crate::client::camera::FpsCamera;
use crate::content::cloud::CloudRegistry;

/// 运行时资源：跟踪 voxel cloud 已生成的实体和共享材质/Mesh 句柄。
#[derive(Resource, Default)]
pub struct CloudVoxelRuntime {
    /// 已生成的 cloud cube 实体列表。
    pub entities: Vec<Entity>,
    /// 三级尺寸共享 mesh（中心大、边缘小）。
    pub meshes: CloudVoxelMeshes,
    /// 四档色彩材质。
    pub materials: CloudVoxelMaterials,
}

/// 三级尺寸的共享 cube Mesh 句柄。
#[derive(Default)]
pub struct CloudVoxelMeshes {
    /// 核心层 8m cube。
    pub core: Option<Handle<Mesh>>,
    /// 鼓包层 6m cube。
    pub lobe: Option<Handle<Mesh>>,
    /// 外壳层 4m cube。
    pub edge: Option<Handle<Mesh>>,
}

impl CloudVoxelMeshes {
    fn reset(&mut self) {
        self.core = None;
        self.lobe = None;
        self.edge = None;
    }
}

/// 顶/鼓包/外壳/底四档使用的 StandardMaterial 句柄。
#[derive(Default)]
pub struct CloudVoxelMaterials {
    /// 核心亮白（略暖）#F2F3F1。
    pub top: Option<Handle<StandardMaterial>>,
    /// 鼓包主体中性浅灰 #D8DDE3。
    pub mid: Option<Handle<StandardMaterial>>,
    /// 外壳中性灰蓝 #BEC7D2。
    pub side: Option<Handle<StandardMaterial>>,
    /// 底部冷深灰 #9DA8B5。
    pub bot: Option<Handle<StandardMaterial>>,
}

impl CloudVoxelMaterials {
    fn reset(&mut self) {
        self.top = None;
        self.mid = None;
        self.side = None;
        self.bot = None;
    }
}

/// 构造一档云层 StandardMaterial：PBR 受光（unlit=false）、roughness=1、
/// reflectance=0.05（云几乎无金属反射）、Fog 启用。
///
/// 作为 free function 而非闭包，避免 system 内多个 `get_or_insert_with`
/// 闭包互相借用 `Assets<StandardMaterial>` 引发冲突。
fn make_cloud_material(
    materials: &mut Assets<StandardMaterial>,
    base_color: Color,
    emissive: LinearRgba,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color,
        // 弱冷色自发光：只提亮背光面、避免底部"死黑"，同时保留方向光
        // 带来的顶/底亮度差异。
        emissive,
        perceptual_roughness: 1.0,
        reflectance: 0.05,
        unlit: false,
        cull_mode: Some(Face::Back),
        alpha_mode: AlphaMode::Opaque,
        fog_enabled: true,
        ..default()
    })
}

/// 退出游戏时清理全部 voxel cloud 实体，释放 Mesh / Material。
pub fn cleanup_voxel_cloud_system(mut commands: Commands, mut runtime: ResMut<CloudVoxelRuntime>) {
    for entity in runtime.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    runtime.meshes.reset();
    runtime.materials.reset();
}

/// 进入游戏状态、内容注册表就绪后程序生成 voxel 云场。
///
/// 锚点：进入游戏瞬间的玩家相机位置。玩家在 ±RANGE 范围内能看到云；
/// 超出范围后云不可见（V1 简化，后续可加 cluster streaming）。
/// 参数多为 Bevy system 注入（资源/查询），豁免 too_many_arguments。
#[allow(clippy::too_many_arguments)]
pub fn setup_voxel_cloud_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cloud_registry: Res<CloudRegistry>,
    weather_cell: Option<Res<crate::game::world::weather::WeatherCell>>,
    cloud_state: Res<CloudWeatherState>,
    camera_query: Query<&GlobalTransform, With<FpsCamera>>,
    mut runtime: ResMut<CloudVoxelRuntime>,
) {
    // 防御性清理：drain 已有实体防止与旧实例叠加，再复位运行时字段。
    for entity in runtime.entities.drain(..) {
        commands.entity(entity).despawn();
    }
    runtime.meshes.reset();
    runtime.materials.reset();

    let Some(definition) = cloud_registry.primary() else {
        log::warn!("[voxel-cloud] no cloud definition available; skipping");
        return;
    };
    let Some(layer) = definition.layers.first() else {
        log::warn!("[voxel-cloud] cloud definition has no layers; skipping");
        return;
    };

    // 优先读 Game 层权威天气（含 cloud_water 初始值），缺失时退回表现状态默认。
    let coverage = weather_cell
        .map(|cell| cell.cloud_water.clamp(0.0, 1.0))
        .unwrap_or_else(|| cloud_state.normalized().coverage);
    let camera_pos = camera_query
        .iter()
        .next()
        .map(|t| t.translation())
        .unwrap_or(Vec3::ZERO);

    // 三级尺寸共享 mesh：核心 8m / 鼓包 6m / 外壳 4m。
    let mesh_core: Handle<Mesh> = runtime
        .meshes
        .core
        .get_or_insert_with(|| {
            meshes.add(Cuboid::new(CORE_CELL_SIZE, CORE_CELL_SIZE, CORE_CELL_SIZE))
        })
        .clone();
    let mesh_lobe: Handle<Mesh> = runtime
        .meshes
        .lobe
        .get_or_insert_with(|| {
            meshes.add(Cuboid::new(
                CLOUD_CELL_SIZE,
                CLOUD_CELL_SIZE,
                CLOUD_CELL_SIZE,
            ))
        })
        .clone();
    let mesh_edge: Handle<Mesh> = runtime
        .meshes
        .edge
        .get_or_insert_with(|| {
            meshes.add(Cuboid::new(EDGE_CELL_SIZE, EDGE_CELL_SIZE, EDGE_CELL_SIZE))
        })
        .clone();

    // 四档冷白/灰材质：核心亮白、鼓包主体、外壳灰蓝、底部冷深灰。
    let mat_top: Handle<StandardMaterial> = runtime
        .materials
        .top
        .get_or_insert_with(|| {
            make_cloud_material(
                &mut materials,
                Color::srgb(0.949, 0.953, 0.945), // #F2F3F1（略暖白）
                LinearRgba::new(0.24, 0.26, 0.28, 1.0),
            )
        })
        .clone();
    let mat_mid: Handle<StandardMaterial> = runtime
        .materials
        .mid
        .get_or_insert_with(|| {
            make_cloud_material(
                &mut materials,
                Color::srgb(0.847, 0.867, 0.890), // #D8DDE3（中性浅灰）
                LinearRgba::new(0.14, 0.16, 0.19, 1.0),
            )
        })
        .clone();
    let mat_side: Handle<StandardMaterial> = runtime
        .materials
        .side
        .get_or_insert_with(|| {
            make_cloud_material(
                &mut materials,
                Color::srgb(0.745, 0.780, 0.824), // #BEC7D2（中性灰蓝）
                LinearRgba::new(0.09, 0.11, 0.14, 1.0),
            )
        })
        .clone();
    let mat_bot: Handle<StandardMaterial> = runtime
        .materials
        .bot
        .get_or_insert_with(|| {
            make_cloud_material(
                &mut materials,
                Color::srgb(0.616, 0.659, 0.710), // #9DA8B5（冷深灰）
                LinearRgba::new(0.05, 0.07, 0.10, 1.0),
            )
        })
        .clone();

    // 分级撒点生成 Mega Cluster（核心 + 4~8 鼓包）。
    let base_height = layer.height;
    let seed = definition.seed;
    let clusters = generate_clusters(base_height, coverage, seed, camera_pos);

    // 计算生成 AABB：所有 cluster 的核心与鼓包包络（×ENVELOPE_SCALE）的并集，
    // 再裁剪到相机周围 ±CLOUD_RANGE。
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for cluster in &clusters {
        for (center, radius) in cluster_envelopes(cluster) {
            min = min.min(center - radius * ENVELOPE_SCALE);
            max = max.max(center + radius * ENVELOPE_SCALE);
        }
    }
    let cam_min = camera_pos - Vec3::splat(CLOUD_RANGE);
    let cam_max = camera_pos + Vec3::splat(CLOUD_RANGE);
    let min = min.max(cam_min);
    let max = max.min(cam_max);
    if min.x > max.x || min.y > max.y || min.z > max.z {
        log::info!("[voxel-cloud] cloud AABB empty; skipping");
        return;
    }

    // 三层遍历：核心 8m → 鼓包 6m → 外壳 4m。各层只生成自己的角色，
    // 位置互斥（sample_voxel 保证每个点只有一种角色），尺寸不随机混合。
    let spawned_core = spawn_voxel_layer(
        &mut commands,
        &clusters,
        min,
        max,
        CORE_CELL_SIZE,
        VoxelRole::Core,
        &mesh_core,
        &mat_top,
        &mat_bot,
        seed,
        &mut runtime,
    );
    let spawned_lobe = spawn_voxel_layer(
        &mut commands,
        &clusters,
        min,
        max,
        CLOUD_CELL_SIZE,
        VoxelRole::Lobe,
        &mesh_lobe,
        &mat_mid,
        &mat_bot,
        seed,
        &mut runtime,
    );
    let spawned_edge = spawn_voxel_layer(
        &mut commands,
        &clusters,
        min,
        max,
        EDGE_CELL_SIZE,
        VoxelRole::Edge,
        &mesh_edge,
        &mat_side,
        &mat_bot,
        seed,
        &mut runtime,
    );
    let total = spawned_core + spawned_lobe + spawned_edge;

    // 统计各等级云团数量（读取 tier，同时让日志呈现天空纵深分布）。
    let large = clusters
        .iter()
        .filter(|c| c.tier == CloudTier::Large)
        .count();
    let medium = clusters
        .iter()
        .filter(|c| c.tier == CloudTier::Medium)
        .count();
    let small = clusters
        .iter()
        .filter(|c| c.tier == CloudTier::Small)
        .count();

    log::info!(
        "[voxel-cloud] spawned {total} cubes (core {spawned_core} / lobe {spawned_lobe} / edge {spawned_edge}); coverage={coverage:.2}, clusters={} (large {large} / medium {medium} / small {small})",
        clusters.len(),
    );
}

/// 遍历一个 voxel 层：按 cell 尺寸在 AABB 内步进，只生成指定角色。
///
/// - Core/Lobe：实心核（归一化半径 < `SOLID_CORE_R`）直接生成。
/// - Edge：包络内且密度高于阈值才生成（外壳薄层），形成不规则下垂/阶梯。
///
/// 生成的实体登记进 `runtime.entities`（供退出清理），返回本层 spawn 数量。
/// 参数较多是因为三级层共享同一遍历骨架；豁免 too_many_arguments。
#[allow(clippy::too_many_arguments)]
fn spawn_voxel_layer(
    commands: &mut Commands,
    clusters: &[CloudCluster],
    min: Vec3,
    max: Vec3,
    cell: f32,
    role: VoxelRole,
    mesh: &Handle<Mesh>,
    main_mat: &Handle<StandardMaterial>,
    bot_mat: &Handle<StandardMaterial>,
    seed: u32,
    runtime: &mut CloudVoxelRuntime,
) -> u32 {
    let steps_x = ((max.x - min.x) / cell).ceil() as i32;
    let steps_y = ((max.y - min.y) / cell).ceil() as i32;
    let steps_z = ((max.z - min.z) / cell).ceil() as i32;
    let mut spawned = 0u32;

    for ix in 0..=steps_x {
        let x = min.x + ix as f32 * cell + cell * 0.5;
        for iz in 0..=steps_z {
            let z = min.z + iz as f32 * cell + cell * 0.5;
            for iy in 0..=steps_y {
                let y = min.y + iy as f32 * cell + cell * 0.5;
                let p = Vec3::new(x, y, z);

                let Some(sample) = sample_voxel(p, clusters) else {
                    continue;
                };
                if sample.role != role {
                    continue;
                }

                // 外壳需要密度阈值（过滤外缘碎屑）；核心/鼓包是实心核直接生成。
                if role == VoxelRole::Edge
                    && density_at(p, clusters, seed) <= EDGE_DENSITY_THRESHOLD
                {
                    continue;
                }

                // 底部统一冷深灰；其余按角色选主色。
                let mat = if p.y < sample.core_bottom + BOTTOM_SHADE_BAND {
                    bot_mat
                } else {
                    main_mat
                };

                let entity = commands
                    .spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_translation(p),
                        Visibility::default(),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ))
                    .id();
                runtime.entities.push(entity);
                spawned += 1;
            }
        }
    }
    spawned
}

/// 在天气云量变化时刷新云场（V2 待实现：调整密度阈值重 spawn）。
#[allow(dead_code)]
pub fn refresh_voxel_cloud_on_coverage_change(
    _commands: Commands,
    _runtime: ResMut<CloudVoxelRuntime>,
    _weather: Res<CloudWeatherState>,
) {
    // V2 占位：MVP 把云量锁定在 spawn 时刻，未来用 hash + 重密度切换 cube 可见性。
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/sky/cloud/voxel.rs"]
mod tests;
