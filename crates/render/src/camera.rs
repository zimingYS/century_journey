//! 相机与光源（P0 临时，P0-e 起迁往 `cj-game` 的 `player`）

use bevy::prelude::*;

/// 临时生成相机和光源
pub fn spawn_camera_and_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.5, 0.0)),
    ));
}
