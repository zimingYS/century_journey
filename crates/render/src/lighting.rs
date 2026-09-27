//! 光源

use bevy::prelude::*;

/// 临时生成光源
pub fn spawn_light(mut commands: Commands) {
    // 平行光
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.5, 0.0)),
    ));

    // 环境光
    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 150.0,
        ..default()
    });
}
