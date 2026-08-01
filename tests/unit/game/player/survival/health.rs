//! 验证伤害结算的数值边界以及死亡事件的幂等性。

use super::*;
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::player::survival::events::DamageSource;
use bevy::prelude::{App, IntoScheduleConfigs, ResMut, Resource, Update};

#[derive(Resource, Default)]
struct DeathEventCount(usize);

fn count_death_events(mut reader: MessageReader<DeathEvent>, mut count: ResMut<DeathEventCount>) {
    count.0 += reader.read().count();
}

#[test]
fn exact_lethal_damage_emits_one_death_and_invalid_damage_is_ignored() {
    let mut app = App::new();
    app.init_resource::<DeathEventCount>()
        .add_message::<DamageEvent>()
        .add_message::<DeathEvent>()
        .insert_resource(PlayerGameMode {
            mode: GameMode::Survival,
        })
        .add_systems(Update, (damage_system, count_death_events).chain());
    let player = app
        .world_mut()
        .spawn((
            Player,
            Health {
                current: 2.0,
                max: 20.0,
            },
            PlayerLifecycle::default(),
        ))
        .id();

    app.world_mut().write_message(DamageEvent {
        target: player,
        amount: f32::NAN,
        source: DamageSource::Generic,
    });
    app.world_mut().write_message(DamageEvent {
        target: player,
        amount: 2.0,
        source: DamageSource::Generic,
    });
    app.world_mut().write_message(DamageEvent {
        target: player,
        amount: 2.0,
        source: DamageSource::Generic,
    });
    app.update();

    assert_eq!(app.world().get::<Health>(player).unwrap().current, 0.0);
    assert_eq!(
        app.world().get::<PlayerLifecycle>(player).unwrap().state,
        PlayerLifeState::Dead
    );
    assert_eq!(app.world().resource::<DeathEventCount>().0, 1);
}

#[test]
fn health_ignores_invalid_damage_and_clamps_lethal_damage() {
    let mut health = Health::default();

    health.apply_damage(f32::NAN);
    health.apply_damage(-3.0);
    assert_eq!(health.current, health.max);

    health.apply_damage(health.max);
    assert_eq!(health.current, 0.0);
}
