use crate::game::player::components::stats::{Defense, Health, Hunger};
use crate::game::player::events::{DamageEvent, DeathEvent, HealEvent};
use crate::game::player::model::components::{PlayerJoint, PlayerMesh};
use crate::game::player::model::rig::PlayerRigEntities;
use crate::game::player::systems::raycast::TargetVoxel;
use bevy::prelude::*;

pub mod components;
pub mod systems;
pub mod camera;
pub mod events;
pub mod model;
pub mod view_model;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(model::PlayerModelPlugin)
            .add_plugins(view_model::ViewModelPlugin)
            .init_resource::<TargetVoxel>()
            .add_message::<DamageEvent>()
            .add_message::<HealEvent>()
            .add_message::<DeathEvent>()
            .add_systems(Startup, (spawn_player, camera::convert_mouse_lock_on_startup))
            .add_systems(Update, (
                camera::player_look_system,
                camera::toggle_mouse_lock_system,
                camera::toggle_perspective_system,
                camera::camera_perspective_sync_system,
                camera::setup_player_camera_system,
                first_person_visibility_system,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<model::config::PlayerModelConfig>,
) {
    let (rig_root, _entities) = model::rig::spawn_player_rig_v2(&mut commands, &mut meshes, &mut materials, &config);

    let camera = commands.spawn((
        camera::FpsCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.75, 0.0),
    )).id();

    // 第一人称物品根节点
    let _vm_root = commands.spawn((
        view_model::ViewModelRoot,
        view_model::ViewModelAnimator::default(),
        Name::new("ViewModelRoot"),
        Transform::default(),
        Visibility::default(),
    )).id();
    commands.entity(camera).add_child(_vm_root);

    commands.spawn((
        components::Player,
        model::animation::PlayerAnimationState::default(),
        components::PlayerGravity::default(),
        components::PlayerCollider::default(),
        components::PlayerMovement::default(),
        Health::default(),
        Hunger::default(),
        Defense::default(),
        Transform::from_xyz(0.0, 70.0, 0.0),
        Visibility::default(),
    )).add_child(rig_root)
    .add_child(camera);
}

/// 第一人称: 隐藏全身, 仅显示右臂 (通过 Entity)
fn first_person_visibility_system(
    camera_query: Query<&camera::FpsCamera, With<Camera3d>>,
    rig: Option<Res<PlayerRigEntities>>,
    mut joint_query: Query<(Entity, &PlayerJoint, &mut Visibility), Without<PlayerMesh>>,
    mut mesh_query: Query<(Entity, &PlayerMesh, &mut Visibility), Without<PlayerJoint>>,
) {
    let is_fp = camera_query.single().map(|c| c.is_first_person).unwrap_or(true);
    let Some(rig) = rig.as_ref() else { return };

    // 右臂相关的 entity 集合
    let right_arm_joints = [rig.upper_arm_r, rig.forearm_r, rig.hand_r];
    let right_arm_meshes = [rig.upper_arm_r, rig.forearm_r, rig.hand_r];

    for (entity, _joint, mut vis) in &mut joint_query {
        if is_fp && right_arm_joints.contains(&entity) {
            *vis = Visibility::Inherited;
        } else {
            *vis = if is_fp { Visibility::Hidden } else { Visibility::Inherited };
        }
    }
    for (entity, _mesh, mut vis) in &mut mesh_query {
        if is_fp && right_arm_meshes.contains(&entity) {
            *vis = Visibility::Inherited;
        } else {
            *vis = if is_fp { Visibility::Hidden } else { Visibility::Inherited };
        }
    }
}