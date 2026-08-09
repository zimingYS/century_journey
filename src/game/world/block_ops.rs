//! 提供世界方块读写的领域操作，统一处理跨区块坐标和脏标记。

use crate::content::block::event::BlockChangedEvent;
use crate::game::world::state::WorldState;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::prelude::*;
use std::sync::Arc;

/// 根据世界坐标获取方块 ID
pub fn get_voxel_at_world(world_pos: IVec3, world_state: &WorldState) -> u16 {
    let chunk_pos = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE as i32),
        world_pos.y.div_euclid(CHUNK_SIZE as i32),
        world_pos.z.div_euclid(CHUNK_SIZE as i32),
    );
    let local_x = world_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = world_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = world_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    world_state
        .chunk(chunk_pos)
        .map(|c| c.get_voxel(local_x, local_y, local_z))
        .unwrap_or(0)
}

/// 同步写入一个方块并返回变更事件（**渐进废弃中**）。
///
/// 生产调用方已全部迁移到 `VoxelChangeBuffer`（由 `apply_voxel_changes`
/// 统一应用）。目前仅测试用作 setup 工具；新代码一律走 buffer，
/// 待测试改造完成后删除本函数。
pub fn set_voxel_at_world(
    world_pos: IVec3,
    block_id: u16,
    world_state: &mut WorldState,
) -> Option<BlockChangedEvent> {
    let chunk_pos = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE as i32),
        world_pos.y.div_euclid(CHUNK_SIZE as i32),
        world_pos.z.div_euclid(CHUNK_SIZE as i32),
    );
    let local_x = world_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = world_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = world_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    let arc = world_state.chunk_mut(chunk_pos)?;
    let chunk_data = Arc::make_mut(arc);
    let old_block_id = chunk_data.get_voxel(local_x, local_y, local_z);
    if old_block_id == block_id {
        return None;
    }

    chunk_data.set_voxel(local_x, local_y, local_z, block_id);
    Some(BlockChangedEvent {
        world_pos,
        old_block_id,
        new_block_id: block_id,
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/game/world/block_ops.rs"]
mod tests;
