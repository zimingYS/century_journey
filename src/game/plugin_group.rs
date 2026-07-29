use bevy::prelude::*;

use crate::game::crafting::plugin::CraftingPlugin;
use crate::game::gameplay::GameplayPlugin;
use crate::game::inventory::InventoryPlugin;
use crate::game::player::plugin::GamePlayerPlugin;
use crate::game::save::GameSavePlugin;
use crate::game::simulation::SimulationPlugin;
use crate::game::world::GameWorldPlugin;

/// Game 灞傛彃浠惰仛鍚堝叆鍙ｃ€?///
/// GamePluginGroup 鐩存帴娉ㄥ唽鏉冨▉鐜╂硶鎻掍欢锛?/// 鍖呮嫭 GamePlayerPlugin锛?/// ClientPlayerPlugin 浠呬繚鐣欐湰鍦拌〃鐜般€佺浉鏈哄拰妯″瀷缁戝畾銆?
pub struct GamePluginGroup;

impl Plugin for GamePluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayPlugin,
            SimulationPlugin,
            CraftingPlugin,
            GameWorldPlugin,
            InventoryPlugin,
            GamePlayerPlugin,
            GameSavePlugin,
        ));
    }
}
