use super::*;
use crate::game::world::block_ops::{get_voxel_at_world, set_voxel_at_world};
use crate::game::world::chunk::ChunkData;
use crate::game::world::state::ChunkRuntime;
use crate::shared::voxel_change::ChangeSource;
use bevy::math::IVec3;
use std::sync::Arc;

fn state_with_chunk() -> WorldState {
    let mut state = WorldState::default();
    state.insert_chunk(IVec3::ZERO, Arc::new(ChunkData::new()));
    state
}

fn change(pos: IVec3, block_id: u16) -> VoxelChange {
    VoxelChange {
        pos,
        block_id,
        source: ChangeSource::Player,
    }
}

#[test]
fn applies_change_and_bumps_revision() {
    let mut world = state_with_chunk();
    let mut runtime = ChunkRuntime::default();
    let mut buffer = VoxelChangeBuffer(vec![change(IVec3::new(1, 2, 3), 9)]);

    let applied = apply_changes(&mut world, &mut runtime, &mut buffer);

    assert_eq!(applied.len(), 1);
    assert_eq!(get_voxel_at_world(IVec3::new(1, 2, 3), &world), 9);
    assert_eq!(runtime.revision(IVec3::ZERO), 1);
    assert!(buffer.0.is_empty(), "buffer 应用后应清空");
}

#[test]
fn no_op_change_does_not_bump_revision() {
    let mut world = state_with_chunk();
    set_voxel_at_world(IVec3::new(1, 2, 3), 9, &mut world);
    let mut runtime = ChunkRuntime::default();
    let mut buffer = VoxelChangeBuffer(vec![change(IVec3::new(1, 2, 3), 9)]);

    let applied = apply_changes(&mut world, &mut runtime, &mut buffer);

    assert!(applied.is_empty());
    assert_eq!(runtime.revision(IVec3::ZERO), 0, "无变化不递增修订号");
}

#[test]
fn unloaded_chunk_is_skipped() {
    let mut world = WorldState::default();
    let mut runtime = ChunkRuntime::default();
    let mut buffer = VoxelChangeBuffer(vec![change(IVec3::new(16, 0, 0), 9)]);

    let applied = apply_changes(&mut world, &mut runtime, &mut buffer);

    assert!(applied.is_empty());
    assert_eq!(runtime.revision(IVec3::new(1, 0, 0)), 0);
}

#[test]
fn order_preserved_across_chunks() {
    let mut world = state_with_chunk();
    let mut runtime = ChunkRuntime::default();
    // 同一区块内两个变更，跨区块也各自应用
    let mut buffer = VoxelChangeBuffer(vec![
        change(IVec3::new(1, 2, 3), 9),
        change(IVec3::new(2, 2, 3), 7),
        change(IVec3::new(17, 0, 0), 5), // 相邻区块（未加载 → 跳过）
    ]);

    let applied = apply_changes(&mut world, &mut runtime, &mut buffer);

    assert_eq!(applied.len(), 2);
    assert_eq!(get_voxel_at_world(IVec3::new(1, 2, 3), &world), 9);
    assert_eq!(get_voxel_at_world(IVec3::new(2, 2, 3), &world), 7);
    assert_eq!(runtime.revision(IVec3::ZERO), 2);
}
