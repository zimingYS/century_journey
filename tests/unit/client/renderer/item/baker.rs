use super::*;

#[test]
fn requested_fix_generated_item_material_receives_world_lighting() {
    assert!(!generated_item_material().unlit);
}
