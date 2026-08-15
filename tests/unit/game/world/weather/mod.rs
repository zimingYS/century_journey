use super::*;
use crate::game::world::generation::terrain::climate::{ClimateConfig, ClimateSampler};
use crate::shared::random::DeterministicRng;

#[test]
fn summer_is_rainier_than_winter() {
    for humidity in [0.1, 0.3, 0.5, 0.7, 0.9] {
        let summer = rain_probability(Season::Summer, humidity);
        let winter = rain_probability(Season::Winter, humidity);
        assert!(summer > winter, "夏季应比冬季多雨，湿度 {humidity}");
    }
}

#[test]
fn humid_climate_is_rainier_than_dry() {
    for season in [
        Season::Spring,
        Season::Summer,
        Season::Autumn,
        Season::Winter,
    ] {
        let dry = rain_probability(season, 0.1);
        let humid = rain_probability(season, 0.9);
        assert!(humid > dry, "潮湿气候应比干燥多雨，季节 {season:?}");
    }
}

#[test]
fn rain_probability_stays_within_bounds() {
    for season in [
        Season::Spring,
        Season::Summer,
        Season::Autumn,
        Season::Winter,
    ] {
        for humidity in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let p = rain_probability(season, humidity);
            assert!((0.0..=1.0).contains(&p));
        }
    }
}

#[test]
fn step_weather_is_deterministic_given_same_state_and_seed() {
    let mut a = WeatherCell::default();
    let mut b = WeatherCell::default();
    let mut rng_a = DeterministicRng::new(42);
    let mut rng_b = DeterministicRng::new(42);

    step_weather(&mut a, &mut rng_a, Season::Summer, 0.8);
    step_weather(&mut b, &mut rng_b, Season::Summer, 0.8);

    assert_eq!(a, b, "同状态与种子应产生相同转移");
}

#[test]
fn step_weather_keeps_dimensions_within_unit_range() {
    let mut weather = WeatherCell {
        cloud_water: 0.5,
        precipitation: 0.0,
        fog_density: 0.1,
        temperature_c: 20.0,
    };
    let mut rng = DeterministicRng::new(7);
    for _ in 0..100 {
        step_weather(&mut weather, &mut rng, Season::Spring, 0.5);
        assert!((0.0..=1.0).contains(&weather.cloud_water));
        assert!((0.0..=1.0).contains(&weather.precipitation));
        assert!((0.0..=1.0).contains(&weather.fog_density));
    }
}

#[test]
fn rain_cools_down_the_temperature() {
    let base = 0.6; // 温暖气候带
    let clear = compute_temperature_c(base, 0.0, 14);
    let rainy = compute_temperature_c(base, 0.9, 14);
    assert!(rainy < clear, "降雨应降低温度，晴 {clear} vs 雨 {rainy}");
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
    let weather = WeatherCell {
        cloud_water: 0.6,
        precipitation: 0.8,
        fog_density: 0.2,
        temperature_c: 20.0,
    };
    for hour in [0u8, 2, 6, 12, 14, 18, 22] {
        let base = climate.sample_temperature_with_season(0, 0, Season::Summer);
        let temp = compute_temperature_c(base, weather.precipitation, hour);
        assert!(
            (-40.0..=60.0).contains(&temp),
            "温度应在合理范围，得到 {temp}"
        );
    }
}
