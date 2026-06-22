use bevy::prelude::*;

use crate::client::camera::{CameraPlugin, FpsCamera};
use crate::client::viewmodel::{ViewModelAnimator, ViewModelPlugin, ViewModelRoot};
use crate::game::player::components::stats::{Defense, Health, Hunger};
use crate::game::player::components::{Player, PlayerCollider, PlayerGravity, PlayerMovement};
use crate::game::player::model::PlayerModelPlugin;
use crate::game::player::model::animation::PlayerAnimationState;
use crate::game::player::model::components::{PlayerJoint, PlayerMesh};
use crate::game::player::model::config::PlayerModelConfig;
use crate::game::player::model::rig::PlayerRigEntities;
use crate::game::player::plugin::GamePlayerPlugin;

/// 客户端玩家 Plugin。
///
/// 负责组装所有玩家相关子 Plugin（Game 逻辑 + Client 表现），
/// 并生成玩家实体（spawn_player）。
///
/// Client → Game 为合法依赖方向。
pub struct ClientPlayerPlugin;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlayerPlugin)
            .add_plugins(PlayerModelPlugin)
            .add_plugins(ViewModelPlugin)
            .add_plugins(CameraPlugin)
            .add_systems(Startup, spawn_player)
            .add_systems(Update, first_person_visibility_system);
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<PlayerModelConfig>,
) {
    let (rig_root, _entities) = crate::game::player::model::rig::spawn_player_rig_v2(
        &mut commands,
        &mut meshes,
        &mut materials,
        &config,
    );

    let camera = commands
        .spawn((
            FpsCamera::default(),
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.75, 0.0),
        ))
        .id();

    let _vm_root = commands
        .spawn((
            ViewModelRoot,
            ViewModelAnimator::default(),
            Name::new("ViewModelRoot"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    commands.entity(camera).add_child(_vm_root);

    commands
        .spawn((
            Player,
            PlayerAnimationState::default(),
            PlayerGravity::default(),
            PlayerCollider::default(),
            PlayerMovement::default(),
            Health::default(),
            Hunger::default(),
            Defense::default(),
            Transform::from_xyz(0.0, 70.0, 0.0),
            Visibility::default(),
        ))
        .add_child(rig_root)
        .add_child(camera);
}

/// 第一人称: 隐藏全身, 仅显示右臂
fn first_person_visibility_system(
    camera_query: Query<&FpsCamera, With<Camera3d>>,
    rig: Option<Res<PlayerRigEntities>>,
    mut joint_query: Query<(Entity, &PlayerJoint, &mut Visibility), Without<PlayerMesh>>,
    mut mesh_query: Query<(Entity, &PlayerMesh, &mut Visibility), Without<PlayerJoint>>,
) {
    let is_fp = camera_query
        .single()
        .map(|c| c.is_first_person)
        .unwrap_or(true);
    let Some(rig) = rig.as_ref() else { return };

    let right_arm_joints = [rig.upper_arm_r, rig.forearm_r, rig.hand_r];
    let right_arm_meshes = [rig.upper_arm_r, rig.forearm_r, rig.hand_r];

    for (entity, _joint, mut vis) in &mut joint_query {
        if is_fp && right_arm_joints.contains(&entity) {
            *vis = Visibility::Inherited;
        } else {
            *vis = if is_fp {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
    for (entity, _mesh, mut vis) in &mut mesh_query {
        if is_fp && right_arm_meshes.contains(&entity) {
            *vis = Visibility::Inherited;
        } else {
            *vis = if is_fp {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
}
