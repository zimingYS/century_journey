use bevy::prelude::*;

use crate::app::state::AppState;
use crate::content::item::registry::registry::{
    auto_generate_block_items_system, load_item_definitions_system, ItemRegistry,
};
use crate::content::item::texture::registry::load_item_textures_system;

/// Inventory 模块 Plugin
///
/// 负责: 初始化 Content 层的 ItemRegistry 和 ItemTextureRegistry。
/// Game 层不再拥有 Definition/Registry/Loader/Texture 职责。
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .add_systems(
                OnEnter(AppState::Loading),
                (load_item_textures_system,),
            )
            .add_systems(Startup, (load_item_textures_system,))
            .add_systems(
                OnEnter(AppState::InGame),
                (auto_generate_block_items_system, load_item_definitions_system).chain(),
            );
    }
}
