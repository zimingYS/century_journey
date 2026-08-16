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

/// 同步摄像机位置
pub(super) fn camera_perspective_sync_system(
    mut camera_query: Query<(&FpsCamera, &mut Transform), With<Camera3d>>,
) {
    for (fps_camera, mut camera_transform) in camera_query.iter_mut() {
        camera_transform.translation = perspective_offset(fps_camera.perspective);
        camera_transform.rotation = perspective_rotation(fps_camera);
    }
}

/// 返回指定视角对应的相机局部偏移。
pub(super) fn perspective_offset(perspective: CameraPerspective) -> Vec3 {
    match perspective {
        CameraPerspective::FirstPerson => Vec3::new(0.0, 0.78, -0.18),
        CameraPerspective::SecondPerson => Vec3::new(0.0, 0.62, -4.5),
        CameraPerspective::ThirdPerson => Vec3::new(0.0, 0.62, 4.5),
    }
}

/// 返回指定视角对应的相机局部旋转。
pub(super) fn perspective_rotation(camera: &FpsCamera) -> Quat {
    match camera.perspective {
        CameraPerspective::FirstPerson | CameraPerspective::ThirdPerson => camera.pitch_rotation(),
        // 第二人称相机绕 Y 掉头 180° 面向玩家，俯仰不再取反：
        // Ry(π)·Rx(p) 的 forward 为 (0, sin p, cos p)，与第一人称一致（正俯仰 = 视线向上）。
        CameraPerspective::SecondPerson => {
            Quat::from_rotation_y(std::f32::consts::PI) * Quat::from_rotation_x(camera.pitch)
        }
    }
}
