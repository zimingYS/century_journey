use super::*;

#[test]
fn generated_item_material_receives_world_lighting() {
    assert!(!generated_item_material().unlit);
}
