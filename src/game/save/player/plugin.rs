//! 组装玩家数据加载、脏状态跟踪和自动保存流程。

use super::PlayerSaveManager;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::game::save::player::runtime::dirty_tracking::{
    gamemode_dirty_tracking_system, inventory_dirty_tracking_system, player_position_dirty_system,
};
use crate::game::save::player::runtime::load::load_player_on_enter_system;
use crate::game::save::player::runtime::write::{auto_save_player_system, save_on_exit_system};
use crate::shared::states::AppState;
use bevy::app::{App, Last, Plugin, Update};
use bevy::prelude::{IntoScheduleConfigs, OnEnter, in_state};

/// 组装玩家存档的加载、变化跟踪和写入系统。
pub(in crate::game::save) struct PlayerSavePlugin;

impl Plugin for PlayerSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSaveManager>()
            .add_systems(
                OnEnter(AppState::InGame),
                load_player_on_enter_system
                    .in_set(ContentReloadSet::Consumers)
                    .run_if(content_reload_requested),
            )
            .add_systems(
                Update,
                (
                    player_position_dirty_system,
                    inventory_dirty_tracking_system,
                    gamemode_dirty_tracking_system,
                    auto_save_player_system,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(Last, save_on_exit_system.run_if(in_state(AppState::InGame)));
    }
}
