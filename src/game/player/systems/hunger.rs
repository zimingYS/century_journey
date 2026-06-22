use crate::client::state::InputBlocked;
use crate::game::player::components::Player;
use crate::game::player::components::stats::{Health, Hunger};
use crate::game::player::events::{DamageEvent, DamageSource, HealEvent};
use bevy::prelude::*;

/// Action Cost 系统 — 冲刺/跳跃消耗饥饿
pub fn action_cost_system(
    time: Res<Time>,
    input_blocked: Res<InputBlocked>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Hunger, With<Player>>,
) {
    if input_blocked.0 {
        return;
    }
    let dt = time.delta_secs();

    let sprinting = keyboard.pressed(KeyCode::ShiftLeft)
        && keyboard.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]);
    let jumped = keyboard.just_pressed(KeyCode::Space);

    for mut hunger in &mut query {
        if sprinting {
            hunger.exhaust(0.1 * dt);
        }
        if jumped {
            hunger.exhaust(0.05);
        }
    }
}

/// 满饥饿自动回血 (每 4s, hunger >= 18)
pub fn natural_regeneration_system(
    mut timer: Local<f32>,
    time: Res<Time>,
    query: Query<(Entity, &Health, &Hunger), With<Player>>,
    mut heal_writer: MessageWriter<HealEvent>,
) {
    *timer -= time.delta_secs();
    if *timer > 0.0 {
        return;
    }
    *timer = 4.0;

    for (entity, health, hunger) in &query {
        if hunger.current >= 18.0 && health.current < health.max {
            heal_writer.write(HealEvent {
                target: entity,
                amount: 1.0,
            });
        }
    }
}

/// 饥饿伤害 (每 4s, hunger <= 0)
pub fn starvation_damage_system(
    mut timer: Local<f32>,
    time: Res<Time>,
    query: Query<(Entity, &Hunger), With<Player>>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    *timer -= time.delta_secs();
    if *timer > 0.0 {
        return;
    }
    *timer = 4.0;

    for (entity, hunger) in &query {
        if hunger.is_starving() {
            damage_writer.write(DamageEvent {
                target: entity,
                amount: 1.0,
                source: DamageSource::Starvation,
            });
        }
    }
}
