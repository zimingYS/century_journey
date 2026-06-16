use bevy::prelude::*;
use crate::player::model::components::{
    PlayerJoint, PlayerMesh, PlayerModelMarker, PlayerPart, PlayerRig,
    HeldItemAnchor, OffHandAnchor, HelmetAnchor, ChestAnchor, BackAnchor,
};
use crate::player::model::config::PlayerModelConfig;

/// 缓存所有关节 Entity
#[derive(Resource, Clone)]
pub struct PlayerRigEntities {
    pub head_joint:  Entity,
    pub body_joint:  Entity,
    pub upper_arm_r: Entity, pub upper_arm_l: Entity,
    pub forearm_r:   Entity, pub forearm_l:   Entity,
    pub hand_r:      Entity, pub hand_l:      Entity,
    pub thigh_r:     Entity, pub thigh_l:     Entity,
    pub calf_r:      Entity, pub calf_l:      Entity,
    pub held_item:   Entity,
    pub offhand:     Entity,
    pub helmet:      Entity,
    pub chest:       Entity,
    pub back:        Entity,
}

pub fn spawn_player_rig_v2(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &PlayerModelConfig,
) -> (Entity, PlayerRigEntities) {
    let cube = meshes.add(Cuboid::default());

    let root = commands.spawn((
        PlayerRig, PlayerModelMarker,
        Transform::from_xyz(0.0, 0.0, 0.0), Visibility::default(),
        Name::new("PlayerRig"),
    )).id();

    // Body + Head
    let body_joint = spawn_joint(commands, root, PlayerPart::Body, config);
    spawn_mesh(commands, &cube, materials, body_joint, PlayerPart::Body, config);

    let head_joint = spawn_joint(commands, root, PlayerPart::Head, config);
    spawn_mesh(commands, &cube, materials, head_joint, PlayerPart::Head, config);

    // Right arm
    let (ua_r, fa_r, ha_r) = build_arm_chain(commands, root, &cube, materials, true, config);
    // Left arm
    let (ua_l, fa_l, ha_l) = build_arm_chain(commands, root, &cube, materials, false, config);

    let held_item = spawn_anchor(commands, ha_r, "HeldItemAnchor");
    let offhand   = spawn_anchor(commands, ha_l, "OffHandAnchor");

    // Legs
    let thigh_r = spawn_joint(commands, root, PlayerPart::thigh_r(), config);
    spawn_mesh(commands, &cube, materials, thigh_r, PlayerPart::thigh_r(), config);
    let calf_r = spawn_joint(commands, thigh_r, PlayerPart::calf_r(), config);
    spawn_mesh(commands, &cube, materials, calf_r, PlayerPart::calf_r(), config);

    let thigh_l = spawn_joint(commands, root, PlayerPart::thigh_l(), config);
    spawn_mesh(commands, &cube, materials, thigh_l, PlayerPart::thigh_l(), config);
    let calf_l = spawn_joint(commands, thigh_l, PlayerPart::calf_l(), config);
    spawn_mesh(commands, &cube, materials, calf_l, PlayerPart::calf_l(), config);

    // Equipment anchors
    let helmet = spawn_anchor(commands, head_joint, "HelmetAnchor");
    let chest  = spawn_anchor(commands, body_joint, "ChestAnchor");
    let back   = spawn_anchor(commands, body_joint, "BackAnchor");

    let entities = PlayerRigEntities {
        head_joint, body_joint,
        upper_arm_r: ua_r, upper_arm_l: ua_l,
        forearm_r: fa_r, forearm_l: fa_l,
        hand_r: ha_r, hand_l: ha_l,
        thigh_r, thigh_l,
        calf_r, calf_l,
        held_item, offhand, helmet, chest, back,
    };
    commands.insert_resource(entities.clone());

    (root, entities)
}

fn build_arm_chain(
    commands: &mut Commands, root: Entity,
    cube: &Handle<Mesh>, materials: &mut ResMut<Assets<StandardMaterial>>,
    is_right: bool, config: &PlayerModelConfig,
) -> (Entity, Entity, Entity) {
    let upper  = if is_right { PlayerPart::upper_arm_r() } else { PlayerPart::upper_arm_l() };
    let forearm = if is_right { PlayerPart::forearm_r() } else { PlayerPart::forearm_l() };
    let hand    = if is_right { PlayerPart::hand_r() } else { PlayerPart::hand_l() };

    let uj = spawn_joint(commands, root, upper, config);
    spawn_mesh(commands, cube, materials, uj, upper, config);

    let fj = spawn_joint(commands, uj, forearm, config);
    spawn_mesh(commands, cube, materials, fj, forearm, config);

    let hj = spawn_joint(commands, fj, hand, config);
    spawn_mesh(commands, cube, materials, hj, hand, config);

    (uj, fj, hj)
}

fn spawn_joint(commands: &mut Commands, parent: Entity, part: PlayerPart, config: &PlayerModelConfig) -> Entity {
    let offset = PlayerModelConfig::joint_offset(part);
    let entity = commands.spawn((
        PlayerJoint(part),
        Name::new(format!("Joint_{:?}", part)),
        Transform::from_translation(offset),
        Visibility::default(),
    )).id();
    commands.entity(parent).add_child(entity);
    entity
}

fn spawn_mesh(
    commands: &mut Commands, cube: &Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    joint: Entity, part: PlayerPart, config: &PlayerModelConfig,
) -> Entity {
    let offset = PlayerModelConfig::mesh_offset(part);
    let half = PlayerModelConfig::half_dims(part);
    let scale = Vec3::new(half.x * 2.0, half.y * 2.0, half.z * 2.0) * config.base_scale;
    let mat = materials.add(StandardMaterial {
        base_color: PlayerModelConfig::color(part),
        perceptual_roughness: 0.85,
        ..default()
    });
    let entity = commands.spawn((
        PlayerMesh(part),
        Name::new(format!("Mesh_{:?}", part)),
        Mesh3d(cube.clone()),
        MeshMaterial3d(mat.clone()),
        Transform { translation: offset, scale, ..default() },
        Visibility::default(),
    )).id();
    commands.entity(joint).add_child(entity);
    entity
}

fn spawn_anchor(commands: &mut Commands, parent: Entity, name: &str) -> Entity {
    let e = commands.spawn((
        HeldItemAnchor,  // always used, specific type filtered later
        Name::new(name.to_string()),
        Transform::default(),
        Visibility::default(),
    )).id();
    commands.entity(parent).add_child(e);
    e
}
