use crate::game::player::components::Player;
use crate::shared::states::input_blocked::InputBlocked;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog};
use bevy::pbr::AtmosphereSettings;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

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
        ));
    }
}

pub fn player_look_system(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut FpsCamera), Without<Player>>,
    input_blocked: Res<InputBlocked>,
) {
    if input_blocked.0 {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    // 左右旋转
    if let Ok(mut player_transform) = player_query.single_mut() {
        player_transform.rotate_y(-delta.x * 0.0015);
    }

    // 上下旋转
    if let Ok((mut camera_transform, mut fps_camera)) = camera_query.single_mut() {
        camera_transform.rotate_local_x(-delta.y * 0.0015);
    }
}

pub fn convert_mouse_lock_on_startup(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor_options) = cursor_options_query.single_mut() else {
        return;
    };

    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

pub fn toggle_mouse_lock_system(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut cursor_options) = cursor_options_query.single_mut() else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::Escape) {
        if cursor_options.grab_mode == CursorGrabMode::Locked {
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
        } else {
            cursor_options.visible = false;
            cursor_options.grab_mode = CursorGrabMode::Locked;
        }
    }
}

/// F5切换第一人称/第三人称视角
pub fn toggle_perspective_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut FpsCamera, With<Camera3d>>,
) {
    if !keyboard_input.just_pressed(KeyCode::F5) {
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
        app.add_systems(Startup, convert_mouse_lock_on_startup)
            .add_systems(
                Update,
                (
                    player_look_system,
                    toggle_mouse_lock_system,
                    toggle_perspective_system,
                    camera_perspective_sync_system,
                    setup_player_camera_system,
                ),
            );
    }
}
