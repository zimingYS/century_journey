use super::*;

#[test]
fn omitted_light_transmission_has_the_same_release_and_debug_default() {
    let value = serde_json::json!({
        "identifier": "test:stone",
        "display_name": "Stone",
        "render_mode": "Opaque",
        "textures": { "top": "blocks/stone" },
        "hardness": 1.0
    });

    let block: BlockProperty = serde_json::from_value(value).unwrap();

    assert_eq!(block.light_transmission, 0.0);
}
