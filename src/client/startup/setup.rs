use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

pub fn setup(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        RenderLayers::layer(0).with(1),
    ));

    info!("[Century Journey] game started");
}
