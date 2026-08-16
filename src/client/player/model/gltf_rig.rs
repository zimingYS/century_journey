//! 从 Blockbench 导出的 `player.glb` 加载玩家全身模型，并建立命名骨骼锚点映射。
//!
//! 模型以 `armature=false` 导出：每个肢体 cube 是独立 mesh（head/body/上臂/前臂/手/大腿/小腿
//! 共 12 个），group 作为关节 node 保留层级。这样程序化动画可以直接旋转关节 node 驱动四肢，
//! 第一人称也能单独隐藏头/身/腿网格、保留手臂网格，实现 MC 风格的"第一人称看到手"。
//!
//! 场景由 `bevy_world_serialization` 的 `WorldAssetRoot` 异步实例化。Bevy 0.19 把 `SceneRoot`
//! 重命名为 `WorldAssetRoot`，加载完成通过 `WorldInstanceReady` observer 事件通知。我们订阅
//! 这个事件，在回调里通过 `Children` + `Name` 找到 glTF 节点，把 Entity 写回 `PlayerRigEntities`
//! 供第一人称可见性、装备和手持物品挂点消费。

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy::world_serialization::{WorldAssetRoot, WorldInstanceReady};

use crate::client::player::model::components::{PlayerPart, PlayerRig};
use crate::client::player::model::rig::{PlayerRigEntities, held_item_grip_transform};

/// glTF 资源在 assets 下的相对路径。
///
/// 配合 `GltfAssetLabel::Scene(0)` 让 Bevy 只解析第一个 scene，
/// 不必先把完整 `Gltf` 资源加载完再访问 `scenes` 字段。
pub const PLAYER_GLTF_PATH: &str = "models/player/player.glb";

/// 标记需要等异步场景加载完成、然后建立骨骼映射的实体。
///
/// 记录目标 `player` 实体：`PlayerRigEntities` 必须插到 **player（LocalPlayer）实体**上，
/// 而不是 scene root——所有下游系统（first_person_visibility / full_body / animation_pose）
/// 都用 `With<LocalPlayer>` 查询这份组件。scene root（WorldAssetRoot 实体）只是承载
/// glTF 场景节点层级。
#[derive(Component)]
pub struct PendingPlayerRigBind {
    /// `PlayerRigEntities` 的最终挂载实体（本地玩家实体 / 预览根实体）。
    pub player: Entity,
}

/// 在游戏启动时为本地玩家实体生成 glTF 场景实例，并把 `PendingPlayerRigBind`
/// 一起插入，等待后续映射系统填入 `PlayerRigEntities`。
pub fn spawn_glb_player_rig(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent: Entity,
    player: Entity,
    name: &'static str,
) -> Entity {
    let entity = commands
        .spawn((
            Name::new(name),
            PlayerRig,
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset(PLAYER_GLTF_PATH)),
            ),
            Transform::default(),
            Visibility::default(),
            PendingPlayerRigBind { player },
        ))
        .id();
    commands.entity(parent).add_child(entity);
    entity
}

/// 订阅 `WorldInstanceReady` 事件：当带 `PendingPlayerRigBind` 的实体被实例化完成后，
/// 跑一次骨骼映射，把 `PlayerRigEntities` 插进去并清除待绑定标记。
///
/// `WorldInstanceReady` 是 Bevy 0.19 的 `EntityEvent`（用 `world.commands().trigger` 发出），
/// 订阅方要用 `On<WorldInstanceReady>` 作为参数；不能走 `MessageReader` 通道。
/// 调用方需要在 `PlayerModelPlugin::build` 里 `app.add_observer(bind_player_rig_on_ready)`。
pub fn bind_player_rig_on_ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    pending_query: Query<&PendingPlayerRigBind>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    mesh_query: Query<(), With<Mesh3d>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    let scene_root = event.entity;
    // 只处理本地玩家 / 预览（带 PendingPlayerRigBind 的实体）
    let Ok(pending) = pending_query.get(scene_root) else {
        return;
    };
    let player = pending.player;

    // 1) 收集场景里所有命名节点，建立 head/body/right_hand 等的 name→entity 映射。
    //    关节节点（group）在层级里先于同名 mesh（cube），or_insert 保证映射到关节。
    let mut by_name: std::collections::HashMap<String, Entity> = std::collections::HashMap::new();
    collect_named_descendants(scene_root, &children_query, &name_query, &mut by_name);
    let find = |name: &str| -> Option<Entity> { by_name.get(name).copied() };

    // 2) 写入 PlayerRigEntities（给挂点/装备/手持物系统用）。
    //    group 名已与人体左右一致：right_arm = 玩家右臂，直接 1:1 对应。
    let upper_arm_r = find("right_arm").unwrap_or(scene_root);
    let upper_arm_l = find("left_arm").unwrap_or(scene_root);
    let thigh_r = find("right_leg").unwrap_or(scene_root);
    let thigh_l = find("left_leg").unwrap_or(scene_root);
    let head_joint = find("head").unwrap_or(scene_root);
    let body_joint = find("body").unwrap_or(scene_root);
    let hand_r = find("right_hand").unwrap_or(upper_arm_r);
    let hand_l = find("left_hand").unwrap_or(upper_arm_l);

    // 手持物挂点：right_hand 关节的子实体，带握持偏移。必须独立于 hand_r 关节——
    // animation_pose 每帧覆盖 held_item 的 Transform（握持 + 动作摆动），若直接复用
    // hand 关节会把关节旋转覆盖掉，导致手/物品位置错乱（物品贴在小臂上的根因之一）。
    let held_item = commands
        .spawn((
            Name::new("HeldItemAnchor"),
            held_item_grip_transform(),
            Visibility::default(),
        ))
        .id();
    commands.entity(hand_r).add_child(held_item);

    let rig_entities = PlayerRigEntities {
        root: scene_root,
        head_joint,
        body_joint,
        upper_arm_r,
        upper_arm_l,
        forearm_r: find("right_forearm").unwrap_or(upper_arm_r),
        forearm_l: find("left_forearm").unwrap_or(upper_arm_l),
        hand_r,
        hand_l,
        thigh_r,
        thigh_l,
        calf_r: find("right_calf").unwrap_or(thigh_r),
        calf_l: find("left_calf").unwrap_or(thigh_l),
        // glTF 模型是 MC 4×4（leg/calf），没有独立的 foot 节点——用 calf 当 fallback
        foot_r: find("right_calf").unwrap_or(thigh_r),
        foot_l: find("left_calf").unwrap_or(thigh_l),
        held_item,
        offhand: hand_l,
        helmet: head_joint,
        chest: body_joint,
        back: body_joint,
        head_mesh: head_joint,
        // 手臂 mesh 与身体 mesh 分开收集，供第一人称单独隐藏头/身/腿。
        arm_meshes: collect_arm_meshes(upper_arm_r, upper_arm_l, &children_query, &mesh_query),
        body_meshes: collect_body_meshes(
            head_joint,
            body_joint,
            thigh_r,
            thigh_l,
            &children_query,
            &mesh_query,
        ),
        mesh_entities: Vec::new(),
    };

    // 所有 mesh = 手臂 mesh + 身体 mesh（用于渲染层同步等整体操作）。
    let mut all_meshes = rig_entities.arm_meshes.clone();
    all_meshes.extend(rig_entities.body_meshes.iter().copied());
    let arm_mesh_count = rig_entities.arm_meshes.len();
    let body_mesh_count = rig_entities.body_meshes.len();

    // 玩家模型 glb 以 `doubleSided=True` 导出，Bevy 据此把材质设为 `cull_mode=None`
    // （双面渲染）。第一人称相机位于 head 几何体内部，若保持双面渲染会看到 head 的
    // 内表面（穿模遮挡视线）。这里显式把所有玩家 mesh 的材质改为单面（`cull_mode=Back`）：
    // 从外部（第三人称、阴影相机）看正面正常渲染并投射完整阴影；从内部（相机在 head 内）
    // 看背面被剔除，head 自动隐形，无需再用 Visibility 隐藏。
    for mesh_entity in &all_meshes {
        if let Ok(material_handle) = mesh_material_query.get(*mesh_entity) {
            if let Some(mut material) = materials.get_mut(&material_handle.0) {
                material.cull_mode = Some(Face::Back);
            }
        }
    }

    let mut rig_entities = rig_entities;
    rig_entities.mesh_entities = all_meshes;

    commands.entity(player).insert(rig_entities);
    commands.entity(scene_root).remove::<PendingPlayerRigBind>();
    info!(
        "[gltf] 玩家 rig 绑定完成: 命名节点={} 手臂mesh={} 身体mesh={}",
        by_name.len(),
        arm_mesh_count,
        body_mesh_count,
    );
}

/// 递归遍历 `scene_root` 的后代，把带 `Name` 的节点塞进 `by_name`。
fn collect_named_descendants(
    root: Entity,
    children_query: &Query<&Children>,
    name_query: &Query<&Name>,
    by_name: &mut std::collections::HashMap<String, Entity>,
) {
    let Ok(children) = children_query.get(root) else {
        return;
    };
    for child in children.iter() {
        if let Ok(name) = name_query.get(child) {
            // Bevy 0.19 的 `Name` 持有 `Box<str>`，`as_str()` 借用 `&str`
            let s = name.as_str();
            by_name.entry(s.to_string()).or_insert(child);
        }
        collect_named_descendants(child, children_query, name_query, by_name);
    }
}

/// 收集某关节子树里所有带 `Mesh3d` 的实体（用于第一人称可见性和渲染层同步）。
fn collect_mesh_descendants(
    root: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<(), With<Mesh3d>>,
    out: &mut Vec<Entity>,
) {
    if mesh_query.get(root).is_ok() {
        out.push(root);
    }
    if let Ok(children) = children_query.get(root) {
        for child in children.iter() {
            collect_mesh_descendants(child, children_query, mesh_query, out);
        }
    }
}

/// 收集左右手臂子树里的全部 mesh（上臂/前臂/手）。
fn collect_arm_meshes(
    upper_arm_r: Entity,
    upper_arm_l: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<(), With<Mesh3d>>,
) -> Vec<Entity> {
    let mut out = Vec::new();
    collect_mesh_descendants(upper_arm_r, children_query, mesh_query, &mut out);
    collect_mesh_descendants(upper_arm_l, children_query, mesh_query, &mut out);
    out
}

/// 收集头/身/腿子树里的全部 mesh。
fn collect_body_meshes(
    head: Entity,
    body: Entity,
    thigh_r: Entity,
    thigh_l: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<(), With<Mesh3d>>,
) -> Vec<Entity> {
    let mut out = Vec::new();
    for joint in [head, body, thigh_r, thigh_l] {
        collect_mesh_descendants(joint, children_query, mesh_query, &mut out);
    }
    out
}

/// 工具：把 PlayerPart 枚举映射回 glTF 节点的 name（用于将来做节点级动画控制时定位）。
pub fn part_to_glb_name(part: PlayerPart) -> &'static str {
    match part {
        PlayerPart::Head => "head",
        PlayerPart::Body => "body",
        PlayerPart::UpperArmL(true) => "right_arm",
        PlayerPart::UpperArmL(false) => "left_arm",
        PlayerPart::ForearmL(true) => "right_forearm",
        PlayerPart::ForearmL(false) => "left_forearm",
        PlayerPart::HandL(true) => "right_hand",
        PlayerPart::HandL(false) => "left_hand",
        PlayerPart::ThighL(true) => "right_leg",
        PlayerPart::ThighL(false) => "left_leg",
        PlayerPart::CalfL(true) => "right_calf",
        PlayerPart::CalfL(false) => "left_calf",
        PlayerPart::FootL(true) => "right_calf",
        PlayerPart::FootL(false) => "left_calf",
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/player/model/gltf_rig.rs"]
mod tests;
