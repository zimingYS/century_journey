//! cj-render —— 渲染层
//!
//! 规则：只消费渲染数据，不知道模拟算法（不依赖 sim）。

pub mod input;
pub mod meshing;
pub mod resources;
pub mod terrain;

use bevy::prelude::*;
use terrain::spawn_section_meshes;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (spawn_section_meshes, spawn_camera_and_light));
    }
}

/// 临时生成相机和光源
fn spawn_camera_and_light(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            color: Color::WHITE,
            brightness: 150.0,
            ..default()
        },
        Transform::from_xyz(60.0, 190.0, 60.0).looking_at(Vec3::new(8.0, 128.0, 8.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.5, 0.0)),
    ));
}
