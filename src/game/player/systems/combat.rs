use bevy::prelude::*;
use crate::game::player::components::stats::{Health, Defense, Hunger};
use crate::game::player::events::{DamageEvent, HealEvent, DeathEvent};
use crate::game::player::components::Player;

/// 伤害处理 (含盔甲减伤)
pub fn damage_system(
    mut reader: MessageReader<DamageEvent>,
    mut query: Query<(&mut Health, Option<&Defense>), With<Player>>,
    mut death_writer: MessageWriter<DeathEvent>,
) {
    for event in reader.read() {
        if let Ok((mut health, defense_opt)) = query.get_mut(event.target) {
            let reduction = defense_opt.map_or(0.0, |d| d.damage_reduction());
            health.apply_damage(event.amount * (1.0 - reduction));
            if health.is_dead() {
                death_writer.write(DeathEvent { entity: event.target });
            }
        }
    }
}

/// 治疗处理
pub fn heal_system(
    mut reader: MessageReader<HealEvent>,
    mut query: Query<&mut Health, With<Player>>,
) {
    for event in reader.read() {
        if let Ok(mut health) = query.get_mut(event.target) {
            health.apply_heal(event.amount);
        }
    }
}

/// 死亡处理
pub fn death_system(
    mut reader: MessageReader<DeathEvent>,
    mut query: Query<(&mut Transform, &mut Health, &mut Hunger), With<Player>>,
) {
    for event in reader.read() {
        if let Ok((mut transform, mut health, mut hunger)) = query.get_mut(event.entity) {
            transform.translation = Vec3::new(0.0, 70.0, 0.0);
            *health = Health::default();
            *hunger = Hunger::default();
            log::info!("[Combat] 玩家重生在出生点");
        }
    }
}