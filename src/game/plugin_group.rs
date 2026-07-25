use bevy::prelude::*;

use crate::game::crafting::plugin::CraftingPlugin;
use crate::game::gameplay::GameplayPlugin;
use crate::game::inventory::plugin::InventoryPlugin;
use crate::game::simulation::SimulationPlugin;
use crate::game::world::WorldPlugin;

/// Game 层插件聚合入口。
///
/// 当前保持单机客户端既有的 Game 注册顺序。玩家规则暂时仍由
/// `ClientPlayerPlugin` 兼容注册，待无窗口装配测试补齐后再迁入此处。
pub struct GamePluginGroup;

impl Plugin for GamePluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayPlugin,
            SimulationPlugin,
            CraftingPlugin,
            WorldPlugin,
            InventoryPlugin,
        ));
    }
}
