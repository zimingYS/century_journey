use super::super::tint::{
    apply_vertex_tint, biome_tint, final_tint, quantize_tint, season_tint, unquantize_tint,
};
use crate::content::block::definition::BlockTint;
use crate::game::world::time::Season;

#[test]
fn biome_tint_stays_in_unit_range_across_climate_extremes() {
    for temp in [0.0, 0.25, 0.5, 0.75, 1.0] {
        for hum in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let color = biome_tint(BlockTint::GrassTop, temp, hum);
            for c in color {
                assert!((0.0..=1.0).contains(&c), "{c} out of [0,1] at temp={temp},hum={hum}");
            }
        }
    }
}

#[test]
fn biome_tint_dry_warm_shifts_grass_toward_yellow() {
    let warm_dry = biome_tint(BlockTint::GrassTop, 0.9, 0.1);
    let cool_wet = biome_tint(BlockTint::GrassTop, 0.1, 0.9);
    assert!(
        warm_dry[0] > warm_dry[2],
        "warm_dry 应红 > 蓝：{warm_dry:?}"
    );
    assert!(
        cool_wet[2] > cool_wet[0],
        "cool_wet 应蓝 > 红：{cool_wet:?}"
    );
    assert!(
        warm_dry[0] > cool_wet[0],
        "warm_dry 应比 cool_wet 更红：{} vs {}",
        warm_dry[0],
        cool_wet[0]
    );
}

#[test]
fn season_tint_stays_within_unit_range() {
    for season in [Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
        for c in season_tint(season) {
            assert!((0.0..=1.0).contains(&c), "season {season:?} tint out of range: {c}");
        }
    }
}

#[test]
fn autumn_shifts_warm_and_dampens_green() {
    let autumn = season_tint(Season::Autumn);
    assert!(autumn[0] >= 0.9 && autumn[1] <= 0.7, "秋季应红升绿降：{autumn:?}");
}

#[test]
fn winter_desaturates() {
    let winter = season_tint(Season::Winter);
    assert!(winter[0] <= 0.8 && winter[2] >= 0.85, "冬季应整体偏白/淡：{winter:?}");
}

#[test]
fn final_tint_multiplies_biome_and_season() {
    let combined = final_tint(BlockTint::GrassTop, 0.5, 0.5, Season::Summer);
    let expected_r = biome_tint(BlockTint::GrassTop, 0.5, 0.5)[0] * season_tint(Season::Summer)[0];
    assert!(
        (combined[0] - expected_r).abs() < 1e-5,
        "final_tint[0] 应等于 biome×season：got {} expected {}",
        combined[0],
        expected_r
    );
}

#[test]
fn quantize_tint_round_trips_with_small_error() {
    let original = [0.33, 0.66, 1.0];
    let q = quantize_tint(original);
    let back = unquantize_tint(q);
    for i in 0..3 {
        assert!(
            (original[i] - back[i]).abs() <= 1.0 / 15.0 + 1e-4,
            "量化回环误差过大：{original:?} -> {q:?} -> {back:?}"
        );
    }
}

#[test]
fn white_quant_tint_is_full_bright() {
    assert_eq!(
        super::super::tint::white_tint(),
        super::super::tint::LightRgb { r: 15, g: 15, b: 15 }
    );
}

#[test]
fn apply_vertex_tint_multiplies_per_channel() {
    let base = [0.5, 0.4, 0.3, 1.0];
    let tinted = apply_vertex_tint(base, [0.5, 1.0, 0.5]);
    assert_eq!(tinted[0], 0.25);
    assert_eq!(tinted[1], 0.4);
    assert_eq!(tinted[2], 0.15);
    assert_eq!(tinted[3], 1.0);
}

#[test]
fn apply_vertex_tint_clamps_to_floor() {
    let very_dark = apply_vertex_tint([0.05, 0.05, 0.05, 1.0], [0.05, 0.05, 0.05]);
    for c in &very_dark[..3] {
        assert!(*c >= 0.02, "低于下限：{very_dark:?}");
    }
}
