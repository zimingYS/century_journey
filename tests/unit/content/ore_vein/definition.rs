use super::*;

#[test]
fn ore_vein_definition_deserializes_all_generation_fields() {
    let value = serde_json::json!({
        "identifier": "test:coal",
        "display_name": "测试煤矿脉",
        "block": "test:coal_ore",
        "priority": 1,
        "min_y": 0,
        "max_y": 64,
        "threshold": -0.35,
        "scale": 0.08
    });

    let definition: OreVeinDefinition = serde_json::from_value(value).unwrap();

    assert_eq!(definition.identifier.to_string(), "test:coal");
    assert_eq!(definition.display_name, "测试煤矿脉");
    assert_eq!(definition.block.to_string(), "test:coal_ore");
    assert_eq!(definition.priority, 1);
    assert_eq!(definition.min_y, 0);
    assert_eq!(definition.max_y, 64);
    assert_eq!(definition.threshold, -0.35);
    assert_eq!(definition.scale, 0.08);
}

#[test]
fn missing_priority_defaults_to_zero() {
    let value = serde_json::json!({
        "identifier": "test:coal",
        "display_name": "测试煤矿脉",
        "block": "test:coal_ore",
        "min_y": 0,
        "max_y": 64,
        "threshold": -0.35,
        "scale": 0.08
    });

    let definition: OreVeinDefinition = serde_json::from_value(value).unwrap();

    assert_eq!(definition.priority, 0);
}

#[test]
fn negative_threshold_and_scale_are_preserved() {
    let value = serde_json::json!({
        "identifier": "test:gold",
        "display_name": "测试金矿脉",
        "block": "test:gold_ore",
        "priority": 3,
        "min_y": -40,
        "max_y": 20,
        "threshold": -0.65,
        "scale": 0.12
    });

    let definition: OreVeinDefinition = serde_json::from_value(value).unwrap();

    assert_eq!(definition.min_y, -40);
    assert_eq!(definition.max_y, 20);
    assert_eq!(definition.threshold, -0.65);
    assert_eq!(definition.scale, 0.12);
}
