use super::*;

#[test]
fn identical_inputs_generate_identical_unique_tree_voxels() {
    let parameters = TreeBlueprintParameters::generated_tree();
    let first = TreeBlueprint::generate(IVec3::new(3, 7, -2), 19, 4, 5, parameters);
    let second = TreeBlueprint::generate(IVec3::new(3, 7, -2), 19, 4, 5, parameters);
    let unique = first
        .voxels()
        .iter()
        .map(|voxel| voxel.world_pos)
        .collect::<HashSet<_>>();

    assert_eq!(first, second);
    assert_eq!(unique.len(), first.voxels().len());
}

#[test]
fn generated_tree_parameters_preserve_the_existing_shape_ranges() {
    let anchor = IVec3::new(0, 1, 0);
    let blueprint =
        TreeBlueprint::generate(anchor, 0, 8, 9, TreeBlueprintParameters::generated_tree());

    for dy in 0..4 {
        assert!(
            blueprint
                .voxels()
                .iter()
                .any(|voxel| voxel.world_pos == anchor + IVec3::Y * dy && voxel.block_id == 8)
        );
    }
    assert!(
        !blueprint
            .voxels()
            .iter()
            .any(|voxel| { voxel.world_pos == anchor + IVec3::Y * 4 && voxel.block_id == 8 })
    );
    assert!(blueprint.voxels().iter().all(|voxel| {
        voxel.block_id != 9 || (voxel.world_pos - (anchor + IVec3::Y * 4)).length_squared() <= 4
    }));
}
