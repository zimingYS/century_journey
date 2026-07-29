use super::*;

#[test]
fn generated_model_uses_its_texture_for_gui() {
    let texture = Identifier::new("century_journey", "sapling");
    let definition = ItemModelDefinition::generated(texture.clone(), 0.03, false);

    assert_eq!(generated_gui_texture(&definition), Some(&texture));
}
