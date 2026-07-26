use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::{PlayerLifeState, PlayerLifecycle, RespawnPoint};
use crate::game::player::lifecycle::events::{DeathEvent, RespawnRequest};
use crate::game::player::movement::components::PlayerVelocity;
use crate::game::player::physics::components::PlayerGravity;
use crate::game::player::survival::environment::EnvironmentExposure;
use crate::game::player::survival::events::DamageSource;
use crate::game::player::survival::health::Health;
use crate::game::player::survival::hunger::{FoodUseState, Hunger};
use crate::game::world::entity::dropped_item::{
    DroppedItemVelocity, spawn_dropped_item_with_velocity,
};
use bevy::math::Vec3;
use bevy::prelude::{Commands, MessageReader, Query, Res, ResMut, Resource, Time, Transform, With};

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
