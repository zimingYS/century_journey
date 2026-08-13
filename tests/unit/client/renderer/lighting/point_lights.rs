use super::*;
use crate::content::block::definition::BlockLightDef;

fn source() -> BlockLightSource {
    BlockLightSource {
        world_pos: IVec3::new(1, 2, 3),
        light: BlockLightDef {
            emission: 15,
            color: [1.0, 0.5, 0.25],
            range: 9,
            casts_shadow: true,
        },
    }
}

#[test]
fn point_light_preserves_color_range_and_shadow_choice() {
    let light = point_light(source(), true, 1.0);
    assert_eq!(light.color, Color::linear_rgb(1.0, 0.5, 0.25));
    assert_eq!(light.intensity, MAX_BLOCK_LIGHT_LUMENS);
    assert_eq!(light.range, 9.5);
    assert!(light.shadow_maps_enabled);
}

#[test]
fn point_light_scales_low_emission_linearly() {
    let mut source = source();
    source.light.emission = 5;
    let light = point_light(source, false, 1.0);

    assert!((light.intensity - MAX_BLOCK_LIGHT_LUMENS / 3.0).abs() < f32::EPSILON);
}

#[test]
fn source_center_uses_voxel_center() {
    assert_eq!(source_center(source()), Vec3::new(1.5, 2.5, 3.5));
}

#[test]
fn point_light_lod_keeps_near_light_and_fades_smoothly() {
    assert_eq!(
        point_light_distance_fade(POINT_LIGHT_FADE_START_DISTANCE),
        1.0
    );
    assert_eq!(point_light_distance_fade(MAX_POINT_LIGHT_DISTANCE), 0.0);

    let midpoint = (POINT_LIGHT_FADE_START_DISTANCE + MAX_POINT_LIGHT_DISTANCE) * 0.5;
    assert!((point_light_distance_fade(midpoint) - 0.5).abs() < f32::EPSILON);
    assert_eq!(
        point_light(source(), false, 0.5).intensity,
        MAX_BLOCK_LIGHT_LUMENS * 0.5
    );
}
