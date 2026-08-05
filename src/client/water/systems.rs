//! 水面材质动画、水下检测与滤镜表现系统。
//!
//! 这些系统只运行在渲染帧：权威水体仍由 Game 的体素状态决定，客户端只把它
//! 转换成材质参数和相机后处理。

use super::components::UnderwaterOverlay;
use super::constants::*;
use super::material::WaterMaterial;
use crate::client::camera::FpsCamera;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::game::player::identity::LocalPlayer;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::state::WorldState;
use bevy::camera::Exposure;
use bevy::light::VolumetricFog;
use bevy::math::Affine2;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::render::view::ColorGrading;
use bevy::time::Real;

/// 水下状态：记录浸没目标和平滑滤镜强度。
#[derive(Resource, Debug, Clone, Copy)]
pub struct UnderwaterState {
    /// 当前滤镜强度（0=水面以上，1=完全浸没）。
    pub depth: f32,
    /// 目标滤镜强度，由玩家头部方块检测决定。
    pub target: f32,
}

impl Default for UnderwaterState {
    fn default() -> Self {
        Self {
            depth: 0.0,
            target: 0.0,
        }
    }
}

/// 计算下一帧滤镜强度：向目标平滑逼近，并限制在有效范围。
pub fn underwater_depth_step(depth: f32, target: f32, rate: f32, dt: f32) -> f32 {
    let t = (rate * dt).clamp(0.0, 1.0);
    (depth + (target - depth) * t).clamp(0.0, 1.0)
}

/// 计算水面 UV 流动偏移，保留给旧的表现层测试和工具使用。
pub fn water_flow_offset(elapsed: f32, speed: f32, tile: f32) -> f32 {
    if tile <= 0.0 {
        return 0.0;
    }
    (elapsed * speed).rem_euclid(tile)
}

/// 计算水下覆盖层透明度：按深度线性映射到 [0, max_alpha]。
pub fn compute_underwater_alpha(depth: f32, max_alpha: f32) -> f32 {
    depth.clamp(0.0, 1.0) * max_alpha.max(0.0)
}

/// 根据水面与场景深度计算透明渐变因子。
///
/// 该函数与 WGSL 中的厚度映射保持相同边界，便于对岸线泡沫的数值行为做回归测试。
pub fn water_depth_factor(scene_thickness: f32, fade_distance: f32) -> f32 {
    if fade_distance <= 0.0 {
        return 1.0;
    }
    (scene_thickness / fade_distance).clamp(0.0, 1.0)
}

/// 创建全屏水下色层。低透明度只负责统一色相，细节由雾和颜色分级提供。
pub fn spawn_underwater_overlay_system(mut commands: Commands) {
    commands.spawn((
        Name::new("UnderwaterOverlay"),
        UnderwaterOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(
            UNDERWATER_FOG_COLOR[0],
            UNDERWATER_FOG_COLOR[1],
            UNDERWATER_FOG_COLOR[2],
            0.0,
        )),
        GlobalZIndex(28_000),
        Pickable::IGNORE,
        Visibility::Visible,
    ));
}

/// 每帧更新水面基线纹理偏移和效果层时间；两者都使用真实渲染时间。
pub fn water_flow_animation_system(
    time: Res<Time<Real>>,
    render_assets: Res<BlockRenderAssets>,
    mut base_materials: ResMut<Assets<StandardMaterial>>,
    mut effect_materials: ResMut<Assets<WaterMaterial>>,
) {
    let elapsed = time.elapsed_secs() * WATER_SURFACE_TIME_SCALE;
    let offset = water_flow_offset(elapsed, WATER_FLOW_SPEED, WATER_FLOW_TILE);

    if let Some(mut base_material) = base_materials.get_mut(render_assets.water_base_material()) {
        base_material.uv_transform = Affine2::from_translation(Vec2::new(offset, offset * 0.63));
    }
    if let Some(mut effect_material) =
        effect_materials.get_mut(render_assets.water_effect_material())
    {
        effect_material.extension.uniform.time_seconds = elapsed;
    }
}
/// 检测玩家头部所在体素是否为水，并更新滤镜目标。
pub fn underwater_detect_system(
    registry: Option<Res<BlockRegistry>>,
    world_state: Res<WorldState>,
    player_query: Query<&Transform, With<LocalPlayer>>,
    mut state: ResMut<UnderwaterState>,
) {
    let Some(registry) = registry else {
        return;
    };
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let head = player_transform.translation + Vec3::Y * 0.8;
    let block_id = get_voxel_at_world(head.floor().as_ivec3(), &world_state);
    state.target = if is_water_id(registry.as_ref(), block_id) {
        1.0
    } else {
        0.0
    };
}
fn is_water_id(registry: &BlockRegistry, block_id: u16) -> bool {
    registry
        .get_identifier_by_id(block_id)
        .is_some_and(|identifier| identifier.path() == "water")
}

/// 将水下状态投影到相机后处理和全屏色层。
///
/// atmosphere_system 每帧先写入天气基线，本系统必须排在它之后，只写入当前
/// 水下强度对应的偏移，避免跨帧累加曝光和雾色。
#[allow(clippy::type_complexity)]
pub fn underwater_filter_system(
    time: Res<Time>,
    mut state: ResMut<UnderwaterState>,
    mut camera_query: Query<
        (
            &mut Exposure,
            &mut DistanceFog,
            &mut VolumetricFog,
            Option<&mut ColorGrading>,
        ),
        With<FpsCamera>,
    >,
    mut overlay_query: Query<&mut BackgroundColor, With<UnderwaterOverlay>>,
) {
    state.depth = underwater_depth_step(
        state.depth,
        state.target,
        UNDERWATER_DEPTH_RATE,
        time.delta_secs(),
    );
    let depth = state.depth;

    if let Ok((mut exposure, mut distance_fog, mut volumetric_fog, grading)) =
        camera_query.single_mut()
    {
        let base_start = distance_fog_start(&distance_fog);
        let base_end = distance_fog_end(&distance_fog);
        let fog_color = Color::srgba(
            UNDERWATER_FOG_COLOR[0],
            UNDERWATER_FOG_COLOR[1],
            UNDERWATER_FOG_COLOR[2],
            0.95,
        );
        distance_fog.color = distance_fog.color.mix(&fog_color, depth);
        distance_fog.falloff = FogFalloff::Linear {
            start: base_start + (UNDERWATER_FOG_NEAR - base_start) * depth,
            end: base_end + (UNDERWATER_FOG_FAR - base_end) * depth,
        };
        volumetric_fog.ambient_color = volumetric_fog.ambient_color.mix(
            &Color::srgb(
                UNDERWATER_VOLUMETRIC_COLOR[0],
                UNDERWATER_VOLUMETRIC_COLOR[1],
                UNDERWATER_VOLUMETRIC_COLOR[2],
            ),
            depth,
        );
        exposure.ev100 += UNDERWATER_EXPOSURE_OFFSET * depth;

        if let Some(mut grading) = grading {
            grading.global.exposure = UNDERWATER_COLOR_GRADE_EXPOSURE * depth;
            grading.global.temperature = -0.18 * depth;
            grading.global.tint = -0.05 * depth;
            grading.global.post_saturation =
                1.0 - (1.0 - UNDERWATER_COLOR_GRADE_SATURATION) * depth;
        }
    }

    if let Ok(mut background) = overlay_query.single_mut() {
        background.0 = Color::srgba(
            UNDERWATER_FOG_COLOR[0],
            UNDERWATER_FOG_COLOR[1],
            UNDERWATER_FOG_COLOR[2],
            compute_underwater_alpha(depth, UNDERWATER_OVERLAY_MAX_ALPHA),
        );
    }
}

/// 退出游戏状态时重置滤镜状态，避免残留。
pub fn reset_underwater_state_system(
    mut state: ResMut<UnderwaterState>,
    mut overlay_query: Query<&mut BackgroundColor, With<UnderwaterOverlay>>,
) {
    *state = UnderwaterState::default();
    if let Ok(mut background) = overlay_query.single_mut() {
        background.0 = Color::srgba(
            UNDERWATER_FOG_COLOR[0],
            UNDERWATER_FOG_COLOR[1],
            UNDERWATER_FOG_COLOR[2],
            0.0,
        );
    }
}
fn distance_fog_start(fog: &DistanceFog) -> f32 {
    match fog.falloff {
        FogFalloff::Linear { start, .. } => start,
        _ => 0.0,
    }
}

fn distance_fog_end(fog: &DistanceFog) -> f32 {
    match fog.falloff {
        FogFalloff::Linear { end, .. } => end,
        _ => 0.0,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/client/water/systems.rs"]
mod tests;
