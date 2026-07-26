use crate::game::player::combat::events::AttackEvent;
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::game::player::identity::{LocalPlayer, Player};
use crate::game::player::lifecycle::components::PlayerLifecycle;
use crate::game::player::survival::events::{DamageEvent, DamageSource};
use bevy::prelude::{Entity, MessageReader, MessageWriter, Query, Res, Transform, With};

/// 将本地攻击输入转换为对准范围内玩家实体的攻击请求。
pub fn melee_attack_input_system(
    actions: Res<PlayerActionState>,
    attacker_query: Query<
        (Entity, &Transform, &PlayerLifecycle),
        (With<Player>, With<LocalPlayer>),
    >,
    target_query: Query<(Entity, &Transform, &PlayerLifecycle), With<Player>>,
    mut writer: MessageWriter<AttackEvent>,
) {
    if !actions.just_pressed(PlayerAction::Attack) {
        return;
    }
    let Ok((attacker, attacker_transform, lifecycle)) = attacker_query.single() else {
        return;
    };
    if !lifecycle.is_alive() {
        return;
    }

    let forward = attacker_transform.forward().as_vec3();
    let mut closest = None;
    for (target, target_transform, lifecycle) in &target_query {
        if target == attacker || !lifecycle.is_alive() {
            continue;
        }
        let offset = target_transform.translation - attacker_transform.translation;
        let distance = offset.length();
        if distance > 3.0 || distance <= f32::EPSILON {
            continue;
        }
        if forward.dot(offset / distance) < 0.65 {
            continue;
        }
        if closest.is_none_or(|(_, best_distance)| distance < best_distance) {
            closest = Some((target, distance));
        }
    }
    if let Some((target, _)) = closest {
        writer.write(AttackEvent {
            attacker,
            target,
            amount: 2.0,
        });
    }
}

pub fn attack_damage_system(
    mut reader: MessageReader<AttackEvent>,
    mut writer: MessageWriter<DamageEvent>,
) {
    for attack in reader.read() {
        if attack.attacker == attack.target || attack.amount <= 0.0 {
            continue;
        }
        writer.write(DamageEvent {
            target: attack.target,
            amount: attack.amount,
            source: DamageSource::Entity(attack.attacker),
        });
    }
}
