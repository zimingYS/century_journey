use crate::game::player::model::components::{
    BackAnchor, ChestAnchor, HeldItemAnchor, HelmetAnchor, OffHandAnchor, PlayerJoint, PlayerMesh,
    PlayerModelMarker, PlayerPart, PlayerRig,
};
use crate::game::player::model::config::PlayerModelConfig;
use bevy::prelude::*;

/// 真实手掌上的主手物品握持姿态。
///
/// 玩家手臂骨骼沿局部 -Y 方向生长，而大多数物品模型也把局部 Y 当成“高度/柄”的方向。
/// 因此这里先把物品坐标系绕 X 轴转 90 度，让物品从手掌向前伸出，而不是顺着前臂躺下。
pub fn held_item_grip_transform() -> Transform {
    Transform {
        translation: Vec3::new(0.0, -0.13, -0.09),
        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        scale: Vec3::ONE,
    }
}
/// 一个玩家 Rig 的所有关键实体链接。
///
/// 本地玩家和未来的远程玩家都应该把这份组件挂在自己的 Player 实体上。动画、装备和手持物品渲染只依赖这些链接，
/// 不再依赖单独的第一人称 ViewModel。
#[derive(Component, Clone)]
pub struct PlayerRigEntities {
    /// Rig 根实体。
    pub root: Entity,
    /// 头部关节。
    pub head_joint: Entity,
    /// 身体关节。
    pub body_joint: Entity,
    /// 右上臂关节。
    pub upper_arm_r: Entity,
    /// 左上臂关节。
    pub upper_arm_l: Entity,
    /// 右前臂关节。
    pub forearm_r: Entity,
    /// 左前臂关节。
    pub forearm_l: Entity,
    /// 右手关节。
    pub hand_r: Entity,
    /// 左手关节。
    pub hand_l: Entity,
    /// 右大腿关节。
    pub thigh_r: Entity,
    /// 左大腿关节。
    pub thigh_l: Entity,
    /// 右小腿关节。
    pub calf_r: Entity,
    /// 左小腿关节。
    pub calf_l: Entity,
    /// 右手持物品挂点。
    pub held_item: Entity,
    /// 左手副手挂点。
    pub offhand: Entity,
    /// 头盔挂点。
    pub helmet: Entity,
    /// 胸甲挂点。
    pub chest: Entity,
    /// 背部挂点。
    pub back: Entity,
    /// 头部可渲染网格，用于第一人称隐藏头部。
    pub head_mesh: Entity,
    /// 所有身体网格实体，用于本地玩家可见性和渲染层同步。
    pub mesh_entities: Vec<Entity>,
}

/// 生成可同时服务第一人称和第三人称的真实玩家 Rig。
pub fn spawn_player_rig_v2(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &PlayerModelConfig,
) -> (Entity, PlayerRigEntities) {
    let cube = meshes.add(Cuboid::default());

    let root = commands
        .spawn((
            PlayerRig,
            PlayerModelMarker,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::default(),
            Name::new("PlayerRig"),
        ))
        .id();

    let body_joint = spawn_joint(commands, root, PlayerPart::Body, config);
    let body_mesh = spawn_mesh(
        commands,
        &cube,
        materials,
        body_joint,
        PlayerPart::Body,
        config,
    );

    let head_joint = spawn_joint(commands, root, PlayerPart::Head, config);
    let head_mesh = spawn_mesh(
        commands,
        &cube,
        materials,
        head_joint,
        PlayerPart::Head,
        config,
    );

    let right_arm = build_arm_chain(commands, root, &cube, materials, true, config);
    let left_arm = build_arm_chain(commands, root, &cube, materials, false, config);

    let held_item = spawn_held_item_anchor(commands, right_arm.hand_joint);
    let offhand = spawn_offhand_anchor(commands, left_arm.hand_joint);

    let thigh_r = spawn_joint(commands, root, PlayerPart::thigh_r(), config);
    let thigh_r_mesh = spawn_mesh(
        commands,
        &cube,
        materials,
        thigh_r,
        PlayerPart::thigh_r(),
        config,
    );
    let calf_r = spawn_joint(commands, thigh_r, PlayerPart::calf_r(), config);
    let calf_r_mesh = spawn_mesh(
        commands,
        &cube,
        materials,
        calf_r,
        PlayerPart::calf_r(),
        config,
    );

    let thigh_l = spawn_joint(commands, root, PlayerPart::thigh_l(), config);
    let thigh_l_mesh = spawn_mesh(
        commands,
        &cube,
        materials,
        thigh_l,
        PlayerPart::thigh_l(),
        config,
    );
    let calf_l = spawn_joint(commands, thigh_l, PlayerPart::calf_l(), config);
    let calf_l_mesh = spawn_mesh(
        commands,
        &cube,
        materials,
        calf_l,
        PlayerPart::calf_l(),
        config,
    );

    let helmet = spawn_helmet_anchor(commands, head_joint);
    let chest = spawn_chest_anchor(commands, body_joint);
    let back = spawn_back_anchor(commands, body_joint);

    let mesh_entities = vec![
        body_mesh,
        head_mesh,
        right_arm.upper_mesh,
        right_arm.forearm_mesh,
        right_arm.hand_mesh,
        left_arm.upper_mesh,
        left_arm.forearm_mesh,
        left_arm.hand_mesh,
        thigh_r_mesh,
        calf_r_mesh,
        thigh_l_mesh,
        calf_l_mesh,
    ];

    let entities = PlayerRigEntities {
        root,
        head_joint,
        body_joint,
        upper_arm_r: right_arm.upper_joint,
        upper_arm_l: left_arm.upper_joint,
        forearm_r: right_arm.forearm_joint,
        forearm_l: left_arm.forearm_joint,
        hand_r: right_arm.hand_joint,
        hand_l: left_arm.hand_joint,
        thigh_r,
        thigh_l,
        calf_r,
        calf_l,
        held_item,
        offhand,
        helmet,
        chest,
        back,
        head_mesh,
        mesh_entities,
    };

    (root, entities)
}

struct ArmRigEntities {
    upper_joint: Entity,
    forearm_joint: Entity,
    hand_joint: Entity,
    upper_mesh: Entity,
    forearm_mesh: Entity,
    hand_mesh: Entity,
}

fn build_arm_chain(
    commands: &mut Commands,
    root: Entity,
    cube: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    is_right: bool,
    config: &PlayerModelConfig,
) -> ArmRigEntities {
    let upper = if is_right {
        PlayerPart::upper_arm_r()
    } else {
        PlayerPart::upper_arm_l()
    };
    let forearm = if is_right {
        PlayerPart::forearm_r()
    } else {
        PlayerPart::forearm_l()
    };
    let hand = if is_right {
        PlayerPart::hand_r()
    } else {
        PlayerPart::hand_l()
    };

    let upper_joint = spawn_joint(commands, root, upper, config);
    let upper_mesh = spawn_mesh(commands, cube, materials, upper_joint, upper, config);

    let forearm_joint = spawn_joint(commands, upper_joint, forearm, config);
    let forearm_mesh = spawn_mesh(commands, cube, materials, forearm_joint, forearm, config);

    let hand_joint = spawn_joint(commands, forearm_joint, hand, config);
    let hand_mesh = spawn_mesh(commands, cube, materials, hand_joint, hand, config);

    ArmRigEntities {
        upper_joint,
        forearm_joint,
        hand_joint,
        upper_mesh,
        forearm_mesh,
        hand_mesh,
    }
}

fn spawn_joint(
    commands: &mut Commands,
    parent: Entity,
    part: PlayerPart,
    _config: &PlayerModelConfig,
) -> Entity {
    let offset = PlayerModelConfig::joint_offset(part);
    let entity = commands
        .spawn((
            PlayerJoint(part),
            Name::new(format!("Joint_{part:?}")),
            Transform::from_translation(offset),
            Visibility::default(),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

fn spawn_mesh(
    commands: &mut Commands,
    cube: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    joint: Entity,
    part: PlayerPart,
    config: &PlayerModelConfig,
) -> Entity {
    let offset = PlayerModelConfig::mesh_offset(part);
    let half = PlayerModelConfig::half_dims(part);
    let scale = Vec3::new(half.x * 2.0, half.y * 2.0, half.z * 2.0) * config.base_scale;
    let mat = materials.add(StandardMaterial {
        base_color: PlayerModelConfig::color(part),
        perceptual_roughness: 0.85,
        ..default()
    });
    let entity = commands
        .spawn((
            PlayerMesh(part),
            Name::new(format!("Mesh_{part:?}")),
            Mesh3d(cube.clone()),
            MeshMaterial3d(mat),
            Transform {
                translation: offset,
                scale,
                ..default()
            },
            Visibility::default(),
        ))
        .id();
    commands.entity(joint).add_child(entity);
    entity
}

fn spawn_held_item_anchor(commands: &mut Commands, parent: Entity) -> Entity {
    spawn_anchor(
        commands,
        parent,
        HeldItemAnchor,
        "HeldItemAnchor",
        held_item_grip_transform(),
    )
}

fn spawn_offhand_anchor(commands: &mut Commands, parent: Entity) -> Entity {
    spawn_anchor(
        commands,
        parent,
        OffHandAnchor,
        "OffHandAnchor",
        Transform {
            translation: Vec3::new(0.0, -0.13, -0.09),
            ..default()
        },
    )
}

fn spawn_helmet_anchor(commands: &mut Commands, parent: Entity) -> Entity {
    spawn_anchor(
        commands,
        parent,
        HelmetAnchor,
        "HelmetAnchor",
        Transform::default(),
    )
}

fn spawn_chest_anchor(commands: &mut Commands, parent: Entity) -> Entity {
    spawn_anchor(
        commands,
        parent,
        ChestAnchor,
        "ChestAnchor",
        Transform::default(),
    )
}

fn spawn_back_anchor(commands: &mut Commands, parent: Entity) -> Entity {
    spawn_anchor(
        commands,
        parent,
        BackAnchor,
        "BackAnchor",
        Transform::from_translation(Vec3::new(0.0, 0.12, 0.18)),
    )
}

fn spawn_anchor<B: Bundle>(
    commands: &mut Commands,
    parent: Entity,
    marker: B,
    name: &str,
    transform: Transform,
) -> Entity {
    let entity = commands
        .spawn((
            marker,
            Name::new(name.to_string()),
            transform,
            Visibility::default(),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}
