use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::stats::{Defense, Health, Hunger};
use crate::game::player::components::{
    EnvironmentExposure, FoodUseState, LocalPlayer, Player, PlayerGravity, PlayerLifeState,
    PlayerLifecycle, PlayerVelocity, RespawnPoint,
};
use crate::game::player::events::{
    AttackEvent, DamageEvent, DamageSource, DeathEvent, HealEvent, RespawnRequest,
};
use crate::game::world::entity::dropped_item::{
    DroppedItemVelocity, spawn_dropped_item_with_velocity,
};
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeathDropRule {
    KeepInventory,
    #[default]
    DropInventory,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct DeathRules {
    pub drop_rule: DeathDropRule,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LastDeathInfo {
    pub source: Option<DamageSource>,
    pub position: Vec3,
    pub dropped_stacks: usize,
}

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

/// 伤害处理；同一次死亡只产生一个死亡事件。
pub fn damage_system(
    mut reader: MessageReader<DamageEvent>,
    mut query: Query<(&mut Health, Option<&Defense>, &mut PlayerLifecycle), With<Player>>,
    mut death_writer: MessageWriter<DeathEvent>,
) {
    for event in reader.read() {
        let Ok((mut health, defense_opt, mut lifecycle)) = query.get_mut(event.target) else {
            continue;
        };
        if !lifecycle.is_alive() || !event.amount.is_finite() || event.amount <= 0.0 {
            continue;
        }
        let reduction = defense_opt.map_or(0.0, Defense::damage_reduction);
        health.apply_damage(event.amount * (1.0 - reduction));
        if health.is_dead() {
            lifecycle.state = PlayerLifeState::Dead;
            death_writer.write(DeathEvent {
                entity: event.target,
                source: event.source,
            });
        }
    }
}

/// 治疗处理
pub fn heal_system(
    mut reader: MessageReader<HealEvent>,
    mut query: Query<(&mut Health, &PlayerLifecycle), With<Player>>,
) {
    for event in reader.read() {
        if let Ok((mut health, lifecycle)) = query.get_mut(event.target)
            && lifecycle.is_alive()
        {
            health.apply_heal(event.amount);
        }
    }
}

/// 进入 Dead 状态，按规则生成死亡掉落并等待玩家确认重生。
pub fn death_system(
    mut reader: MessageReader<DeathEvent>,
    mut query: Query<
        (
            &Transform,
            &mut PlayerVelocity,
            &mut PlayerGravity,
            &PlayerLifecycle,
            &mut InventoryState,
        ),
        With<Player>,
    >,
    gamemode: Res<PlayerGameMode>,
    rules: Res<DeathRules>,
    mut last_death: ResMut<LastDeathInfo>,
    mut commands: Commands,
) {
    for event in reader.read() {
        let Ok((transform, mut velocity, mut gravity, lifecycle, mut inventory)) =
            query.get_mut(event.entity)
        else {
            continue;
        };
        if lifecycle.state != PlayerLifeState::Dead {
            continue;
        }
        velocity.horizontal = Vec3::ZERO;
        gravity.velocity_y = 0.0;
        gravity.fall_distance = 0.0;
        inventory.opened = false;

        let should_drop = gamemode.is_survival() && rules.drop_rule == DeathDropRule::DropInventory;
        let drops = if should_drop {
            drain_death_inventory(&mut inventory)
        } else {
            Vec::new()
        };
        for (index, stack) in drops.iter().cloned().enumerate() {
            let angle = index as f32 * 2.399_963_1;
            let offset = Vec3::new(angle.cos(), 0.35, angle.sin()) * 0.45;
            let position = transform.translation + offset;
            spawn_dropped_item_with_velocity(
                &mut commands,
                position,
                stack,
                DroppedItemVelocity::passive(position),
            );
        }
        *last_death = LastDeathInfo {
            source: Some(event.source),
            position: transform.translation,
            dropped_stacks: drops.len(),
        };
        log::info!(
            "[生存] 玩家死亡，原因={}，掉落 {} 组物品",
            event.source.display_name(),
            drops.len()
        );
    }
}

pub fn respawn_request_system(
    mut reader: MessageReader<RespawnRequest>,
    mut query: Query<
        (
            &mut Transform,
            &mut Health,
            &mut Hunger,
            &mut PlayerLifecycle,
            &RespawnPoint,
            &mut PlayerVelocity,
            &mut PlayerGravity,
            &mut EnvironmentExposure,
            &mut FoodUseState,
        ),
        With<Player>,
    >,
) {
    for request in reader.read() {
        let Ok((
            mut transform,
            mut health,
            mut hunger,
            mut lifecycle,
            respawn,
            mut velocity,
            mut gravity,
            mut exposure,
            mut food_use,
        )) = query.get_mut(request.entity)
        else {
            continue;
        };
        if lifecycle.state != PlayerLifeState::Dead {
            continue;
        }
        transform.translation = respawn.0;
        *health = Health::default();
        *hunger = Hunger::default();
        *velocity = PlayerVelocity::default();
        *gravity = PlayerGravity::default();
        *exposure = EnvironmentExposure::default();
        food_use.cancel();
        lifecycle.state = PlayerLifeState::Respawning;
        lifecycle.respawn_remaining = 0.15;
    }
}

pub fn respawn_transition_system(
    time: Res<Time>,
    mut query: Query<&mut PlayerLifecycle, With<Player>>,
) {
    for mut lifecycle in &mut query {
        if lifecycle.state != PlayerLifeState::Respawning {
            continue;
        }
        lifecycle.respawn_remaining -= time.delta_secs();
        if lifecycle.respawn_remaining <= 0.0 {
            lifecycle.state = PlayerLifeState::Alive;
            lifecycle.respawn_remaining = 0.0;
        }
    }
}

pub fn player_is_alive(query: Query<&PlayerLifecycle, With<Player>>) -> bool {
    query.single().is_ok_and(PlayerLifecycle::is_alive)
}

fn drain_death_inventory(inventory: &mut InventoryState) -> Vec<ItemStack> {
    let mut drops = Vec::new();
    drops.extend(inventory.hotbar.stacks.iter_mut().filter_map(Option::take));
    drops.extend(
        inventory
            .survival
            .backpack
            .iter_mut()
            .filter_map(Option::take),
    );
    drops.extend(
        inventory
            .survival
            .equipment
            .iter_mut()
            .filter_map(Option::take),
    );
    drops.extend(
        inventory
            .survival
            .accessories
            .iter_mut()
            .filter_map(Option::take),
    );
    if let Some(stack) = inventory.cursor.take_stack() {
        drops.push(stack);
    }
    drops.retain(|stack| !stack.is_empty());
    drops
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/systems/combat.rs"]
mod stage_seven_tests;
