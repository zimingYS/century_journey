use bevy::prelude::*;
use crate::player::camera::{convert_mouse_lock_on_startup, player_look_system, toggle_mouse_lock_system};
use crate::player::systems::movement::player_movement_system;

pub mod components;
pub mod systems;
pub mod camera;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup,(
                spawn_player,
                convert_mouse_lock_on_startup
            ))
            .add_systems(Update, (
                player_look_system,
                player_movement_system,
                toggle_mouse_lock_system,
            ));
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 生成玩家身体
    commands.spawn((
        components::Player,
        components::PlayerMovement::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
    )).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::default().mesh().build())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ));

        parent.spawn((
            camera::FpsCamera::default(),
            Camera3d::default(),
            Transform::from_xyz(0.0, 1.65, 0.0)
        ));
    });
}