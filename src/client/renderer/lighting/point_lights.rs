//! 把邻近发光方块映射为有严格数量预算的 Bevy 点光源。

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::game::player::identity::LocalPlayer;
use crate::game::world::lighting::WorldLighting;
use crate::game::world::lighting::rebuild::BlockLightSource;

/// 同时存在的实体点光上限；体素 RGB 光不受此表现预算影响。
const MAX_ACTIVE_POINT_LIGHTS: usize = 24;
/// 阴影点光需要六面阴影贴图，因此只保留离玩家最近的一小部分。
const MAX_SHADOWED_POINT_LIGHTS: usize = 6;
/// 超出该距离的实体点光不会改善当前画面，直接由体素光场承担低频照明。
const MAX_POINT_LIGHT_DISTANCE: f32 = 40.0;
/// 满光级方块映射到 Bevy 点光的玩法标定流明数。
const MAX_BLOCK_LIGHT_LUMENS: f32 = 16_384.0;

/// 标记由方块光表现池创建的 Bevy 点光实体。
#[derive(Component)]
struct BlockPointLight;

/// 客户端点光实体缓存；只跟踪表现实体，不拥有权威光源数据。
#[derive(Resource, Default)]
pub(super) struct BlockPointLightCache {
    last_lighting_revision: Option<u64>,
    last_player_voxel: Option<IVec3>,
    entities: HashMap<IVec3, Entity>,
}

/// 在玩家跨格或权威光场重建后同步最近的点光源集合。
pub(super) fn sync_block_point_lights(
    mut commands: Commands,
    lighting: Res<WorldLighting>,
    player_query: Query<&GlobalTransform, With<LocalPlayer>>,
    mut cache: ResMut<BlockPointLightCache>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_position = player_transform.translation();
    let player_voxel = player_position.floor().as_ivec3();
    if cache.last_lighting_revision == Some(lighting.revision)
        && cache.last_player_voxel == Some(player_voxel)
    {
        return;
    }
    cache.last_lighting_revision = Some(lighting.revision);
    cache.last_player_voxel = Some(player_voxel);

    let max_distance_squared = MAX_POINT_LIGHT_DISTANCE * MAX_POINT_LIGHT_DISTANCE;
    let mut candidates = lighting
        .sources
        .iter()
        .copied()
        .filter_map(|source| {
            let center = source_center(source);
            let distance_squared = center.distance_squared(player_position);
            (distance_squared <= max_distance_squared).then_some((distance_squared, source))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|(left_distance, left), (right_distance, right)| {
        left_distance
            .total_cmp(right_distance)
            .then_with(|| source_key(*left).cmp(&source_key(*right)))
    });
    candidates.truncate(MAX_ACTIVE_POINT_LIGHTS);

    let desired_positions = candidates
        .iter()
        .map(|(_, source)| source.world_pos)
        .collect::<HashSet<_>>();
    cache.entities.retain(|position, entity| {
        if desired_positions.contains(position) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });

    let mut shadowed = 0usize;
    for (_, source) in candidates {
        let shadows_enabled = source.light.casts_shadow && shadowed < MAX_SHADOWED_POINT_LIGHTS;
        if shadows_enabled {
            shadowed += 1;
        }
        let bundle = (
            BlockPointLight,
            Name::new("BlockPointLight"),
            point_light(source, shadows_enabled),
            Transform::from_translation(source_center(source)),
        );
        if let Some(entity) = cache.entities.get(&source.world_pos).copied() {
            commands.entity(entity).insert(bundle);
        } else {
            let entity = commands.spawn(bundle).id();
            cache.entities.insert(source.world_pos, entity);
        }
    }
}

/// 离开世界时清理会话期点光，避免下个存档继承旧表现实体。
pub(super) fn cleanup_block_point_lights(
    mut commands: Commands,
    mut cache: ResMut<BlockPointLightCache>,
) {
    for entity in cache.entities.drain().map(|(_, entity)| entity) {
        commands.entity(entity).despawn();
    }
    cache.last_lighting_revision = None;
    cache.last_player_voxel = None;
}

fn point_light(source: BlockLightSource, shadows_enabled: bool) -> PointLight {
    let strength = source.light.emission.min(15) as f32 / 15.0;
    PointLight {
        color: Color::linear_rgb(
            source.light.color[0],
            source.light.color[1],
            source.light.color[2],
        ),
        intensity: MAX_BLOCK_LIGHT_LUMENS * strength,
        range: source.light.range.max(1) as f32 + 0.5,
        radius: 0.12,
        shadow_maps_enabled: shadows_enabled,
        contact_shadows_enabled: shadows_enabled,
        shadow_depth_bias: 0.06,
        shadow_normal_bias: 0.8,
        shadow_map_near_z: 0.08,
        ..default()
    }
}

fn source_center(source: BlockLightSource) -> Vec3 {
    source.world_pos.as_vec3() + Vec3::splat(0.5)
}

fn source_key(source: BlockLightSource) -> (i32, i32, i32) {
    (source.world_pos.x, source.world_pos.y, source.world_pos.z)
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/lighting/point_lights.rs"]
mod tests;
