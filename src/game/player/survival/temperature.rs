//! 从权威天气温度推导过热/过冷的生存惩罚。
//!
//! 温度是「环境生存」的第三根支柱：过热加速口渴（出汗）、过冷加速饥饿（保暖耗能），
//! 二者都会降低移动速度，极端温度按周期造成伤害。数据源是 `WeatherCell.temperature_c`。

use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::components::PlayerLifecycle;
use crate::game::player::survival::events::{DamageEvent, DamageSource};
use crate::game::player::survival::hunger::Hunger;
use crate::game::player::survival::thirst::Thirst;
use crate::game::world::weather::WeatherCell;
use bevy::prelude::*;

/// 过热起始温度（°C），高于此值开始口渴加速与减速。
const OVERHEAT_THRESHOLD: f32 = 32.0;
/// 失温起始温度（°C），低于此值开始饥饿加速与减速。
const COLD_THRESHOLD: f32 = 10.0;
/// 过热致死温度（°C），高于此值周期扣血。
const OVERHEAT_LETHAL: f32 = 45.0;
/// 失温致死温度（°C），低于此值周期扣血。
const COLD_LETHAL: f32 = -5.0;

/// 温度暴露状态：缓存减速倍率与极限伤害冷却，供移动与伤害系统消费。
#[derive(Component, Debug, Clone, Copy)]
pub struct TemperatureExposure {
    /// 当前温度施加的移动速度倍率（1.0=正常，过热/过冷 <1.0）。
    pub speed_multiplier: f32,
    /// 下一次极限温度伤害生效前的固定步倒计时。
    pub damage_cooldown: f32,
}

impl Default for TemperatureExposure {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            damage_cooldown: 0.0,
        }
    }
}

/// 温度生存规则：过热口渴加速、过冷饥饿加速、减速倍率与极限周期扣血。
pub fn temperature_survival_system(
    time: Res<Time<Fixed>>,
    weather: Res<WeatherCell>,
    gamemode: Res<PlayerGameMode>,
    mut query: Query<
        (
            Entity,
            &mut Thirst,
            &mut Hunger,
            &mut TemperatureExposure,
            &PlayerLifecycle,
        ),
        With<Player>,
    >,
    mut writer: MessageWriter<DamageEvent>,
) {
    let temp = weather.temperature_c;
    let dt = time.delta_secs();

    for (entity, mut thirst, mut hunger, mut exposure, lifecycle) in &mut query {
        if !lifecycle.is_alive() || gamemode.is_creative() {
            exposure.speed_multiplier = 1.0;
            continue;
        }

        exposure.speed_multiplier = temperature_speed_multiplier(temp);

        // 过热 → 口渴加速（出汗流失水分）。
        if temp > OVERHEAT_THRESHOLD {
            thirst.exhaust((temp - OVERHEAT_THRESHOLD) * 0.02 * dt);
        }
        // 过冷 → 饥饿加速（产热耗能）。
        if temp < COLD_THRESHOLD {
            hunger.exhaust((COLD_THRESHOLD - temp) * 0.015 * dt);
        }

        // 极限温度周期扣血（每 4 秒）。
        exposure.damage_cooldown = (exposure.damage_cooldown - dt).max(0.0);
        let source = if temp > OVERHEAT_LETHAL {
            Some(DamageSource::Overheating)
        } else if temp < COLD_LETHAL {
            Some(DamageSource::Hypothermia)
        } else {
            None
        };
        if let Some(source) = source
            && exposure.damage_cooldown <= 0.0
        {
            writer.write(DamageEvent {
                target: entity,
                amount: 1.0,
                source,
            });
            exposure.damage_cooldown = 4.0;
        }
    }
}

/// 温度对移动速度的倍率（舒适=1.0，过热/过冷线性降到 0.5）。
pub fn temperature_speed_multiplier(temp: f32) -> f32 {
    if temp > OVERHEAT_THRESHOLD {
        let ratio = (temp - OVERHEAT_THRESHOLD) / (OVERHEAT_LETHAL - OVERHEAT_THRESHOLD);
        (1.0 - ratio * 0.5).max(0.5)
    } else if temp < COLD_THRESHOLD {
        let ratio = (COLD_THRESHOLD - temp) / (COLD_THRESHOLD - COLD_LETHAL);
        (1.0 - ratio * 0.5).max(0.5)
    } else {
        1.0
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/survival/temperature.rs"]
mod tests;
