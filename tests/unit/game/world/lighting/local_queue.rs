use super::*;

#[test]
fn local_queue_preserves_priority_and_deduplicates() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::X);
    queue.enqueue(IVec3::ZERO);
    queue.enqueue(IVec3::X);
    queue.prioritize_edit(IVec3::ZERO, false);

    let columns = queue.pop_columns(8);
    assert_eq!(columns[0].column, (0, 0));
    assert_eq!(columns[1].column, (1, 0));
    assert!(queue.pop_columns(8).is_empty());
}

#[test]
fn interaction_target_can_be_dispatched_as_a_single_first_column() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::X);
    queue.prioritize_edit(IVec3::ZERO, false);

    assert_eq!(queue.pop_columns(1)[0].column, (0, 0));
    assert_eq!(queue.pop_columns(2)[0].column, (1, 0));
}

#[test]
fn selecting_a_column_removes_all_of_its_vertical_targets() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::new(2, -1, 3));
    queue.enqueue(IVec3::new(2, 0, 3));
    queue.enqueue(IVec3::new(2, 1, 3));

    assert_eq!(queue.pop_columns(1)[0].column, (2, 3));
    assert!(queue.pop_columns(1).is_empty());
}

#[test]
fn column_keeps_priority_from_any_vertical_target() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::new(2, -1, 3));
    queue.prioritize_edit(IVec3::new(2, 1, 3), false);

    assert!(queue.pop_columns(1)[0].priority);
}

#[test]
fn aged_target_bypasses_newer_interaction_priority() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::X);
    for _ in 0..LOCAL_LIGHTING_STARVATION_TICKS {
        queue.age();
    }
    queue.prioritize_edit(IVec3::ZERO, false);

    let selected = queue.pop_columns(1);
    assert_eq!(selected[0].column, (1, 0));
    assert!(selected[0].is_starved());
    assert!(!selected[0].priority);
}

#[test]
fn requeue_preserves_wait_time_across_missing_neighborhoods() {
    let mut queue = LocalLightingQueue::default();
    queue.requeue(
        IVec3::new(2, 0, 3),
        LOCAL_LIGHTING_STARVATION_TICKS,
        false,
        false,
    );

    assert!(queue.has_starved_target());
    assert!(queue.pop_columns(1)[0].is_starved());
}

#[test]
fn edit_merge_window_waits_two_fixed_ticks_after_last_change() {
    let mut queue = LocalLightingQueue::default();
    queue.restart_edit_merge_window();

    assert!(queue.wait_for_edit_merge());
    assert!(queue.wait_for_edit_merge());
    assert!(!queue.wait_for_edit_merge());
}

#[test]
fn interaction_target_bypasses_the_edit_merge_window() {
    let mut queue = LocalLightingQueue::default();
    queue.prioritize_edit(IVec3::ZERO, false);
    queue.restart_edit_merge_window();

    assert!(!queue.wait_for_edit_merge());
}

#[test]
fn column_preserves_sky_dirty_from_any_edit_target() {
    let mut queue = LocalLightingQueue::default();
    queue.prioritize_edit(IVec3::new(2, 0, 3), false);
    queue.prioritize_edit(IVec3::new(2, 1, 3), true);

    assert!(queue.pop_columns(1)[0].sky_dirty);
}

#[test]
fn continuous_edits_remain_immediately_dispatchable() {
    let mut queue = LocalLightingQueue::default();

    for _ in 0..8 {
        queue.prioritize_edit(IVec3::ZERO, false);
        queue.restart_edit_merge_window();
        queue.age();
        assert!(!queue.wait_for_edit_merge());
    }
}

#[test]
fn non_sky_neighbor_requeue_keeps_the_fast_path() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue_with_sky(IVec3::ZERO, false);

    assert!(!queue.pop_columns(1)[0].sky_dirty);
}

#[test]
fn initial_streaming_target_requires_sky_rebuild() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::ZERO);

    assert!(queue.pop_columns(1)[0].sky_dirty);
}
