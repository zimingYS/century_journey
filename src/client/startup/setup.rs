use bevy::prelude::*;

pub fn setup(mut commands: Commands) {
    // 添加光源
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    info!("[世纪之旅] 游戏已启动！");
}
