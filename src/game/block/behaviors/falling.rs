//! 重力方块行为：下方变空时下落一格。

use crate::content::block::behavior::BlockBehavior;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::state::WorldState;
use crate::shared::voxel_change::{ChangeSource, VoxelChange, VoxelChangeBuffer};
use bevy::prelude::{Commands, IVec3};

/// 受重力影响的方块（沙/砂砾等）。
///
/// 由行为分发系统驱动：`on_neighbor_update`（下方支撑被挖）或
/// `on_change`（悬空放置）。每次只下落一格，apply 后下方再次变化
/// 触发链式下落。
pub struct FallingBlockBehavior;

impl BlockBehavior<WorldState> for FallingBlockBehavior {
    fn on_neighbor_update(
        &self,
        world_pos: IVec3,
        block_id: u16,
        neighbor_pos: IVec3,
        neighbor_block_id: u16,
        world_storage: &WorldState,
        changes: &mut VoxelChangeBuffer,
        _commands: &mut Commands,
    ) {
        // 只响应下方邻居变化；下方变空才下落（v1：水/熔岩沉没后期做）
        if neighbor_pos != world_pos - IVec3::Y || neighbor_block_id != 0 {
            return;
        }
        push_fall(world_pos, block_id, world_storage, changes);
    }

    fn on_change(
        &self,
        world_pos: IVec3,
        block_id: u16,
        world_storage: &WorldState,
        changes: &mut VoxelChangeBuffer,
        _commands: &mut Commands,
    ) {
        // 放置后检查：下方为空则下落（悬空放沙）
        push_fall(world_pos, block_id, world_storage, changes);
    }
}

/// 尝试让方块下落一格：自身仍在原位且下方为空才推命令。
fn push_fall(
    world_pos: IVec3,
    block_id: u16,
    world_storage: &WorldState,
    changes: &mut VoxelChangeBuffer,
) {
    // 自身已被挖掉/替换：跳过（事件可能滞后）
    if get_voxel_at_world(world_pos, world_storage) != block_id {
        return;
    }
    let below = world_pos - IVec3::Y;
    if get_voxel_at_world(below, world_storage) != 0 {
        return;
    }

    // 下落 = 当前位置清空 + 下方写入。来源用 WorldGen（"世界自身行为"，
    // 落下的方块仍视为自然方块；Provenance v1 落地时再考虑来源继承）。
    changes.0.push(VoxelChange {
        pos: world_pos,
        block_id: 0,
        source: ChangeSource::WorldGen,
    });
    changes.0.push(VoxelChange {
        pos: below,
        block_id,
        source: ChangeSource::WorldGen,
    });
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/block/behaviors/falling.rs"]
mod tests;
