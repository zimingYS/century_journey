//! 对树木阶段体素执行完整预检与原子替换，避免受阻生长留下半棵树。

use super::runtime::world_to_chunk_position;
use crate::content::block::event::BlockChangedEvent;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::world::block_ops::{get_voxel_at_world, set_voxel_at_world};
use crate::game::world::state::WorldState;
use crate::game::world::structure::TreeBlueprint;
use crate::shared::tag::identifier::TagId;
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

/// 完整校验目标蓝图后一次性扩展树体素，并返回实际发生的方块变化。
///
/// 旧阶段中不再属于目标蓝图的枝叶会原样保留，避免在没有体素来源标记时执行破坏性清理。
/// 目标位置只接受空气或当前阶段预期方块；所有检查在第一次写入前完成，因此区块缺失、
/// 支撑失效或任一位置受阻都不会产生部分结果。
pub(super) fn try_apply_stage_transition(
    root: IVec3,
    support_tag: &TagId,
    current_form: CurrentTreeForm<'_>,
    target_blueprint: &TreeBlueprint,
    tag_registry: &RuntimeTagRegistry,
    world_state: &mut WorldState,
    mut chunk_is_ready: impl FnMut(IVec3) -> bool,
) -> Option<Vec<BlockChangedEvent>> {
    if !support_is_valid(
        root,
        support_tag,
        tag_registry,
        world_state,
        &mut chunk_is_ready,
    ) {
        return None;
    }

    let current = current_voxels(root, current_form);
    let target = target_blueprint
        .voxels()
        .iter()
        .map(|voxel| (voxel.world_pos, voxel.block_id))
        .collect::<HashMap<_, _>>();
    if !current.contains_key(&root) || !target.contains_key(&root) {
        return None;
    }

    let mut positions = target.keys().copied().collect::<Vec<_>>();
    positions.sort_by_key(|position| (position.x, position.y, position.z));

    for &position in &positions {
        let chunk_position = world_to_chunk_position(position);
        if !world_state.contains_chunk(chunk_position) || !chunk_is_ready(chunk_position) {
            return None;
        }
        let actual = get_voxel_at_world(position, world_state);
        match current.get(&position) {
            Some(&expected) if position == root && actual != expected => return None,
            Some(&expected) if actual != 0 && actual != expected => return None,
            None if actual != 0 => return None,
            _ => {}
        }
    }

    let mut changes = Vec::with_capacity(positions.len());
    for position in positions {
        let desired = target[&position];
        if let Some(change) = set_voxel_at_world(position, desired, world_state) {
            changes.push(change);
        }
    }
    Some(changes)
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
