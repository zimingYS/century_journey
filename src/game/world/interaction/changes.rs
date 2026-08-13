//! 定义并应用权威方块变更，使存档、网格和邻居更新共享同一事实源。

use crate::content::block::event::{BlockChangedEvent, BlockNeighborChangedEvent};
use crate::game::world::chunk::{ChunkComponents, ChunkState};
use crate::game::world::state::WorldState;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::prelude::{Entity, IVec3, MessageReader, MessageWriter, Query, ResMut};

const NEIGHBOR_OFFSETS: [IVec3; 6] = [
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(1, 0, 0),
    IVec3::new(0, 0, -1),
    IVec3::new(0, 0, 1),
];

/// 将已提交的方块写入转换为区块刷新和六方向邻居通知。
///
/// 本系统在固定步的玩家交互之后执行。所有运行时写入都先产生 `BlockChangedEvent`，
/// 因而环境规则不需要依赖具体的玩家、液体或未来生长系统。
pub(super) fn propagate_block_changes_system(
    mut changes: MessageReader<BlockChangedEvent>,
    mut neighbor_changes: MessageWriter<BlockNeighborChangedEvent>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
    mut world_state: ResMut<WorldState>,
) {
    for change in changes.read() {
        mark_changed_chunks_dirty(change.world_pos, &mut chunk_query, &mut world_state);

        for offset in NEIGHBOR_OFFSETS {
            neighbor_changes.write(BlockNeighborChangedEvent {
                neighbor_pos: change.world_pos + offset,
                changed_pos: change.world_pos,
                old_block_id: change.old_block_id,
                new_block_id: change.new_block_id,
            });
        }
    }
}

/// 标记受单格写入影响的区块及边界相邻区块重新构建网格和保存快照。
fn mark_changed_chunks_dirty(
    world_pos: IVec3,
    chunk_query: &mut Query<(Entity, &ChunkComponents, &mut ChunkState)>,
    world_state: &mut WorldState,
) {
    let chunk_pos = IVec3::new(
        world_pos.x.div_euclid(CHUNK_SIZE as i32),
        world_pos.y.div_euclid(CHUNK_SIZE as i32),
        world_pos.z.div_euclid(CHUNK_SIZE as i32),
    );
    let local = IVec3::new(
        world_pos.x.rem_euclid(CHUNK_SIZE as i32),
        world_pos.y.rem_euclid(CHUNK_SIZE as i32),
        world_pos.z.rem_euclid(CHUNK_SIZE as i32),
    );
    let max_index = CHUNK_SIZE as i32 - 1;
    let mut dirty_chunks = vec![chunk_pos];

    if local.y == 0 {
        dirty_chunks.push(chunk_pos + IVec3::NEG_Y);
    }
    if local.y == max_index {
        dirty_chunks.push(chunk_pos + IVec3::Y);
    }
    if local.x == 0 {
        dirty_chunks.push(chunk_pos + IVec3::NEG_X);
    }
    if local.x == max_index {
        dirty_chunks.push(chunk_pos + IVec3::X);
    }
    if local.z == 0 {
        dirty_chunks.push(chunk_pos + IVec3::NEG_Z);
    }
    if local.z == max_index {
        dirty_chunks.push(chunk_pos + IVec3::Z);
    }

    // 存档修改时间需要真实时间戳，而非会暂停或加速的世界模拟时间。
    let modified_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    for &dirty_chunk in &dirty_chunks {
        world_state.mark_chunk_modified(dirty_chunk, modified_at);
    }

    for (_, components, mut state) in chunk_query.iter_mut() {
        if dirty_chunks.contains(&components.position) {
            *state = ChunkState::LightingPending;
        }
    }
}
