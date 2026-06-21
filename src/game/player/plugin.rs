use bevy::prelude::*;

use crate::game::player::events::{DamageEvent, DeathEvent, HealEvent};
use crate::game::player::systems::raycast::TargetVoxel;

/// 纯游戏逻辑 Plugin — 仅注册 Game 层系统，不依赖 Client。
pub struct GamePlayerPlugin;

impl Plugin for GamePlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TargetVoxel>()
            .add_message::<DamageEvent>()
            .add_message::<HealEvent>()
            .add_message::<DeathEvent>()
            .add_systems(Update, (
                crate::game::player::systems::movement::player_movement_system,
                crate::game::player::systems::gravity::player_gravity_system,
            ))
            .add_systems(Update, (
                crate::game::player::systems::interaction::voxel_interaction_system,
                crate::game::player::systems::raycast::draw_voxel_highlight_system,
                crate::game::player::systems::raycast::update_raycast_system,
            ))
            .add_systems(Update, (
                crate::game::player::systems::combat::damage_system,
                crate::game::player::systems::combat::heal_system,
                crate::game::player::systems::combat::death_system,
            ).chain())
            .add_systems(Update, (
                crate::game::player::systems::hunger::action_cost_system,
                crate::game::player::systems::hunger::natural_regeneration_system,
                crate::game::player::systems::hunger::starvation_damage_system,
            ).chain())
            .add_systems(Update, crate::game::player::systems::armor_calc::armor_calculation_system)
            .add_systems(Update, crate::game::player::systems::interaction::drop_item_system);
    }
}
