use super::*;

#[test]
fn local_queue_preserves_priority_and_deduplicates() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::X);
    queue.enqueue(IVec3::ZERO);
    queue.enqueue(IVec3::X);
    queue.prioritize(IVec3::ZERO);

    assert_eq!(queue.pop_columns(8), vec![(0, 0), (1, 0)]);
    assert!(queue.pop_columns(8).is_empty());
}

#[test]
fn interaction_target_can_be_dispatched_as_a_single_first_column() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::X);
    queue.prioritize(IVec3::ZERO);
    queue.interaction_pending = true;

    assert_eq!(queue.pop_columns(1), vec![(0, 0)]);
    assert_eq!(queue.pop_columns(2), vec![(1, 0)]);
}

#[test]
fn selecting_a_column_removes_all_of_its_vertical_targets() {
    let mut queue = LocalLightingQueue::default();
    queue.enqueue(IVec3::new(2, -1, 3));
    queue.enqueue(IVec3::new(2, 0, 3));
    queue.enqueue(IVec3::new(2, 1, 3));

    assert_eq!(queue.pop_columns(1), vec![(2, 3)]);
    assert!(queue.pop_columns(1).is_empty());
}
