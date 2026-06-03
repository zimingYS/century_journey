use crate::player::systems::raycast::TargetVoxel;
use bevy::prelude::*;

pub mod components;
pub mod systems;
pub mod camera;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TargetVoxel>()
            .add_systems(Startup,(
                spawn_player,
                camera::convert_mouse_lock_on_startup
            ))
            .add_systems(Update, (
                camera::player_look_system,
                camera::toggle_mouse_lock_system,
                camera::setup_player_camera_system,
                systems::movement::player_movement_system,
                systems::gravity::player_gravity_system,
                systems::interaction::voxel_interaction_system,
                systems::raycast::draw_voxel_highlight_system,
                systems::raycast::update_raycast_system,
            ));
    }
}

fn spawn_player(
    mut commands: Commands,
) {
    // 生成玩家身体
    commands.spawn((
        components::Player,
        components::PlayerGravity::default(),
        components::PlayerCollider::default(),
        components::PlayerMovement::default(),
        Transform::from_xyz(0.0, 70.0, 0.0),
        Visibility::default(),
    )).with_children(|parent| {
        parent.spawn((
            camera::FpsCamera::default(),
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.75, 0.0),
        ));
    });
}