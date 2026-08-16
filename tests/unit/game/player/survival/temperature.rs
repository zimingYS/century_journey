use super::*;
use crate::game::player::survival::events::DamageEvent;
use std::time::Duration;

#[test]
fn speed_multiplier_is_one_in_comfort_zone() {
    for temp in [10.0, 20.0, 32.0] {
        assert_eq!(temperature_speed_multiplier(temp), 1.0, "舒适区温度 {temp}");
    }
}

#[test]
fn speed_multiplier_decreases_toward_extremes() {
    assert!(temperature_speed_multiplier(38.0) < 1.0, "过热应减速");
    assert!(temperature_speed_multiplier(0.0) < 1.0, "过冷应减速");
    assert_eq!(
        temperature_speed_multiplier(45.0),
        0.5,
        "致死过热应降到最低"
    );
    assert_eq!(
        temperature_speed_multiplier(-5.0),
        0.5,
        "致死失温应降到最低"
    );
    assert_eq!(
        temperature_speed_multiplier(100.0),
        0.5,
        "远超阈值应夹紧到 0.5"
    );
}

#[derive(Resource, Default)]
struct DamageCount(usize);

fn count_damage(mut reader: MessageReader<DamageEvent>, mut count: ResMut<DamageCount>) {
    count.0 += reader.read().count();
}

fn run_fixed_step(app: &mut App) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_millis(750));
    app.world_mut().run_schedule(FixedUpdate);
}

fn spawn_player(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((
            Player,
            Thirst {
                current: 20.0,
                max: 20.0,
            },
            Hunger::default(),
            TemperatureExposure::default(),
            PlayerLifecycle::default(),
        ))
        .id()
}

#[test]
fn extreme_heat_accelerates_thirst_and_deals_damage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(WeatherCell {
            cloud_water: 0.5,
            precipitation: 0.0,
            fog_density: 0.0,
            temperature_c: 50.0,
            humidity: 0.5,
        })
        .init_resource::<PlayerGameMode>()
        .init_resource::<DamageCount>()
        .add_message::<DamageEvent>()
        .add_systems(
            FixedUpdate,
            (temperature_survival_system, count_damage).chain(),
        );

    let player = spawn_player(&mut app);
    let initial_thirst = app.world().get::<Thirst>(player).unwrap().current;

    for _ in 0..6 {
        run_fixed_step(&mut app);
    }

    let thirst = app.world().get::<Thirst>(player).unwrap().current;
    assert!(
        thirst < initial_thirst,
        "过热应加速口渴，初始 {initial_thirst} 当前 {thirst}"
    );
    assert!(
        app.world().resource::<DamageCount>().0 > 0,
        "极限过热应造成周期伤害"
    );
}

#[test]
fn extreme_cold_accelerates_hunger_and_deals_damage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(WeatherCell {
            cloud_water: 0.5,
            precipitation: 0.0,
            fog_density: 0.0,
            temperature_c: -10.0,
            humidity: 0.5,
        })
        .init_resource::<PlayerGameMode>()
        .init_resource::<DamageCount>()
        .add_message::<DamageEvent>()
        .add_systems(
            FixedUpdate,
            (temperature_survival_system, count_damage).chain(),
        );

    let player = spawn_player(&mut app);
    let initial_saturation = app.world().get::<Hunger>(player).unwrap().saturation;

    for _ in 0..6 {
        run_fixed_step(&mut app);
    }

    let saturation = app.world().get::<Hunger>(player).unwrap().saturation;
    assert!(
        saturation < initial_saturation,
        "过冷应加速饥饿（先扣饱和度），初始 {initial_saturation} 当前 {saturation}"
    );
    assert!(
        app.world().resource::<DamageCount>().0 > 0,
        "极限失温应造成周期伤害"
    );
}
