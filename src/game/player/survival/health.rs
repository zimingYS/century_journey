//! 处理生命值约束、伤害、治疗和死亡消息转换。

use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::{PlayerLifeState, PlayerLifecycle};
use crate::game::player::lifecycle::events::DeathEvent;
use crate::game::player::survival::events::{DamageEvent, HealEvent};
use crate::game::player::survival::protection::Defense;
use bevy::prelude::{Component, MessageReader, MessageWriter, Query, Res, With};
use crate::game::gameplay::gamemode::PlayerGameMode;

/// 生命值
#[derive(Component, Debug, Clone)]
pub struct Health {
    /// 当前生命值。
    pub current: f32,
    /// 允许恢复到的生命值上限。
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 20.0,
            max: 20.0,
        }
    }
}

impl Health {
    /// 返回供 HUD 使用且约束在零到一之间的生命比例。
    pub fn fraction(&self) -> f32 {
        if !self.current.is_finite() || !self.max.is_finite() || self.max <= 0.0 {
            return 0.0;
        }
        (self.current / self.max).clamp(0.0, 1.0)
    }
    /// 判断当前生命值是否已经耗尽。
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
    /// 应用有限正数伤害，并把生命值下限约束为零。
    pub fn apply_damage(&mut self, amount: f32) {
        if amount.is_finite() && amount > 0.0 {
            self.current = (self.current - amount).max(0.0);
        }
    }
    /// 应用有限正数治疗，并把生命值上限约束为 `max`。
    pub fn apply_heal(&mut self, amount: f32) {
        if amount.is_finite() && amount > 0.0 {
            self.current = (self.current + amount).min(self.max);
        }
    }
}

/// 伤害处理；同一次死亡只产生一个死亡事件。
pub fn damage_system(
    mut reader: MessageReader<DamageEvent>,
    mut query: Query<(&mut Health, Option<&Defense>, &mut PlayerLifecycle), With<Player>>,
    mut death_writer: MessageWriter<DeathEvent>,
    gamemode: Res<PlayerGameMode>,
) {
    for event in reader.read() {
        let Ok((mut health, defense_opt, mut lifecycle)) = query.get_mut(event.target) else {
            continue;
        };

        if gamemode.is_creative(){
            continue;
        }

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

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/survival/health.rs"]
mod tests;
