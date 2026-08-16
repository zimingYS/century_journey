//! 本地玩家表现装配与第一人称可见性系统。

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::client::camera::FpsCamera;
use crate::client::interpolation::SimulationPresentation;
use crate::client::player::model::animation::PlayerAnimationState;
use crate::client::player::model::gltf_rig::spawn_glb_player_rig;
use crate::client::player::model::rig::PlayerRigEntities;
use crate::game::player::identity::LocalPlayer;
use crate::game::player::physics::components::PlayerCollider;

/// 世界渲染层编号：玩家模型与第一人称手臂使用该层，避免被其他渲染层过滤。
const WORLD_RENDER_LAYER: usize = 0;

/// 为已创建的本地权威玩家附加客户端表现。
///
/// Game 层拥有玩家的模拟状态；本系统只创建相机、骨架、动画和渲染层级。
///
/// **关键**：Game 层把 `player` 实体的 transform 定位在碰撞箱**中心**（不是脚底），
/// 而新 glTF 模型的脚底在 mesh 本地 y=0（程序化 rig 通过 joint_offset 把模型整体下移补偿了这点）。
/// 这里我们查询 `PlayerCollider` 的 half_height，把 `rig_root` 相对 player 位置下移这段距离，
/// 让模型脚底在 player 位置正下方，贴地站立。
pub(super) fn attach_local_player_presentation_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    players: Query<(Entity, Option<&PlayerCollider>), With<LocalPlayer>>,
) {
    let Ok((player, collider)) = players.single() else {
        return;
    };
    // 玩家碰撞箱半高决定了"脚底相对实体位置"的偏移量；fallback 0.9 米
    // 与 PlayerCollider::default() 保持一致，避免 PlayerCollider 缺失时贴脸
    let feet_offset = collider.map(|c| -c.half_extents.y).unwrap_or(-0.9);

    // **关键**：feet_offset 只应用在 rig_root 上（不应用在 presentation_root）。
    // presentation_root 下还挂着第一人称相机（局部 y=0.75），若整体下移会让相机
    // 也跟着降到腰部/脚下，导致"第一人称相机位置靠下"。rig_root 是模型的
    // 根节点，单独下移 rig_root 让脚底贴地而不影响相机头部高度。
    let presentation_root = commands
        .spawn((
            Name::new("PlayerPresentation"),
            // 使用 full_transform 让 presentation_root 跟随 player 的位移与偏航；
            // 相机作为其子节点会随玩家水平转身而绕玩家旋转（以玩家为中心的视角）。
            // 若用 translation_only，相机只跟随位移、不跟随旋转，会导致横向转视野时
            // 相机停在原地打转、与垂直俯仰的体验不一致。
            SimulationPresentation::full_transform(),
            Transform::default(),
            Visibility::default(),
        ))
        .id();
    let rig_root = spawn_glb_player_rig(
        &mut commands,
        &asset_server,
        presentation_root,
        player,
        "PlayerRig",
    );
    commands
        .entity(rig_root)
        .insert(Transform::from_xyz(0.0, feet_offset, 0.0));

    // 创建玩家相机
    let camera = commands
        .spawn((
            FpsCamera::default(),
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.75, 0.0),
            RenderLayers::layer(WORLD_RENDER_LAYER),
        ))
        .id();

    // 渲染玩家模型——PlayerRigEntities 由 bind_player_rig_system 异步插入，
    // 这里只负责放进动画状态和可见性。
    commands
        .entity(player)
        .insert((PlayerAnimationState::default(), Visibility::default()));

    commands.entity(player).add_child(presentation_root);
    commands
        .entity(presentation_root)
        .add_child(rig_root)
        .add_child(camera);
}

/// 玩家模型可见性与渲染层同步。
///
/// 玩家模型以 `armature=false` 导出，每个肢体是独立 mesh。第一人称相机位于头部几何体
/// 内部（相机相对 presentation_root 偏移 (0, 0.78, -0.18)，head cube 中心相对
/// presentation_root 约 (0, 0.85, 0)、半尺寸约 0.25），因此 head mesh 依靠 backface
/// culling（bind_player_rig_on_ready 已把材质显式设为 `cull_mode = Back`）从相机内部
/// 自动隐形——相机在头内看到的全是背面，被剔除，无需手动隐藏。
///
/// 身体/腿位于相机下方（body 顶部相对 presentation_root 约 y=0.6 < 相机 y=0.78），
/// 第一人称低头即可看到自己的衣服。所有 mesh（头/身/腿/手臂）在两个视角下都保持可见，
/// 从而对 directional light 投射完整的全身阴影（修复此前隐藏身体导致阴影只剩双手的问题）。
pub(super) fn first_person_visibility_system(
    mut commands: Commands,
    rig_query: Query<&PlayerRigEntities, With<LocalPlayer>>,
    mut mesh_query: Query<(&mut Visibility, Option<&mut RenderLayers>)>,
) {
    let Ok(rig) = rig_query.single() else {
        return;
    };
    // 所有玩家 mesh 始终可见；head 靠 backface culling 从相机内部隐形，
    // body/arm 正常渲染并投射完整阴影。仅同步渲染层，保证与玩家相机同 layer。
    for mesh_entity in &rig.mesh_entities {
        let Ok((mut visibility, layers)) = mesh_query.get_mut(*mesh_entity) else {
            continue;
        };
        let target_layers = RenderLayers::layer(WORLD_RENDER_LAYER);
        *visibility = Visibility::Inherited;
        if let Some(mut layers) = layers {
            *layers = target_layers;
        } else {
            commands.entity(*mesh_entity).insert(target_layers);
        }
    }
}
