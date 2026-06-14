use bevy::prelude::*;
use crate::player::systems::raycast::TargetVoxel;
use crate::player::components::stats::{Defense, Health, Hunger};
use crate::player::events::{DamageEvent, DeathEvent, HealEvent};

pub mod components;
pub mod systems;
pub mod camera;
pub mod events;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TargetVoxel>()
            .add_message::<DamageEvent>()
            .add_message::<HealEvent>()
            .add_message::<DeathEvent>()
            .add_systems(Startup, (spawn_player, camera::convert_mouse_lock_on_startup))
            .add_systems(Update, (
                camera::player_look_system,
                camera::toggle_mouse_lock_system,
                camera::setup_player_camera_system,
                systems::movement::player_movement_system,
                systems::gravity::player_gravity_system,
            ))
            .add_systems(Update, (
                systems::interaction::voxel_interaction_system,
                systems::raycast::draw_voxel_highlight_system,
                systems::raycast::update_raycast_system,
            ))
            .add_systems(Update, (
                systems::combat::damage_system,
                systems::combat::heal_system,
                systems::combat::death_system,
            ).chain())
            .add_systems(Update, (
                systems::hunger::action_cost_system,
                systems::hunger::natural_regeneration_system,
                systems::hunger::starvation_damage_system,
            ).chain())
            .add_systems(Update, systems::armor_calc::armor_calculation_system)
            .add_systems(Update, systems::interaction::drop_item_system);
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
        Health::default(),
        Hunger::default(),
        Defense::default(),
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