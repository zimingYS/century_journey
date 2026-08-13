use super::*;
use crate::game::world::chunk::ChunkData;
use std::sync::Arc;

fn test_light() -> BlockLightDef {
    BlockLightDef {
        emission: 14,
        color: [1.0, 0.5, 0.25],
        range: 12,
        casts_shadow: true,
    }
}

#[test]
fn dependency_columns_add_one_horizontal_chunk_of_halo() {
    let columns = dependency_columns(&[IVec3::new(4, 2, -3)], 1);

    assert_eq!(columns.len(), 9);
    assert!(columns.contains(&(3, -4)));
    assert!(columns.contains(&(4, -3)));
    assert!(columns.contains(&(5, -2)));
}

#[test]
fn dependency_columns_expand_for_long_range_content() {
    let columns = dependency_columns(&[IVec3::ZERO], 2);

    assert_eq!(columns.len(), 25);
    assert!(columns.contains(&(-2, -2)));
    assert!(columns.contains(&(2, 2)));
}

#[test]
fn interaction_batches_the_complete_common_light_halo() {
    assert_eq!(
        local_column_batch_size(false, 2),
        LOCAL_TARGET_COLUMN_BATCH_SIZE
    );
    assert_eq!(local_column_batch_size(true, 1), 9);
    assert_eq!(local_column_batch_size(true, 2), 25);
    assert_eq!(local_column_batch_size(true, 4), 25);
}

#[test]
fn interaction_can_use_one_extra_local_lighting_slot() {
    assert!(local_lighting_slot_available(
        LOCAL_LIGHTING_MAX_IN_FLIGHT,
        true,
    ));
    assert!(!local_lighting_slot_available(
        LOCAL_LIGHTING_MAX_IN_FLIGHT,
        false,
    ));
    assert!(!local_lighting_slot_available(
        LOCAL_LIGHTING_MAX_IN_FLIGHT + 1,
        true,
    ));
}

#[test]
fn long_range_edit_enqueues_the_second_chunk_ring() {
    let mut world = WorldState::default();
    for x in -2..=2 {
        world.insert_chunk(IVec3::new(x, 0, 0), Arc::new(ChunkData::new()));
    }
    let mut queue = LocalLightingQueue::default();

    enqueue_block_change_targets(&world, IVec3::ZERO, 2, false, &mut queue);

    assert_eq!(queue.pop_columns(5).len(), 5);
}

#[test]
fn source_entry_is_added_replaced_and_removed_immediately() {
    let position = IVec3::new(2, 3, 4);
    let mut sources = Vec::new();

    assert!(update_source_entry(
        &mut sources,
        position,
        Some(test_light())
    ));
    assert_eq!(sources.len(), 1);
    assert!(!update_source_entry(
        &mut sources,
        position,
        Some(test_light())
    ));
    assert!(update_source_entry(&mut sources, position, None));
    assert!(sources.is_empty());
}
