use super::*;
use crate::game::world::lighting::chunk_light::{ChunkLight, LightCell, LightRgb};
use std::sync::Arc;

#[test]
fn deep_night_uses_readable_exposure() {
    let night_ev100 = visibility_exposure_ev100(0.0, 1.0, 1.0);
    let noon_ev100 = visibility_exposure_ev100(1.0, -1.0, 0.0);

    assert_eq!(night_ev100, NIGHT_EXPOSURE_EV100);
    assert!(night_ev100 < noon_ev100);
}

#[test]
fn celestial_light_is_suppressed_by_underground_sky_visibility() {
    assert_eq!(celestial_visibility_at(None, Vec3::ZERO), 1.0);

    let mut lighting = WorldLighting::default();
    let mut light = ChunkLight::default();
    light.set(
        2,
        3,
        4,
        LightCell {
            sky: LightRgb { r: 6, g: 3, b: 0 },
            block: LightRgb::default(),
        },
    );
    light.mark_initialized();
    lighting.chunk_lights.insert(IVec3::ZERO, Arc::new(light));

    let visibility = celestial_visibility_at(Some(&lighting), Vec3::new(2.5, 3.5, 4.5));
    assert!((visibility - 0.4).abs() < f32::EPSILON);
}
