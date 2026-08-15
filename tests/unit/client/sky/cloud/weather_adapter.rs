use super::*;
use crate::game::world::weather::WeatherCell;

#[test]
fn syncs_cloud_water_to_coverage_and_fog_to_visibility() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(WeatherCell {
            cloud_water: 0.7,
            fog_density: 0.5,
            ..Default::default()
        })
        .init_resource::<CloudWeatherState>()
        .add_systems(Update, sync_weather_to_cloud_system);

    app.update();

    let state = app.world().resource::<CloudWeatherState>();
    assert_eq!(state.coverage, 0.7, "云量应映射到云场不透明度");
    assert!(
        (state.visibility - (1.0 - 0.5 * 0.6)).abs() < 1e-6,
        "雾霾应降低能见度，得到 {}",
        state.visibility
    );
}

#[test]
fn full_fog_clamps_visibility_above_zero() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(WeatherCell {
            cloud_water: 0.2,
            fog_density: 1.0,
            ..Default::default()
        })
        .init_resource::<CloudWeatherState>()
        .add_systems(Update, sync_weather_to_cloud_system);

    app.update();

    let state = app.world().resource::<CloudWeatherState>();
    assert!(
        (state.visibility - 0.4).abs() < 1e-6,
        "满雾能见度应夹紧到 0.4，得到 {}",
        state.visibility
    );
}
