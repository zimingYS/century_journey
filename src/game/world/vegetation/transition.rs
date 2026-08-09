//! 对树木阶段体素执行完整预检与原子替换，避免受阻生长留下半棵树。

use super::runtime::world_to_chunk_position;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::state::WorldState;
use crate::game::world::structure::TreeBlueprint;
use crate::shared::tag::identifier::TagId;
use crate::shared::voxel_change::{ChangeSource, VoxelChange, VoxelChangeBuffer};
use bevy::prelude::IVec3;
use std::collections::HashMap;

/// 描述阶段推进前世界中应当存在的树木体素形态。
pub(super) enum CurrentTreeForm<'a> {
    /// 树苗阶段只拥有根坐标上的一个树苗方块。
    Sapling(u16),
    /// 幼树阶段通过相同树种与形状种子重建当前蓝图。
    Blueprint(&'a TreeBlueprint),
}

/// 检查树根下方支撑方块已加载、可参与玩法且满足树苗支撑标签。
pub(super) fn support_is_valid(
    root: IVec3,
    support_tag: &TagId,
    tag_registry: &RuntimeTagRegistry,
    world_state: &WorldState,
    mut chunk_is_ready: impl FnMut(IVec3) -> bool,
) -> bool {
    let support_position = root - IVec3::Y;
    let support_chunk = world_to_chunk_position(support_position);
    world_state.contains_chunk(support_chunk)
        && chunk_is_ready(support_chunk)
        && tag_registry.contains(
            support_tag,
            get_voxel_at_world(support_position, world_state),
        )
}

/// 尝试把当前形态原子替换为目标蓝图（预检 + 推命令，不直接写世界）。
///
/// 参数较多（8 个）但都是原子操作的必需输入：根位置、支撑 tag、当前形态、
/// 目标蓝图、tag 注册表、只读世界（预检）、命令缓冲、区块就绪回调。
/// 打包成结构体会降低可读性，故保留直传参数（最小豁免）。
#[allow(clippy::too_many_arguments)]
pub(super) fn try_apply_stage_transition(
    root: IVec3,
    support_tag: &TagId,
    current_form: CurrentTreeForm<'_>,
    target_blueprint: &TreeBlueprint,
    tag_registry: &RuntimeTagRegistry,
    world_state: &WorldState,
    changes: &mut VoxelChangeBuffer,
    mut chunk_is_ready: impl FnMut(IVec3) -> bool,
) -> bool {
    if !support_is_valid(
        root,
        support_tag,
        tag_registry,
        world_state,
        &mut chunk_is_ready,
    ) {
        return false;
    }

    let current = current_voxels(root, current_form);
    let target = target_blueprint
        .voxels()
        .iter()
        .map(|voxel| (voxel.world_pos, voxel.block_id))
        .collect::<HashMap<_, _>>();
    if !current.contains_key(&root) || !target.contains_key(&root) {
        return false;
    }

    let mut positions = target.keys().copied().collect::<Vec<_>>();
    positions.sort_by_key(|position| (position.x, position.y, position.z));

    for &position in &positions {
        let chunk_position = world_to_chunk_position(position);
        if !world_state.contains_chunk(chunk_position) || !chunk_is_ready(chunk_position) {
            return false;
        }
        let actual = get_voxel_at_world(position, world_state);
        match current.get(&position) {
            Some(&expected) if position == root && actual != expected => return false,
            Some(&expected) if actual != 0 && actual != expected => return false,
            None if actual != 0 => return false,
            _ => {}
        }
    }

    for position in positions {
        let desired = target[&position];
        if get_voxel_at_world(position, world_state) != desired {
            changes.0.push(VoxelChange {
                pos: position,
                block_id: desired,
                source: ChangeSource::Vegetation,
            });
        }
    }
    true
}

fn current_voxels(root: IVec3, form: CurrentTreeForm<'_>) -> HashMap<IVec3, u16> {
    match form {
        CurrentTreeForm::Sapling(block_id) => HashMap::from([(root, block_id)]),
        CurrentTreeForm::Blueprint(blueprint) => blueprint
            .voxels()
            .iter()
            .map(|voxel| (voxel.world_pos, voxel.block_id))
            .collect(),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/vegetation/transition.rs"]
mod tests;
