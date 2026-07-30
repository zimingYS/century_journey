use super::*;

#[test]
fn tree_species_definition_deserializes_growth_and_blueprint_ranges() {
    let value = serde_json::json!({
        "identifier": "test:oak",
        "display_name": "测试橡树",
        "sapling_block": "test:sapling",
        "trunk_block": "test:wood",
        "leaves_block": "test:leaves",
        "growth": {
            "attempt_interval_game_minutes": 5,
            "chance_per_attempt": 0.4
        },
        "blueprint": {
            "trunk_height": { "min": 4, "max": 7 },
            "crown_radius": { "min": 2, "max": 3 }
        }
    });

    let definition: TreeSpeciesDefinition = serde_json::from_value(value).unwrap();

    assert_eq!(definition.identifier.to_string(), "test:oak");
    assert_eq!(definition.growth.attempt_interval_game_minutes, 5);
    assert_eq!(definition.blueprint.trunk_height.min, 4);
    assert_eq!(definition.blueprint.trunk_height.max, 7);
}
