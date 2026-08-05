use super::*;

#[test]
fn water_uniform_matches_the_wgsl_parameter_block_size() {
    assert_eq!(WaterMaterialUniform::min_size().get(), 16);
}
