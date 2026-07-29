use super::*;
use crate::game::world::chunk::ChunkData;

#[test]
fn write_reports_only_actual_block_changes() {
    let mut world_state = WorldState::default();
    world_state.insert_chunk(IVec3::ZERO, Arc::new(ChunkData::new()));
    let world_pos = IVec3::new(2, 3, 4);

    let change =
        set_voxel_at_world(world_pos, 9, &mut world_state).expect("write should change air");
    assert_eq!(change.world_pos, world_pos);
    assert_eq!(change.old_block_id, 0);
    assert_eq!(change.new_block_id, 9);
    assert_eq!(get_voxel_at_world(world_pos, &world_state), 9);
    assert!(set_voxel_at_world(world_pos, 9, &mut world_state).is_none());
}

#[test]
fn write_to_unloaded_chunk_does_not_report_a_change() {
    let mut world_state = WorldState::default();

    assert!(set_voxel_at_world(IVec3::new(16, 0, 0), 9, &mut world_state).is_none());
}
