//! 按固定步阶段组装环境、护甲、饥饿和生命规则。

use crate::game::player;
use crate::game::player::survival;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 注册生存消息以及 Survival、Combat 两个固定步阶段的系统。
pub struct PlayerSurvivalPlugin;

impl Plugin for PlayerSurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<survival::events::DamageEvent>()
            .add_message::<survival::events::HealEvent>()
            .add_message::<survival::events::FoodConsumedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    survival::hunger::use_food_system,
                    survival::hunger::action_cost_system,
                    survival::hunger::natural_regeneration_system,
                    survival::hunger::starvation_damage_system,
                    survival::protection::armor_calculation_system,
                    survival::environment::environment_damage_system,
                )
                    .chain()
                    .in_set(SimulationSet::Survival)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (
                    survival::health::damage_system,
                    survival::health::heal_system,
                )
                    .chain()
                    .in_set(SimulationSet::Combat)
                    .run_if(in_state(AppState::InGame))
                    .after(player::combat::attack::attack_damage_system),
            );
    }
}
