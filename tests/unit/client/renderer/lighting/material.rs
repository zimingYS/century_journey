use super::*;

#[test]
fn voxel_uniform_matches_the_wgsl_parameter_block_size() {
    assert_eq!(std::mem::size_of::<VoxelMaterialUniform>(), 16);
}
