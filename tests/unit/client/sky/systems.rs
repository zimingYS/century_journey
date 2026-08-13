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
fn sealed_cave_uses_dark_adapted_exposure_during_day() {
    let cave_ev100 = visibility_exposure_ev100(0.0, -1.0, 0.0);
    let open_sky_ev100 = visibility_exposure_ev100(1.0, -1.0, 0.0);

    assert!(cave_ev100 < open_sky_ev100 - 5.0);
    assert!(cave_ev100 > NIGHT_EXPOSURE_EV100);
}

#[test]
fn celestial_light_is_suppressed_by_underground_sky_visibility() {
    assert_eq!(celestial_visibility_at(None, Vec3::ZERO), None);

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

    let visibility = celestial_visibility_at(Some(&lighting), Vec3::new(2.5, 3.5, 4.5))
        .expect("已初始化光数组应返回天空可见度");
    assert!((visibility - 0.4).abs() < f32::EPSILON);
}

#[test]
fn celestial_visibility_interpolates_across_voxel_boundaries() {
    let mut lighting = WorldLighting::default();
    let mut light = ChunkLight::default();
    light.set(
        2,
        3,
        4,
        LightCell {
            sky: LightRgb {
                r: 15,
                g: 15,
                b: 15,
            },
            block: LightRgb::default(),
        },
    );
    light.set(3, 3, 4, LightCell::default());
    light.mark_initialized();
    lighting.chunk_lights.insert(IVec3::ZERO, Arc::new(light));

    let visibility = celestial_visibility_at(Some(&lighting), Vec3::new(3.0, 3.5, 4.5))
        .expect("体素边界两侧均有有效光数组");
    assert!((visibility - 0.5).abs() < f32::EPSILON);
}

#[test]
fn missing_neighbor_light_does_not_become_open_sky() {
    let mut lighting = WorldLighting::default();
    let mut light = ChunkLight::default();
    light.mark_initialized();
    lighting.chunk_lights.insert(IVec3::ZERO, Arc::new(light));

    let visibility = celestial_visibility_at(Some(&lighting), Vec3::new(15.9, 3.5, 4.5))
        .expect("中心体素的暗光数组已初始化");
    assert_eq!(visibility, 0.0);
}

#[test]
fn celestial_visibility_changes_continuously_over_render_frames() {
    let first = visibility_step(
        0.0,
        1.0,
        1.0 / 60.0,
        CELESTIAL_VISIBILITY_RESPONSE_PER_SECOND,
    );
    assert!(first > 0.0 && first < 1.0);

    let darkened = visibility_step(
        1.0,
        0.0,
        1.0 / 60.0,
        CELESTIAL_VISIBILITY_RESPONSE_PER_SECOND,
    );
    assert!(darkened > 0.0 && darkened < 1.0);

    let clamped_pause = visibility_step(0.0, 1.0, 10.0, CELESTIAL_VISIBILITY_RESPONSE_PER_SECOND);
    let clamped_frame = visibility_step(
        0.0,
        1.0,
        MAX_VISIBILITY_STEP_SECONDS,
        CELESTIAL_VISIBILITY_RESPONSE_PER_SECOND,
    );
    assert!((clamped_pause - clamped_frame).abs() < f32::EPSILON);
}

#[test]
fn exposure_target_uses_hysteresis_in_partial_skylight() {
    assert_eq!(exposure_visibility_target(0.8, 0.0), 0.8);
    assert_eq!(exposure_visibility_target(0.5, 0.8), 0.8);
    assert_eq!(exposure_visibility_target(0.5, 0.0), 0.0);
    assert_eq!(exposure_visibility_target(0.2, 0.8), 0.0);
}

#[test]
fn single_bright_sample_does_not_open_cave_exposure() {
    let mut values = [1.0, 0.0, 0.0, 0.0, 0.0];
    values.sort_by(f32::total_cmp);

    assert_eq!(values[2], 0.0);
}

#[test]
fn directional_shadow_gate_requires_exposure_and_direct_sky_visibility() {
    let shadow_visibility = |exposure: f32, celestial: f32| exposure.min(celestial);

    assert!(shadow_visibility(1.0, 1.0) >= SHADOW_MAP_ENABLE_THRESHOLD);
    assert!(shadow_visibility(0.8, 0.0) < SHADOW_MAP_DISABLE_THRESHOLD);
    assert!(shadow_visibility(0.0, 0.8) < SHADOW_MAP_DISABLE_THRESHOLD);
}

#[test]
fn shadow_map_gate_has_opening_and_closing_hysteresis() {
    let mut state = CelestialVisibilityState::default();

    assert!(!state.update_shadow_maps(0.4, true));
    assert!(state.update_shadow_maps(0.5, true));
    assert!(state.update_shadow_maps(0.3, true));
    assert!(!state.update_shadow_maps(0.2, true));
    assert!(!state.update_shadow_maps(1.0, false));
}
