//! 组装 Game 层的完整存档能力。

use super::SaveConfig;
use super::debug_controls::save_load_keybind_system;
use super::player::PlayerSavePlugin;
use super::world::WorldSavePlugin;
use crate::shared::states::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{IntoScheduleConfigs, in_state};

/// 组装 Game 层的玩家存档、世界存档以及持久化队列。
///
/// 本插件依赖玩家、背包和世界领域插件提供的运行时数据，
/// 因此应由 `GamePluginGroup` 在这些领域插件之后注册。
pub struct GameSavePlugin;

impl Plugin for GameSavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveConfig::default())
            .add_plugins((WorldSavePlugin, PlayerSavePlugin))
            .add_systems(
                Update,
                save_load_keybind_system.run_if(in_state(AppState::InGame)),
            );
    }
}
