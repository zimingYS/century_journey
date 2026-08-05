use crate::content::cloud::definition::{
    CloudDefinition, CloudLayerDefinition, CloudPatchDefinition,
};
use crate::content::cloud::registry::CloudRegistry;
use crate::content::format::Versioned;
use crate::shared::identifier::Identifier;

fn sample_definition() -> CloudDefinition {
    CloudDefinition {
        identifier: Identifier::parse("century_journey:test").expect("identifier"),
        density: 0.5,
        seed: 42,
        layers: vec![CloudLayerDefinition {
            height: 128.0,
            size: 512.0,
            speed: 1.0,
            wind_direction: [1.0, 0.0],
            tint_day: [1.0, 1.0, 1.0],
            tint_night: [0.2, 0.25, 0.35],
            tint_sunset: [1.0, 0.7, 0.5],
            opacity: 0.6,
        }],
        patches: CloudPatchDefinition::default(),
    }
}

#[test]
fn cloud_definition_deserializes_from_json() {
    let json = r#"{
        "format_version": 1,
        "identifier": "century_journey:overworld",
        "density": 0.48,
        "seed": 20260803,
        "layers": [{
            "height": 128.0,
            "size": 512.0,
            "speed": 0.9,
            "wind_direction": [1.0, 0.0],
            "tint_day": [1.0, 1.0, 1.0],
            "tint_night": [0.22, 0.25, 0.36],
            "tint_sunset": [1.0, 0.72, 0.52],
            "opacity": 0.62
        }],
        "patches": {
            "enabled": true,
            "count": 24,
            "spawn_radius": 70.0,
            "scale_min": 5.0,
            "scale_max": 11.0,
            "opacity": 0.32
        }
    }"#;
    let versioned: Versioned<CloudDefinition> =
        serde_json::from_str(json).expect("cloud definition parses");
    let cloud = versioned
        .into_current("definitions/clouds/century_journey/overworld.json")
        .expect("format version matches");

    assert_eq!(
        cloud.identifier,
        Identifier::parse("century_journey:overworld").unwrap()
    );
    assert_eq!(cloud.density, 0.48);
    assert_eq!(cloud.seed, 20260803);
    assert_eq!(cloud.layers.len(), 1);
    assert_eq!(cloud.layers[0].height, 128.0);
    assert_eq!(cloud.layers[0].tint_night, [0.22, 0.25, 0.36]);
    assert_eq!(cloud.layers[0].wind_direction, [1.0, 0.0]);
    assert!(cloud.patches.enabled);
    assert_eq!(cloud.patches.count, 24);
}

#[test]
fn patches_default_to_disabled_with_sane_bounds() {
    let cloud = sample_definition();
    let patches = CloudPatchDefinition::default();
    assert!(!patches.enabled);
    assert_eq!(patches.count, 10);
    assert!(patches.count <= 200);
    assert!(patches.spawn_radius > 0.0);
    assert!(patches.scale_min <= patches.scale_max);
    assert!((0.0..=1.0).contains(&patches.opacity));
    let _ = cloud;
}

#[test]
fn registry_primary_returns_first_definition() {
    let mut registry = CloudRegistry::default();
    assert!(registry.is_empty());
    assert!(registry.primary().is_none());

    registry.replace_definitions(vec![sample_definition()]);
    assert_eq!(registry.len(), 1);
    let primary = registry.primary().expect("primary exists");
    assert_eq!(
        primary.identifier,
        Identifier::parse("century_journey:test").unwrap()
    );
    assert_eq!(primary.density, 0.5);
    assert_eq!(primary.layers.len(), 1);
}
