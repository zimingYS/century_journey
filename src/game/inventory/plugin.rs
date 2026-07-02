use bevy::prelude::*;

/// Game 层 Inventory 模块 Plugin。
///
/// 只负责 Game 层运行时系统。
/// Definition/Registry/Loader/Texture 已在 Content 层的 ItemContentPlugin 中注册。
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        // Game 层目前没有额外的 Inventory 运行时系统需要注册。
        // ItemStack/SlotData/Container 等类型通过 ECS Resources 在各系统中使用。
    }
}
