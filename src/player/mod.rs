use crate::player::components::PlayerCamera;
use crate::player::systems::raycast::TargetVoxel;
use bevy::light::atmosphere::ScatteringMedium;
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
                systems::movement::player_movement_system,
                camera::toggle_mouse_lock_system,
                systems::interaction::voxel_interaction_system,
                systems::raycast::draw_voxel_highlight_system,
                systems::raycast::update_raycast_system,
            ));
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    // 生成玩家身体
    commands.spawn((
        components::Player,
        components::PlayerMovement::default(),
        Transform::from_xyz(0.0, 70.0, 0.0),
        Visibility::default(),
    )).with_children(|parent| {
        parent.spawn((
            PlayerCamera,
            camera::FpsCamera::default(),
            Camera3d::default(),
            Transform::from_xyz(0.0, 1.65, 0.0),
        ));
    });
}