use super::*;
use crate::content::ore_vein::definition::OreVeinDefinition;

fn definition(identifier: &str, block: &str, priority: u32) -> OreVeinDefinition {
    OreVeinDefinition {
        identifier: Identifier::parse(identifier).unwrap(),
        display_name: identifier.into(),
        block: Identifier::parse(block).unwrap(),
        priority,
        min_y: -40,
        max_y: 40,
        threshold: -0.5,
        scale: 0.1,
    }
}

fn resolve_test_block(identifier: &Identifier) -> Option<u16> {
    match identifier.to_string().as_str() {
        "test:coal_ore" => Some(1),
        "test:iron_ore" => Some(2),
        "test:gold_ore" => Some(3),
        "test:air" => Some(0),
        _ => None,
    }
}

#[test]
fn registry_sorts_veins_by_priority_descending() {
    let registry = build_registry(
        vec![
            definition("test:coal", "test:coal_ore", 1),
            definition("test:gold", "test:gold_ore", 3),
            definition("test:iron", "test:iron_ore", 2),
        ],
        resolve_test_block,
    )
    .unwrap();

    let identifiers = registry
        .iter()
        .map(|vein| vein.definition.identifier.to_string())
        .collect::<Vec<_>>();
    assert_eq!(identifiers, ["test:gold", "test:iron", "test:coal"]);
    let ids = registry
        .iter()
        .map(|vein| vein.block_id)
        .collect::<Vec<_>>();
    assert_eq!(ids, [3, 2, 1]);
}

#[test]
fn equal_priority_falls_back_to_identifier_order() {
    let registry = build_registry(
        vec![
            definition("test:zeta", "test:coal_ore", 2),
            definition("test:alpha", "test:iron_ore", 2),
        ],
        resolve_test_block,
    )
    .unwrap();

    let identifiers = registry
        .iter()
        .map(|vein| vein.definition.identifier.to_string())
        .collect::<Vec<_>>();
    assert_eq!(identifiers, ["test:alpha", "test:zeta"]);
}

#[test]
fn duplicate_identifier_is_rejected() {
    let result = build_registry(
        vec![
            definition("test:coal", "test:coal_ore", 1),
            definition("test:coal", "test:iron_ore", 2),
        ],
        resolve_test_block,
    );

    assert!(
        result
            .unwrap_err()
            .contains("duplicate ore vein identifier")
    );
}

#[test]
fn unknown_block_reference_is_rejected() {
    let result = build_registry(
        vec![definition("test:broken", "test:missing", 1)],
        resolve_test_block,
    );

    assert!(result.unwrap_err().contains("unknown block"));
}

#[test]
fn air_block_is_rejected() {
    let result = build_registry(
        vec![definition("test:air_vein", "test:air", 1)],
        resolve_test_block,
    );

    assert!(result.unwrap_err().contains("cannot use the air block"));
}

#[test]
fn failed_replacement_keeps_the_previous_registry() {
    let mut registry = build_registry(
        vec![definition("test:coal", "test:coal_ore", 1)],
        resolve_test_block,
    )
    .unwrap();

    let result = registry.replace_definitions(
        vec![definition("test:broken", "test:missing", 2)],
        &BlockRegistry::default(),
    );

    assert!(result.is_err());
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.iter().next().unwrap().block_id, 1);
}
