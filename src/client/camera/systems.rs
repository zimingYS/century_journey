//! 摄像机装配、视角输入与视角同步系统的实现。

use bevy::audio::SpatialListener;
use bevy::camera::Exposure;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::light::{AtmosphereEnvironmentMapLight, ShadowFilteringMethod, VolumetricFog};
use bevy::pbr::{AtmosphereSettings, DistanceFog, FogFalloff};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::ColorGrading;

use crate::client::camera::types::{CameraPerspective, FpsCamera};
use crate::client::input::ClientActionState;
use crate::game::player::control::action::PlayerAction;
use crate::game::player::identity::Player;
use crate::shared::states::InputContextState;

/// 为本地玩家附加第一人称相机及其俯仰支点。
pub(super) fn setup_player_camera_system(
    mut query: Query<Entity, Added<FpsCamera>>,
    mut commands: Commands,
) {
    for entity in &mut query {
        commands.entity(entity).insert((
            AtmosphereSettings {
                aerial_view_lut_max_distance: 640.0,
                sky_view_lut_samples: 24,
                aerial_view_lut_samples: 16,
                sky_max_samples: 24,
                ..default()
            },
            AtmosphereEnvironmentMapLight {
                intensity: 1.8,
                size: UVec2::splat(256),
                ..default()
            },
            Exposure { ev100: 13.0 },
            Tonemapping::AcesFitted,
            ColorGrading::default(),
            // 水面材质用深度预通道计算水体厚度和岸线泡沫。
            DepthPrepass,
            Bloom::NATURAL,
            VolumetricFog {
                ambient_color: Color::srgb(0.62, 0.72, 0.82),
                ambient_intensity: 0.16,
                ..default()
            },
            DistanceFog {
                color: Color::srgba(0.58, 0.69, 0.78, 0.52),
                directional_light_color: Color::srgba(1.0, 0.76, 0.48, 0.24),
                directional_light_exponent: 24.0,
                falloff: FogFalloff::Linear {
                    start: 64.0,
                    end: 190.0,
                },
            },
            ShadowFilteringMethod::Gaussian,
            SpatialListener::new(0.22),
        ));
    }
}

/// 在渲染帧消费鼠标视角增量并更新玩家偏航和相机俯仰。
pub(super) fn player_look_system(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut FpsCamera, Without<Player>>,
    context: Res<InputContextState>,
    settings: Res<crate::app::flow::GameSettings>,
) {
    if !context.active().allows_gameplay() {
        mouse_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let sensitivity = 0.0015 * settings.mouse_sensitivity;

    // 左右旋转
    if let Ok(mut player_transform) = player_query.single_mut() {
        player_transform.rotate_y(-delta.x * sensitivity);
    }

    // 使用绝对俯仰角重建旋转，避免累计旋转越过垂直方向后天地翻转。
    if let Ok(mut fps_camera) = camera_query.single_mut() {
        fps_camera.add_pitch(-delta.y * sensitivity);
    }
}

/// 使用 F5 切换第一人称与第三人称视角。
pub(super) fn toggle_perspective_system(
    actions: Res<ClientActionState>,
    mut camera_query: Query<&mut FpsCamera, With<Camera3d>>,
) {
    if !actions.just_pressed(PlayerAction::TogglePerspective) {
        return;
    }
    for mut fps_camera in &mut camera_query {
        fps_camera.perspective = fps_camera.perspective.next();
        info!("视角切换: {}", fps_camera.perspective.display_name());
    }
}

/// 同步摄像机位置。
///
/// 第二/第三人称采用**球面轨道（spherical orbit）**：mouse Y 改变球面纬度，
/// camera 沿球面绕玩家移动（pitch↑ 抬高 + 水平距离收缩），camera 始终看向玩家，
/// 实现"绕玩家上下扫描"——而非在原地俯仰。
pub(super) fn camera_perspective_sync_system(
    mut camera_query: Query<(&FpsCamera, &mut Transform), With<Camera3d>>,
) {
    for (fps_camera, mut camera_transform) in camera_query.iter_mut() {
        camera_transform.translation = perspective_offset(fps_camera.perspective, fps_camera.pitch);
        camera_transform.rotation = perspective_rotation(fps_camera);
    }
}

/// 返回指定视角对应的相机局部偏移。
///
/// 第一人称保持眼睛位置不变（FPS 标准行为）。第二/第三人称按球面轨道：
/// pitch=0 时水平距离 4.5m，pitch 升高则高度 +r·sin(p)，水平距离 r·cos(p) 收缩。
pub(super) fn perspective_offset(perspective: CameraPerspective, pitch: f32) -> Vec3 {
    match perspective {
        CameraPerspective::FirstPerson => Vec3::new(0.0, 0.78, -0.18),
        CameraPerspective::ThirdPerson => {
            let r = 4.5;
            Vec3::new(0.0, 0.62 + r * pitch.sin(), r * pitch.cos())
        }
        CameraPerspective::SecondPerson => {
            let r = 4.5;
            Vec3::new(0.0, 0.62 + r * pitch.sin(), -r * pitch.cos())
        }
    }
}

/// 返回指定视角对应的相机局部旋转。
///
/// 第二/第三人称 spherical orbit：rotation 让 camera 的 forward（-Z）指向
/// `-offset`（presentation_root 局部原点即玩家），保持看向玩家。
pub(super) fn perspective_rotation(camera: &FpsCamera) -> Quat {
    match camera.perspective {
        CameraPerspective::FirstPerson => camera.pitch_rotation(),
        CameraPerspective::SecondPerson => look_at_player_quat(perspective_offset(
            CameraPerspective::SecondPerson,
            camera.pitch,
        )),
        CameraPerspective::ThirdPerson => look_at_player_quat(perspective_offset(
            CameraPerspective::ThirdPerson,
            camera.pitch,
        )),
    }
}

/// 返回让 camera 的 forward（Vec3::NEG_Z）指向玩家方向的 Quat。
///
/// 用于球面轨道的第二/第三人称：camera 始终看向 presentation_root（玩家）。
/// offset 是相机相对 presentation_root 的位置，所以相机 forward 应为 `-offset`。
/// 与 Bevy `Transform::look_to` 同构：以世界 +Y 为参考上方向重建正交基，
/// 保证相机地平线保持水平——`from_rotation_arc` 的最短弧在相机几乎背对 -Z
/// （第二人称相机位于玩家前方、forward 接近 +Z）时会产生 ~180° 翻转（画面上下颠倒）。
/// 若 offset 长度为零（理论上不会发生，因为 r>0），返回 identity 避免除零。
fn look_at_player_quat(offset: Vec3) -> Quat {
    // 相机 +Z 轴（back）指向相机身后，即 offset 方向。
    let back = offset.normalize_or_zero();
    if back == Vec3::ZERO {
        return Quat::IDENTITY;
    }
    // up × back 得相机右方向；back ∥ Y（垂直看向玩家）时叉积退化，任取正交方向。
    let right = Vec3::Y.cross(back).try_normalize().unwrap_or(Vec3::X);
    let up = back.cross(right);
    Quat::from_mat3(&Mat3::from_cols(right, up, back))
}
