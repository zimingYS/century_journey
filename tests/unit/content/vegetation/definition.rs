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
            "sapling_duration_game_minutes": 120,
            "young_duration_game_minutes": 360,
            "retry_interval_game_minutes": 7
        },
        "young_blueprint": {
            "trunk_height": { "min": 2, "max": 3 },
            "crown_radius": { "min": 1, "max": 1 }
        },
        "blueprint": {
            "trunk_height": { "min": 4, "max": 7 },
            "crown_radius": { "min": 2, "max": 3 }
        }
    });

    let definition: TreeSpeciesDefinition = serde_json::from_value(value).unwrap();

    assert_eq!(definition.identifier.to_string(), "test:oak");
    assert_eq!(definition.growth.sapling_duration_game_minutes, 120);
    assert_eq!(definition.growth.young_duration_game_minutes, 360);
    assert_eq!(definition.growth.retry_interval_game_minutes, 7);
    assert_eq!(
        definition.young_blueprint.unwrap().trunk_height,
        TreeSizeRange { min: 2, max: 3 }
    );
    assert_eq!(definition.blueprint.trunk_height.min, 4);
    assert_eq!(definition.blueprint.trunk_height.max, 7);
}

#[test]
fn missing_stage_fields_use_defaults_and_old_attempt_interval_is_an_alias() {
    let value = serde_json::json!({
        "identifier": "test:oak",
        "display_name": "测试橡树",
        "sapling_block": "test:sapling",
        "trunk_block": "test:wood",
        "leaves_block": "test:leaves",
        "growth": {
            "attempt_interval_game_minutes": 9,
            "chance_per_attempt": 0.4
        },
        "blueprint": {
            "trunk_height": { "min": 4, "max": 7 },
            "crown_radius": { "min": 2, "max": 3 }
        }
    });

    let definition: TreeSpeciesDefinition = serde_json::from_value(value).unwrap();

    assert_eq!(definition.growth.sapling_duration_game_minutes, 24 * 60);
    assert_eq!(definition.growth.young_duration_game_minutes, 3 * 24 * 60);
    assert_eq!(definition.growth.retry_interval_game_minutes, 9);
    assert!(definition.young_blueprint.is_none());
}

#[test]
fn missing_growth_fields_use_the_complete_default_schedule() {
    let value = serde_json::json!({
        "identifier": "test:oak",
        "display_name": "测试橡树",
        "sapling_block": "test:sapling",
        "trunk_block": "test:wood",
        "leaves_block": "test:leaves",
        "growth": {},
        "blueprint": {
            "trunk_height": { "min": 4, "max": 7 },
            "crown_radius": { "min": 2, "max": 3 }
        }
    });

    let definition: TreeSpeciesDefinition = serde_json::from_value(value).unwrap();

    assert_eq!(definition.growth.sapling_duration_game_minutes, 24 * 60);
    assert_eq!(definition.growth.young_duration_game_minutes, 3 * 24 * 60);
    assert_eq!(definition.growth.retry_interval_game_minutes, 5);
}
