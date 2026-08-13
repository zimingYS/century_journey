use super::*;

#[test]
fn set_get_roundtrip_preserves_channels() {
    let mut light = ChunkLight::default();
    let cell = LightCell {
        sky: LightRgb { r: 15, g: 12, b: 9 },
        block: LightRgb { r: 7, g: 3, b: 11 },
    };
    light.set(1, 2, 3, cell);
    assert_eq!(light.get(1, 2, 3), cell);
}

#[test]
fn channels_are_independent() {
    let mut light = ChunkLight::default();
    light.set(
        0,
        0,
        0,
        LightCell {
            sky: LightRgb::default(),
            block: LightRgb { r: 9, g: 0, b: 0 },
        },
    );
    let cell = light.get(0, 0, 0);
    assert_eq!(cell.sky, LightRgb::default());
    assert_eq!(cell.block.r, 9);
    assert_eq!(cell.block.g, 0);
    assert_eq!(cell.block.b, 0);
}

#[test]
fn values_are_quantized_to_4bit() {
    let mut light = ChunkLight::default();
    // 超过 4bit 的值写入后按上限钳制，避免回绕成黑色。
    light.set(
        5,
        5,
        5,
        LightCell {
            sky: LightRgb { r: 16, g: 0, b: 0 },
            block: LightRgb { r: 255, g: 0, b: 0 },
        },
    );
    let cell = light.get(5, 5, 5);
    assert_eq!(cell.sky.r, 15);
    assert_eq!(cell.block.r, 15);
}

#[test]
fn default_is_all_zero() {
    let light = ChunkLight::default();
    assert_eq!(light.get(8, 8, 8), LightCell::default());
    assert!(!light.is_initialized());
}

#[test]
fn initialized_dark_chunk_is_distinct_from_pending_chunk() {
    let mut light = ChunkLight::default();
    light.mark_initialized();
    assert!(light.is_initialized());
    assert_eq!(light.get(8, 8, 8), LightCell::default());
}

#[test]
fn fingerprint_is_stable_and_invalidated_by_writes() {
    let mut first = ChunkLight::default();
    let mut second = ChunkLight::default();
    first.mark_initialized();
    second.mark_initialized();
    assert_eq!(first.fingerprint(), second.fingerprint());

    first.set(
        1,
        2,
        3,
        LightCell {
            sky: LightRgb { r: 15, g: 8, b: 2 },
            block: LightRgb::default(),
        },
    );
    assert!(!first.is_initialized());
    assert_eq!(first.fingerprint(), 0);

    first.mark_initialized();
    assert_ne!(first.fingerprint(), second.fingerprint());
}

#[test]
fn reset_block_preserves_sky_and_clears_block_channels() {
    let mut light = ChunkLight::default();
    light.set(
        1,
        2,
        3,
        LightCell {
            sky: LightRgb { r: 15, g: 9, b: 4 },
            block: LightRgb { r: 12, g: 7, b: 3 },
        },
    );
    light.mark_initialized();

    light.reset_block();

    assert_eq!(
        light.get(1, 2, 3),
        LightCell {
            sky: LightRgb { r: 15, g: 9, b: 4 },
            block: LightRgb::default(),
        }
    );
    assert!(!light.is_initialized());
}
