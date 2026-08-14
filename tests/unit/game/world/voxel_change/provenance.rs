use crate::game::world::voxel_change::provenance::VoxelProvenance;
use crate::shared::voxel::CHUNK_SIZE;
use crate::shared::voxel_change::ChangeSource;
use bevy::math::IVec3;

#[test]
fn records_and_queries_source() {
    let mut provenance = VoxelProvenance::default();
    let pos = IVec3::new(3, 4, 5);

    assert_eq!(provenance.source_of(pos), None);
    provenance.record(pos, ChangeSource::Ecology);
    assert_eq!(provenance.source_of(pos), Some(ChangeSource::Ecology));

    // 覆盖旧值
    provenance.record(pos, ChangeSource::Player);
    assert_eq!(provenance.source_of(pos), Some(ChangeSource::Player));
    assert_eq!(provenance.len(), 1);
}

#[test]
fn removes_single_entry() {
    let mut provenance = VoxelProvenance::default();
    provenance.record(IVec3::new(0, 0, 0), ChangeSource::Weather);

    assert_eq!(
        provenance.remove(IVec3::new(0, 0, 0)),
        Some(ChangeSource::Weather)
    );
    assert!(provenance.is_empty());
}

#[test]
fn remove_chunk_clears_only_that_chunk() {
    let mut provenance = VoxelProvenance::default();
    let chunk_size = CHUNK_SIZE as i32;

    // 目标区块 (0,0,0) 内两个坐标
    provenance.record(IVec3::new(1, 2, 3), ChangeSource::Player);
    provenance.record(IVec3::new(5, 6, 7), ChangeSource::Weather);
    // 相邻区块 (1,0,0) 内一个坐标，应保留
    provenance.record(IVec3::new(chunk_size + 1, 0, 0), ChangeSource::Fire);

    provenance.remove_chunk(IVec3::ZERO);

    assert_eq!(provenance.source_of(IVec3::new(1, 2, 3)), None);
    assert_eq!(provenance.source_of(IVec3::new(5, 6, 7)), None);
    assert_eq!(
        provenance.source_of(IVec3::new(chunk_size + 1, 0, 0)),
        Some(ChangeSource::Fire),
        "相邻区块记录不应被清理"
    );
    assert_eq!(provenance.len(), 1);
}
