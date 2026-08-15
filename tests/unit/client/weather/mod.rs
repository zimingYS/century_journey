use super::*;

#[test]
fn noise01_stays_within_unit_range() {
    for i in 0..200 {
        let value = noise01(i, i * 7 + 3);
        assert!((0.0..=1.0).contains(&value));
    }
}

fn setup(weather: WeatherCell) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .insert_resource(weather)
        .init_resource::<PrecipitationVisuals>()
        .add_systems(Update, spawn_precipitation_system);
    app.world_mut()
        .spawn((Player, Transform::from_xyz(0.0, 70.0, 0.0)));
    app
}

#[test]
fn spawns_rain_when_warm_and_precipitating() {
    let mut app = setup(WeatherCell {
        precipitation: 0.5,
        temperature_c: 15.0,
        ..Default::default()
    });

    app.update();

    let mut query = app.world_mut().query::<&PrecipitationParticle>();
    let count = query.iter(app.world()).count();
    assert!(count > 0, "降水时应生成雨滴粒子");
    let all_rain = query
        .iter(app.world())
        .all(|p| p.kind == PrecipitationKind::Rain);
    assert!(all_rain, "温度高于冰点应全部为雨滴");
}

#[test]
fn spawns_snow_when_freezing() {
    let mut app = setup(WeatherCell {
        precipitation: 0.5,
        temperature_c: -5.0,
        ..Default::default()
    });

    app.update();

    let mut query = app.world_mut().query::<&PrecipitationParticle>();
    let all_snow = query
        .iter(app.world())
        .all(|p| p.kind == PrecipitationKind::Snow);
    assert!(all_snow, "温度低于冰点应全部为雪花");
}

#[test]
fn spawns_nothing_when_not_precipitating() {
    let mut app = setup(WeatherCell {
        precipitation: 0.0,
        ..Default::default()
    });

    app.update();

    let mut query = app.world_mut().query::<&PrecipitationParticle>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 0, "无降水不应生成粒子");
}
