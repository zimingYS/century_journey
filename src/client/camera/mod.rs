use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::Player;
use crate::shared::states::InputContextState;
use bevy::audio::SpatialListener;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog};
use bevy::pbr::AtmosphereSettings;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

pub use crate::shared::components::camera::FpsCamera;

pub fn setup_player_camera_system(
    mut query: Query<Entity, Added<FpsCamera>>,
    mut commands: Commands,
) {
    for entity in &mut query {
        commands.entity(entity).insert((
            AtmosphereSettings::default(),
            AtmosphereEnvironmentMapLight::default(),
            Exposure { ..default() },
            Tonemapping::AcesFitted,
            Bloom::NATURAL,
            VolumetricFog {
                ambient_intensity: 0.0,
                ..default()
            },
            SpatialListener::new(0.22),
        ));
    }
}

pub fn player_look_system(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut FpsCamera), Without<Player>>,
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

    // 上下旋转
    if let Ok((mut camera_transform, _fps_camera)) = camera_query.single_mut() {
        camera_transform.rotate_local_x(-delta.y * sensitivity);
    }
}

/// F5切换第一人称/第三人称视角
pub fn toggle_perspective_system(
    actions: Res<PlayerActionState>,
    mut camera_query: Query<&mut FpsCamera, With<Camera3d>>,
) {
    if !actions.just_pressed(PlayerAction::TogglePerspective) {
        return;
    }
    for mut fps_camera in &mut camera_query {
        fps_camera.is_first_person = !fps_camera.is_first_person;
        info!(
            "视角切换: {}",
            if fps_camera.is_first_person {
                "第一人称"
            } else {
                "第三人称"
            }
        );
    }
}

/// 同步摄像机位置
pub fn camera_perspective_sync_system(
    mut camera_query: Query<(&FpsCamera, &mut Transform), With<Camera3d>>,
) {
    for (fps_camera, mut camera_transform) in camera_query.iter_mut() {
        let offset = if fps_camera.is_first_person {
            Vec3::new(0.0, 0.75, 0.0)
        } else {
            Vec3::new(0.0, 0.5, 5.0)
        };
        camera_transform.translation = offset;
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_look_system,
                toggle_perspective_system,
                camera_perspective_sync_system,
                setup_player_camera_system,
            )
                .run_if(in_state(crate::shared::states::AppState::InGame)),
        );
    }
}
