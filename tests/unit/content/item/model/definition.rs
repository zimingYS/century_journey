use super::*;

#[test]
fn generated_model_fills_missing_display_transforms() {
    let mut model =
        ItemModelDefinition::generated(Identifier::new("century_journey", "sapling"), 0.03, false);
    model.display = ItemModelDisplay::default();

    model.fill_missing_display_transforms();

    assert!(model.display.gui.is_some());
    assert!(model.display.first_person_right_hand.is_some());
    assert!(model.display.third_person_right_hand.is_some());
    assert!(model.display.ground.is_some());
}
