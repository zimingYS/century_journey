use bevy::prelude::*;

use crate::game::crafting::plugin::CraftingPlugin;
use crate::game::gameplay::GameplayPlugin;
use crate::game::inventory::plugin::InventoryPlugin;
use crate::game::player::plugin::GamePlayerPlugin;
use crate::game::simulation::SimulationPlugin;
use crate::game::world::WorldPlugin;

/// Game 层插件聚合入口。
///
/// GamePluginGroup 直接注册权威玩法插件，
/// 包括 GamePlayerPlugin；
/// ClientPlayerPlugin 仅保留本地表现、相机和模型绑定。
pub struct GamePluginGroup;

impl Plugin for GamePluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayPlugin,
            SimulationPlugin,
            CraftingPlugin,
            WorldPlugin,
            InventoryPlugin,
            GamePlayerPlugin,
        ));
    }
}
