use bevy::prelude::*;

use crate::app::state::AppState;

/// Inventory 模块 Plugin — 负责物品注册表的初始化和物品系统的自动生成。
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::game::inventory::item::registry::ItemRegistry>()
            .add_systems(
                OnEnter(AppState::Loading),
                (crate::game::inventory::item::texture_registry::load_item_textures_system,),
            )
            .add_systems(
                Startup,
                (crate::game::inventory::item::texture_registry::load_item_textures_system,),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    crate::game::inventory::item::registry::auto_generate_block_items_system,
                    crate::game::inventory::item::registry::load_item_definitions_system,
                )
                    .chain(),
            );
    }
}
