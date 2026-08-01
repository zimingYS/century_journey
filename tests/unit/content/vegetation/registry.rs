use super::*;
use crate::content::vegetation::definition::{
    TreeBlueprintDefinition, TreeGrowthDefinition, TreeSizeRange,
};

fn definition(identifier: &str, sapling: &str) -> TreeSpeciesDefinition {
    TreeSpeciesDefinition {
        identifier: Identifier::parse(identifier).unwrap(),
        display_name: identifier.into(),
        sapling_block: Identifier::parse(sapling).unwrap(),
        trunk_block: Identifier::parse("test:wood").unwrap(),
        leaves_block: Identifier::parse("test:leaves").unwrap(),
        growth: TreeGrowthDefinition {
            sapling_duration_game_minutes: 24 * 60,
            young_duration_game_minutes: 3 * 24 * 60,
            retry_interval_game_minutes: 5,
        },
        young_blueprint: None,
        blueprint: TreeBlueprintDefinition {
            trunk_height: TreeSizeRange { min: 4, max: 6 },
            crown_radius: TreeSizeRange { min: 2, max: 3 },
        },
    }
}

fn resolve_test_block(identifier: &Identifier) -> Option<u16> {
    match identifier.to_string().as_str() {
        "test:sapling" => Some(1),
        "test:other_sapling" => Some(2),
        "test:wood" => Some(3),
        "test:leaves" => Some(4),
        _ => None,
    }
}

#[test]
fn registry_indexes_species_by_identifier_and_sapling_block() {
    let registry = build_registry(
        vec![
            definition("test:second", "test:other_sapling"),
            definition("test:first", "test:sapling"),
        ],
        resolve_test_block,
    )
    .unwrap();

    let first = Identifier::parse("test:first").unwrap();
    assert_eq!(registry.get(&first).unwrap().sapling_block_id, 1);
    assert_eq!(
        registry.get_by_sapling_id(2).unwrap().definition.identifier,
        Identifier::parse("test:second").unwrap()
    );
    assert_eq!(registry.iter().next().unwrap().definition.identifier, first);
}

#[test]
fn duplicate_sapling_mapping_is_rejected() {
    let result = build_registry(
        vec![
            definition("test:first", "test:sapling"),
            definition("test:second", "test:sapling"),
        ],
        resolve_test_block,
    );

    assert!(result.unwrap_err().contains("multiple tree species"));
}

#[test]
fn failed_replacement_keeps_the_previous_registry() {
    let mut registry = build_registry(
        vec![definition("test:first", "test:sapling")],
        resolve_test_block,
    )
    .unwrap();
    let first = Identifier::parse("test:first").unwrap();

    let result = registry.replace_definitions(
        vec![definition("test:broken", "test:missing")],
        &BlockRegistry::default(),
    );

    assert!(result.is_err());
    assert_eq!(registry.len(), 1);
    assert!(registry.get(&first).is_some());
}
