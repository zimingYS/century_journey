//! 世界方块变更的唯一应用点。
//!
//! 核心逻辑抽为纯函数 `apply_changes`（可白盒测试），系统只做事件转发。

use crate::content::block::event::BlockChangedEvent;
use crate::game::world::state::ChunkRuntime;
use crate::game::world::state::WorldState;
use crate::shared::voxel::CHUNK_SIZE;
use crate::shared::voxel_change::{VoxelChange, VoxelChangeBuffer};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// 应用缓冲内全部变更，返回实际发生的变化（纯函数，可白盒测试）。
///
/// 按区块分组减少哈希查找；逐格递增修订号；`old == new` 跳过（不 bump 不返回）；
/// 区块未加载跳过；提交顺序 = 应用顺序（确定性）；应用后清空缓冲。
pub fn apply_changes(
    world_state: &mut WorldState,
    runtime: &mut ChunkRuntime,
    buffer: &mut VoxelChangeBuffer,
) -> Vec<BlockChangedEvent> {
    let mut applied = Vec::new();
    if buffer.0.is_empty() {
        return applied;
    }

    let mut grouped: HashMap<IVec3, Vec<VoxelChange>> = HashMap::new();
    for change in buffer.0.drain(..) {
        let chunk_pos = IVec3::new(
            change.pos.x.div_euclid(CHUNK_SIZE as i32),
            change.pos.y.div_euclid(CHUNK_SIZE as i32),
            change.pos.z.div_euclid(CHUNK_SIZE as i32),
        );
        grouped.entry(chunk_pos).or_default().push(change);
    }

    for (chunk_pos, changes) in grouped {
        let Some(chunk_data) = world_state.chunk_mut(chunk_pos) else {
            continue; // 区块未加载：跳过
        };
        let chunk_data = Arc::make_mut(chunk_data);

        for change in changes {
            let local = IVec3::new(
                change.pos.x.rem_euclid(CHUNK_SIZE as i32),
                change.pos.y.rem_euclid(CHUNK_SIZE as i32),
                change.pos.z.rem_euclid(CHUNK_SIZE as i32),
            );
            let old_block_id =
                chunk_data.get_voxel(local.x as usize, local.y as usize, local.z as usize);
            if old_block_id == change.block_id {
                continue; // 无实际变化
            }
            chunk_data.set_voxel(
                local.x as usize,
                local.y as usize,
                local.z as usize,
                change.block_id,
            );
            runtime.bump_revision(chunk_pos);
            applied.push(BlockChangedEvent {
                world_pos: change.pos,
                old_block_id,
                new_block_id: change.block_id,
            });
        }
    }
    applied
}

/// 固定步系统：应用缓冲并把变化转成事件。
pub fn apply_voxel_changes(
    mut buffer: ResMut<VoxelChangeBuffer>,
    mut world_state: ResMut<WorldState>,
    mut runtime: ResMut<ChunkRuntime>,
    mut changed_blocks: MessageWriter<BlockChangedEvent>,
) {
    for change in apply_changes(&mut world_state, &mut runtime, &mut buffer) {
        changed_blocks.write(change);
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/voxel_change/apply.rs"]
mod tests;
