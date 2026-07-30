use super::*;
use crate::game::world::vegetation::TreeInstance;
use crate::shared::identifier::Identifier;

#[test]
fn unload_and_restore_keep_voxels_and_tree_instances_together() {
    let mut world = WorldState::default();
    let mut chunk = ChunkData::new();
    chunk.voxels[0] = 7;
    world.insert_chunk(IVec3::ZERO, Arc::new(chunk));
    let tree = TreeInstance::new_mature(
        IVec3::new(1, 2, 3),
        Identifier::new("century_journey", "oak"),
        55,
        90,
    );
    world.insert_tree_instance(tree.clone()).unwrap();

    let snapshot = world.remove_chunk(IVec3::ZERO).unwrap();
    assert!(!world.contains_chunk(IVec3::ZERO));
    assert!(world.tree_instance(tree.root()).is_none());
    assert_eq!(snapshot.data.voxels[0], 7);
    assert_eq!(snapshot.tree_instances, vec![tree.clone()]);

    world
        .insert_restored_chunk(IVec3::ZERO, snapshot.data, snapshot.tree_instances)
        .unwrap();
    assert_eq!(world.tree_instance(tree.root()), Some(&tree));
}
