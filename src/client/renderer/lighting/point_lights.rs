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
/// 近景范围内保持完整 PBR 点光；超过后由稳定的体素方块光逐步接管。
const POINT_LIGHT_FADE_START_DISTANCE: f32 = 24.0;
/// 超出该距离的实体点光不会改善当前画面，直接由体素光场承担低频照明。
const MAX_POINT_LIGHT_DISTANCE: f32 = 40.0;
/// 满光级方块映射到 Bevy 点光的玩法标定流明数。
const MAX_BLOCK_LIGHT_LUMENS: f32 = 8_192.0;
/// 半格采样一次玩家位置，使远景点光衰减连续且不产生逐帧同步开销。
const PLAYER_LIGHT_LOD_SCALE: f32 = 2.0;

/// 标记由方块光表现池创建的 Bevy 点光实体。
#[derive(Component)]
struct BlockPointLight;

/// 客户端点光实体缓存；只跟踪表现实体，不拥有权威光源数据。
#[derive(Resource, Default)]
pub(super) struct BlockPointLightCache {
    last_lighting_revision: Option<u64>,
    last_player_lod_cell: Option<IVec3>,
    entities: HashMap<IVec3, Entity>,
}

/// 在玩家跨过半格 LOD 单元或权威光场重建后同步最近的点光源集合。
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
    let player_lod_cell = (player_position * PLAYER_LIGHT_LOD_SCALE)
        .floor()
        .as_ivec3();
    if cache.last_lighting_revision == Some(lighting.revision)
        && cache.last_player_lod_cell == Some(player_lod_cell)
    {
        return;
    }
    cache.last_lighting_revision = Some(lighting.revision);
    cache.last_player_lod_cell = Some(player_lod_cell);

    let max_distance_squared = MAX_POINT_LIGHT_DISTANCE * MAX_POINT_LIGHT_DISTANCE;
    let mut candidates = lighting
        .sources
        .iter()
        .copied()
        .filter_map(|source| {
            let center = source_center(source);
            let distance_squared = center.distance_squared(player_position);
            (distance_squared < max_distance_squared).then_some((distance_squared, source))
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
    for (distance_squared, source) in candidates {
        let shadows_enabled = source.light.casts_shadow && shadowed < MAX_SHADOWED_POINT_LIGHTS;
        if shadows_enabled {
            shadowed += 1;
        }
        let distance_fade = point_light_distance_fade(distance_squared.sqrt());
        let bundle = (
            BlockPointLight,
            Name::new("BlockPointLight"),
            point_light(source, shadows_enabled, distance_fade),
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
    cache.last_player_lod_cell = None;
}

fn point_light(source: BlockLightSource, shadows_enabled: bool, distance_fade: f32) -> PointLight {
    let strength = source.light.emission.min(15) as f32 / 15.0;
    PointLight {
        color: Color::linear_rgb(
            source.light.color[0],
            source.light.color[1],
            source.light.color[2],
        ),
        intensity: MAX_BLOCK_LIGHT_LUMENS * strength * distance_fade.clamp(0.0, 1.0),
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

/// 在点光 LOD 边缘使用平滑三次曲线，避免跨距离阈值时亮度突变。
fn point_light_distance_fade(distance: f32) -> f32 {
    let fade_span = MAX_POINT_LIGHT_DISTANCE - POINT_LIGHT_FADE_START_DISTANCE;
    let progress = ((distance - POINT_LIGHT_FADE_START_DISTANCE) / fade_span).clamp(0.0, 1.0);
    1.0 - progress * progress * (3.0 - 2.0 * progress)
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
