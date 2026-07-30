use super::*;
use crate::shared::identifier::Identifier;

fn tree(root: IVec3, seed: u32) -> TreeInstance {
    TreeInstance::new_mature(root, Identifier::new("century_journey", "oak"), seed, 10)
}

#[test]
fn snapshots_are_sorted_and_duplicate_roots_keep_original_instance() {
    let mut store = TreeInstanceStore::default();
    store.insert(tree(IVec3::new(3, 2, 1), 30)).unwrap();
    store.insert(tree(IVec3::new(1, 2, 3), 10)).unwrap();

    assert!(store.insert(tree(IVec3::new(1, 2, 3), 99)).is_err());
    let snapshot = store.snapshot_chunk(IVec3::ZERO);
    assert_eq!(
        snapshot.iter().map(TreeInstance::root).collect::<Vec<_>>(),
        vec![IVec3::new(1, 2, 3), IVec3::new(3, 2, 1)]
    );
    assert_eq!(snapshot[0].shape_seed(), 10);
}

#[test]
fn invalid_chunk_replacement_is_atomic_and_take_removes_bucket() {
    let mut store = TreeInstanceStore::default();
    store.insert(tree(IVec3::new(1, 2, 3), 10)).unwrap();

    let result = store.replace_chunk(IVec3::ZERO, vec![tree(IVec3::new(16, 2, 3), 20)]);
    assert!(result.is_err());
    assert_eq!(store.snapshot_chunk(IVec3::ZERO).len(), 1);

    let taken = store.take_chunk(IVec3::ZERO);
    assert_eq!(taken.len(), 1);
    assert!(store.snapshot_chunk(IVec3::ZERO).is_empty());
}
