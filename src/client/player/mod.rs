use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::client::camera::{CameraPlugin, FpsCamera};
use crate::client::viewmodel::{ViewModelAnimator, ViewModelPart, ViewModelPlugin, ViewModelRoot};
use crate::game::player::components::stats::{Defense, Health, Hunger};
use crate::game::player::components::{Player, PlayerCollider, PlayerGravity, PlayerMovement};
use crate::game::player::model::PlayerModelPlugin;
use crate::game::player::model::animation::PlayerAnimationState;
use crate::game::player::model::components::{PlayerJoint, PlayerMesh};
use crate::game::player::model::config::PlayerModelConfig;
use crate::game::player::plugin::GamePlayerPlugin;

const WORLD_RENDER_LAYER: usize = 0;
const PLAYER_SHADOW_LAYER: usize = 1;

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
            RenderLayers::layer(WORLD_RENDER_LAYER),
        ))
        .id();

    let vm_root = commands
        .spawn((
            ViewModelRoot,
            ViewModelPart,
            ViewModelAnimator::default(),
            Name::new("ViewModelRoot"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    commands.entity(camera).add_child(vm_root);

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

fn first_person_visibility_system(
    mut commands: Commands,
    camera_query: Query<&FpsCamera, With<Camera3d>>,
    mut joint_query: Query<&mut Visibility, (With<PlayerJoint>, Without<PlayerMesh>)>,
    mut mesh_query: Query<
        (Entity, &mut Visibility, Option<&mut RenderLayers>),
        (With<PlayerMesh>, Without<PlayerJoint>),
    >,
) {
    let is_fp = camera_query
        .single()
        .map(|c| c.is_first_person)
        .unwrap_or(true);

    for mut visibility in &mut joint_query {
        *visibility = Visibility::Inherited;
    }

    for (entity, mut visibility, layers) in &mut mesh_query {
        *visibility = Visibility::Inherited;
        let target_layers = if is_fp {
            RenderLayers::layer(PLAYER_SHADOW_LAYER)
        } else {
            RenderLayers::layer(WORLD_RENDER_LAYER)
        };

        if let Some(mut layers) = layers {
            *layers = target_layers;
        } else {
            commands.entity(entity).insert(target_layers);
        }
    }
}
