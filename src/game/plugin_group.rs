//! 组装 Game 层的权威玩法插件。

use bevy::prelude::*;

use crate::game::crafting::CraftingPlugin;
use crate::game::gameplay::GameplayPlugin;
use crate::game::inventory::InventoryPlugin;
use crate::game::player::GamePlayerPlugin;
use crate::game::save::GameSavePlugin;
use crate::game::simulation::SimulationPlugin;
use crate::game::world::GameWorldPlugin;

/// Game 层插件聚合入口。
///
/// 本插件只组装权威模拟、世界、物品栏、玩家与存档领域；客户端表现由 Client 层自行组装。
pub struct GamePluginGroup;

impl Plugin for GamePluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SimulationPlugin,
            GameplayPlugin,
            GameWorldPlugin,
            InventoryPlugin,
            GamePlayerPlugin,
            CraftingPlugin,
            GameSavePlugin,
        ));
    }
}
