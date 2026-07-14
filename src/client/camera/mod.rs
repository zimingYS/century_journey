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

    // 使用绝对俯仰角重建旋转，避免累计旋转越过垂直方向后天地翻转。
    if let Ok((mut camera_transform, mut fps_camera)) = camera_query.single_mut() {
        fps_camera.add_pitch(-delta.y * sensitivity);
        camera_transform.rotation = fps_camera.pitch_rotation();
    }
}

/// 使用 F5 切换第一人称与第三人称视角。
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
        camera_transform.translation = perspective_offset(fps_camera.is_first_person);
        camera_transform.rotation = fps_camera.pitch_rotation();
    }
}

fn perspective_offset(is_first_person: bool) -> Vec3 {
    if is_first_person {
        // 眼位略微前移到胸口前方，低头时视线不会穿过自己的躯干。
        Vec3::new(0.0, 0.78, -0.18)
    } else {
        Vec3::new(0.0, 0.62, 4.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player::model::components::PlayerPart;
    use crate::game::player::model::config::PlayerModelConfig;
    use crate::shared::components::camera::{MAX_CAMERA_PITCH, MIN_CAMERA_PITCH};

    #[test]
    fn player_visual_camera_pitch_is_clamped_before_world_can_flip() {
        let mut camera = FpsCamera::default();
        camera.add_pitch(10.0);
        assert_eq!(camera.pitch, MAX_CAMERA_PITCH);

        camera.add_pitch(-20.0);
        assert_eq!(camera.pitch, MIN_CAMERA_PITCH);
    }

    #[test]
    fn player_visual_first_person_eye_is_in_front_of_torso() {
        let eye = perspective_offset(true);
        let torso_front = -PlayerModelConfig::half_dims(PlayerPart::Body).z;

        assert!(eye.z < torso_front);
        assert!(eye.y > PlayerModelConfig::joint_offset(PlayerPart::Body).y);
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
