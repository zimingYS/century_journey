use bevy::prelude::*;

use crate::content::item::registry::registry::{
    ItemRegistry, auto_generate_block_items_system, load_item_definitions_system,
};
use crate::content::item::texture::registry::load_item_textures_system;
use crate::shared::states::app_state::AppState;

/// Content 层 Item 插件。
///
/// 负责初始化 ItemRegistry、ItemTextureRegistry 和 Content 层加载系统。
/// 属于 Content 层，不依赖 Game 层或 Client 层。
pub struct ItemContentPlugin;

impl Plugin for ItemContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            // 纹理在 Loading 阶段加载
            .add_systems(OnEnter(AppState::Loading), (load_item_textures_system,))
            // 进入游戏后加载物品定义 + 自动生成方块物品
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    auto_generate_block_items_system,
                    load_item_definitions_system,
                )
                    .chain(),
            );
    }
}
