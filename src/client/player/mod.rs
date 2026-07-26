use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::client::camera::{CameraPlugin, FpsCamera};
use crate::client::interpolation::SimulationPresentation;
use crate::game::player::identity::LocalPlayer;
use crate::game::player::lifecycle::spawn::PlayerStartupSet;
use model::PlayerModelPlugin;
use model::animation::PlayerAnimationState;
use model::components::{PlayerMesh, PlayerPart};
use model::config::PlayerModelConfig;

pub mod full_body;
pub mod model;

const WORLD_RENDER_LAYER: usize = 0;
const PLAYER_SHADOW_ONLY_LAYER: usize = 1;

pub struct ClientPlayerPlugin;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerModelPlugin)
            .add_plugins(full_body::FullBodyFirstPersonPlugin)
            .add_plugins(CameraPlugin)
            .add_systems(
                Startup,
                attach_local_player_presentation_system.after(PlayerStartupSet::Authority),
            )
            .add_systems(Update, first_person_visibility_system);
    }
}

/// 为已创建的本地权威玩家附加客户端表现。
///
/// Game 层拥有玩家的模拟状态；本系统只创建相机、骨架、动画和渲染层级。
fn attach_local_player_presentation_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<PlayerModelConfig>,
    players: Query<Entity, With<LocalPlayer>>,
) {
    let Ok(player) = players.single() else {
        return;
    };

    // 创建骨骼
    let (rig_root, rig_entities) =
        model::rig::spawn_player_rig_v2(&mut commands, &mut meshes, &mut materials, &config);

    // 创建玩家相机
    let camera = commands
        .spawn((
            FpsCamera::default(),
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.75, 0.0),
            RenderLayers::layer(WORLD_RENDER_LAYER),
        ))
        .id();

    // 创建玩家表现根实体
    let presentation_root = commands
        .spawn((
            Name::new("PlayerPresentation"),
            SimulationPresentation::translation_only(),
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    // 渲染玩家模型
    commands.entity(player).insert((
        rig_entities,
        PlayerAnimationState::default(),
        Visibility::default(),
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
    rig_query: Query<&model::rig::PlayerRigEntities, With<LocalPlayer>>,
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
