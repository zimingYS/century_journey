use bevy::prelude::*;

use crate::game::gameplay::block_action::{BlockBreakProgress, BlockBreakState};
use crate::game::player::action::PlayerActionState;
use crate::game::player::command::{
    PlayerCommandBuffer, apply_player_command_system, reset_player_command_pipeline_system,
};
use crate::game::player::events::{
    AttackEvent, DamageEvent, DeathEvent, FoodConsumedEvent, HealEvent, RespawnRequest,
};
use crate::game::player::systems::raycast::TargetVoxel;
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;

/// 纯游戏逻辑 Plugin — 仅注册 Game 层系统，不依赖 Client。
pub struct GamePlayerPlugin;

impl Plugin for GamePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TargetVoxel>()
            .init_resource::<PlayerActionState>()
            .init_resource::<PlayerCommandBuffer>()
            .init_resource::<BlockBreakState>()
            .init_resource::<BlockBreakProgress>()
            .init_resource::<crate::game::player::systems::combat::DeathRules>()
            .init_resource::<crate::game::player::systems::combat::LastDeathInfo>()
            .add_message::<AttackEvent>()
            .add_message::<DamageEvent>()
            .add_message::<HealEvent>()
            .add_message::<FoodConsumedEvent>()
            .add_message::<DeathEvent>()
            .add_message::<RespawnRequest>()
            .add_systems(
                OnEnter(AppState::InGame),
                reset_player_command_pipeline_system,
            )
            .add_systems(
                FixedUpdate,
                apply_player_command_system
                    .in_set(SimulationSet::Commands)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                crate::game::player::systems::movement::player_movement_system
                    .in_set(SimulationSet::Movement)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                crate::game::player::systems::gravity::player_gravity_system
                    .in_set(SimulationSet::Physics)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                crate::game::player::systems::raycast::update_raycast_system
                    .in_set(SimulationSet::Targeting)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (
                    crate::game::player::systems::interaction::voxel_interaction_system,
                    crate::game::player::systems::interaction::drop_active_hotbar_action_system,
                    crate::game::player::systems::interaction::drop_item_system,
                )
                    .chain()
                    .in_set(SimulationSet::Interaction)
                    .run_if(in_state(AppState::InGame))
                    .run_if(crate::game::player::systems::combat::player_is_alive),
            )
            .add_systems(
                FixedUpdate,
                (
                    crate::game::player::systems::hunger::use_food_system,
                    crate::game::player::systems::hunger::action_cost_system,
                    crate::game::player::systems::hunger::natural_regeneration_system,
                    crate::game::player::systems::hunger::starvation_damage_system,
                    crate::game::player::systems::armor_calc::armor_calculation_system,
                    crate::game::player::systems::environment::environment_damage_system,
                )
                    .chain()
                    .in_set(SimulationSet::Survival)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (
                    crate::game::player::systems::combat::melee_attack_input_system,
                    crate::game::player::systems::combat::attack_damage_system,
                    crate::game::player::systems::combat::damage_system,
                    crate::game::player::systems::combat::heal_system,
                    crate::game::player::systems::combat::death_system,
                    crate::game::player::systems::combat::respawn_request_system,
                    crate::game::player::systems::combat::respawn_transition_system,
                )
                    .chain()
                    .in_set(SimulationSet::Combat)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                crate::game::player::systems::raycast::draw_voxel_highlight_system
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
