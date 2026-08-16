use super::*;
use crate::game::world::generation::terrain::climate::{ClimateConfig, ClimateSampler};
use crate::shared::random::DeterministicRng;

#[test]
fn evaporation_raises_humidity_with_temperature() {
    let cool = compute_humidity(0.5, 5.0, 0.0);
    let hot = compute_humidity(0.5, 35.0, 0.0);
    assert!(hot > cool, "高温应通过蒸发提高湿度：{cool} vs {hot}");
}

#[test]
fn precipitation_drains_humidity() {
    let dry_sky = compute_humidity(0.5, 20.0, 0.0);
    let rainy = compute_humidity(0.5, 20.0, 0.9);
    assert!(
        rainy < dry_sky,
        "降水应消耗水汽降低湿度：{dry_sky} vs {rainy}"
    );
}

#[test]
fn cloud_shading_cools_temperature() {
    let clear = compute_temperature_c(0.6, 0.0, 0.0, 14);
    let overcast = compute_temperature_c(0.6, 1.0, 0.0, 14);
    assert!(
        overcast < clear,
        "云层遮蔽应降低温度：{clear} vs {overcast}"
    );
}

#[test]
fn rain_cools_temperature() {
    let clear = compute_temperature_c(0.6, 0.0, 0.0, 14);
    let rainy = compute_temperature_c(0.6, 0.0, 0.9, 14);
    assert!(rainy < clear, "降水应降低温度：{clear} vs {rainy}");
}

#[test]
fn step_weather_drives_cloud_toward_humidity() {
    // 湿度高 → 云量上升
    let mut humid = WeatherCell {
        cloud_water: 0.2,
        humidity: 0.8,
        ..Default::default()
    };
    step_weather(&mut humid, &mut DeterministicRng::new(1), Season::Spring);
    assert!(humid.cloud_water > 0.2, "湿度高应推动云量上升");

    // 湿度低 → 云量下降
    let mut dry = WeatherCell {
        cloud_water: 0.8,
        humidity: 0.2,
        ..Default::default()
    };
    step_weather(&mut dry, &mut DeterministicRng::new(1), Season::Spring);
    assert!(dry.cloud_water < 0.8, "湿度低应推动云量消散");
}

#[test]
fn step_weather_is_deterministic_given_same_state_and_seed() {
    let mut a = WeatherCell {
        humidity: 0.6,
        ..Default::default()
    };
    let mut b = a.clone();
    step_weather(&mut a, &mut DeterministicRng::new(42), Season::Summer);
    step_weather(&mut b, &mut DeterministicRng::new(42), Season::Summer);
    assert_eq!(a, b, "同状态与种子应产生相同转移");
}

#[test]
fn step_weather_keeps_dimensions_within_unit_range() {
    let mut weather = WeatherCell {
        cloud_water: 0.5,
        precipitation: 0.0,
        fog_density: 0.1,
        humidity: 0.5,
        ..Default::default()
    };
    let mut rng = DeterministicRng::new(7);
    for _ in 0..100 {
        step_weather(&mut weather, &mut rng, Season::Spring);
        assert!((0.0..=1.0).contains(&weather.cloud_water));
        assert!((0.0..=1.0).contains(&weather.precipitation));
        assert!((0.0..=1.0).contains(&weather.fog_density));
    }
}

#[test]
fn diurnal_temperature_peaks_in_afternoon() {
    let afternoon = diurnal_temperature_factor(14);
    let night = diurnal_temperature_factor(2);
    assert!(afternoon > 0.0);
    assert!(night < 0.0);
    assert!(afternoon > night);
}

#[test]
fn temperature_derives_within_sane_bounds() {
    let climate = ClimateSampler::new(123, ClimateConfig::default());
    for hour in [0u8, 2, 6, 12, 14, 18, 22] {
        let base = climate.sample_temperature_with_season(0, 0, Season::Summer);
        let temp = compute_temperature_c(base, 0.5, 0.5, hour);
        assert!(
            (-40.0..=60.0).contains(&temp),
            "温度应在合理范围，得到 {temp}"
        );
    }
}
