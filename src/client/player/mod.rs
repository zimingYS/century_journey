use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::client::camera::{CameraPlugin, FpsCamera};
use crate::client::interpolation::SimulationPresentation;
use crate::game::crafting::grid::{ActiveCrafting, PlayerCrafting};
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::stats::{Defense, Health, Hunger};
use crate::game::player::components::{
    EnvironmentExposure, FoodUseState, LocalPlayer, Player, PlayerAim, PlayerCollider,
    PlayerGravity, PlayerId, PlayerLifecycle, PlayerMovement, PlayerVelocity, RespawnPoint,
};
use crate::game::player::model::PlayerModelPlugin;
use crate::game::player::model::animation::PlayerAnimationState;
use crate::game::player::model::components::{PlayerMesh, PlayerPart};
use crate::game::player::model::config::PlayerModelConfig;
use crate::game::player::plugin::GamePlayerPlugin;
use crate::game::simulation::SimulationTransformHistory;

pub mod full_body;

const WORLD_RENDER_LAYER: usize = 0;
const PLAYER_SHADOW_ONLY_LAYER: usize = 1;

pub struct ClientPlayerPlugin;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlayerPlugin)
            .add_plugins(PlayerModelPlugin)
            .add_plugins(full_body::FullBodyFirstPersonPlugin)
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
    let (rig_root, rig_entities) = crate::game::player::model::rig::spawn_player_rig_v2(
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

    let presentation_root = commands
        .spawn((
            Name::new("PlayerPresentation"),
            SimulationPresentation::translation_only(),
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    let player_transform = Transform::from_xyz(0.0, 70.0, 0.0);

    let player = commands
        .spawn((
            Player,
            LocalPlayer,
            PlayerAim::default(),
            rig_entities.clone(),
            PlayerAnimationState::default(),
            PlayerGravity::default(),
            PlayerCollider::default(),
            PlayerMovement::default(),
            PlayerVelocity::default(),
            FoodUseState::default(),
            Health::default(),
            Hunger::default(),
            Defense::default(),
            player_transform,
            Visibility::default(),
        ))
        .id();

    commands.entity(player).insert((
        PlayerLifecycle::default(),
        RespawnPoint::default(),
        EnvironmentExposure::default(),
        SimulationTransformHistory::new(player_transform),
        PlayerId::LOCAL,
        InventoryState::default(),
        PlayerCrafting::default(),
        ActiveCrafting::default(),
    ));

    commands.entity(player).add_child(presentation_root);
    commands
        .entity(presentation_root)
        .add_child(rig_root)
        .add_child(camera);
}

/// 第一人称真实身体可见性。
///
/// 第一人称仍渲染同一个身体实体，只隐藏头部网格避免相机穿模；第三人称显示完整身体。
fn first_person_visibility_system(
    mut commands: Commands,
    camera_query: Query<&FpsCamera, With<Camera3d>>,
    rig_query: Query<&crate::game::player::model::rig::PlayerRigEntities, With<LocalPlayer>>,
    mut mesh_query: Query<(&PlayerMesh, &mut Visibility, Option<&mut RenderLayers>)>,
) {
    let is_first_person = camera_query
        .single()
        .map(FpsCamera::is_first_person)
        .unwrap_or(true);
    let Ok(rig) = rig_query.single() else {
        return;
    };

    for mesh_entity in &rig.mesh_entities {
        let Ok((mesh, mut visibility, layers)) = mesh_query.get_mut(*mesh_entity) else {
            continue;
        };

        *visibility = Visibility::Inherited;

        // 第一人称只把头部移到相机不可见、光源可见的层，保留头部阴影。
        let target_layers = if is_first_person && mesh.0 == PlayerPart::Head {
            RenderLayers::layer(PLAYER_SHADOW_ONLY_LAYER)
        } else {
            RenderLayers::layer(WORLD_RENDER_LAYER)
        };
        if let Some(mut layers) = layers {
            *layers = target_layers;
        } else {
            commands.entity(*mesh_entity).insert(target_layers);
        }
    }
}
