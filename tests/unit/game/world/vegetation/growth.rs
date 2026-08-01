use super::*;
use crate::content::vegetation::definition::{TreeBlueprintDefinition, TreeSizeRange};

#[test]
fn content_blueprint_ranges_are_translated_without_changing_boundaries() {
    let definition = TreeBlueprintDefinition {
        trunk_height: TreeSizeRange { min: 2, max: 5 },
        crown_radius: TreeSizeRange { min: 1, max: 3 },
    };

    let parameters = blueprint_parameters(definition);

    assert_eq!(parameters.trunk_height_min, 2);
    assert_eq!(parameters.trunk_height_max, 5);
    assert_eq!(parameters.crown_radius_min, 1);
    assert_eq!(parameters.crown_radius_max, 3);
}

#[test]
fn tree_metadata_changes_mark_the_root_chunk_for_incremental_save() {
    let mut world_state = WorldState::default();
    let root = IVec3::new(-1, 31, 17);

    mark_tree_instance_modified(&mut world_state, root);

    assert!(
        world_state
            .chunk_modified_time(IVec3::new(-1, 1, 1))
            .is_some()
    );
}
