use super::*;

#[test]
fn uv_offset_advances_and_wraps_within_one_texture_unit() {
    assert!((advance_uv_offset(0.0, 1.0, 0.5) - 0.5).abs() < 1e-5);
    assert!((advance_uv_offset(0.8, 1.0, 0.5) - 0.3).abs() < 1e-5);
    assert!((advance_uv_offset(0.0, 2.0, 1.0)).abs() < 1e-5);
    // 零速度不推进。
    assert!((advance_uv_offset(0.3, 0.0, 10.0) - 0.3).abs() < 1e-5);
    // 始终落在 [0, 1) 区间。
    let value = advance_uv_offset(0.99, 3.0, 100.0);
    assert!((0.0..1.0).contains(&value));
}

#[test]
fn tint_color_is_day_tint_at_noon() {
    let color = cloud_tint_color(
        [1.0, 1.0, 1.0],
        [0.2, 0.25, 0.35],
        [1.0, 0.7, 0.5],
        0.0,
        0.0,
    );
    assert!((color[0] - 1.0).abs() < 1e-5);
    assert!((color[1] - 1.0).abs() < 1e-5);
    assert!((color[2] - 1.0).abs() < 1e-5);
}

#[test]
fn tint_color_is_night_tint_at_deep_night() {
    let color = cloud_tint_color(
        [1.0, 1.0, 1.0],
        [0.2, 0.25, 0.35],
        [1.0, 0.7, 0.5],
        1.0,
        0.0,
    );
    assert!((color[0] - 0.2).abs() < 1e-5);
    assert!((color[1] - 0.25).abs() < 1e-5);
    assert!((color[2] - 0.35).abs() < 1e-5);
}

#[test]
fn tint_color_blends_toward_sunset_at_twilight_peak() {
    let day = cloud_tint_color(
        [1.0, 1.0, 1.0],
        [0.2, 0.25, 0.35],
        [1.0, 0.7, 0.5],
        0.5,
        1.0,
    );
    // 黄昏峰值时红色通道保持高位（暖色），蓝色通道明显低于白天。
    assert!(day[0] > day[2]);
    assert!(day[2] < 1.0);
}

#[test]
fn tint_color_stays_within_unit_range() {
    for night in [0.0, 0.25, 0.5, 1.0] {
        for glow in [0.0, 0.5, 1.0] {
            let color = cloud_tint_color(
                [1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
                night,
                glow,
            );
            for value in color {
                assert!((0.0..=1.0).contains(&value), "tint {color:?} out of range");
            }
        }
    }
}

#[test]
fn weather_state_normalizes_external_inputs() {
    let state = CloudWeatherState {
        coverage: 2.0,
        wind_multiplier: -1.0,
        visibility: f32::NAN,
    }
    .normalized();

    assert_eq!(state.coverage, 1.0);
    assert_eq!(state.wind_multiplier, 0.0);
    assert_eq!(state.visibility, 1.0);
}

#[test]
fn world_uv_offset_wraps_negative_and_large_world_positions() {
    let offset = world_uv_offset(Vec2::new(-128.0, 1152.0), 512.0);
    assert!((offset.x - 0.75).abs() < 1e-5);
    assert!((offset.y - 0.25).abs() < 1e-5);
}
