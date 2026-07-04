use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use crate::app::plugin::CorePlugin;
use crate::client::player::ClientPlayerPlugin;
use crate::client::renderer::ClientRenderingPlugin;
use crate::client::sky::SkyPlugin;
use crate::client::startup::plugin::ClientStartupPlugin;
use crate::client::ui::UIPlugin;
use crate::content::block::VoxelPlugin;
use crate::content::item::plugin::ItemContentPlugin;
use crate::content::loot::LootPlugin;
use crate::content::recipe::plugin::RecipeContentPlugin;
use crate::content::tag::TagContentPlugin;
use crate::engine::asset::AssetPlugin;
use crate::engine::task::TaskPlugin;
use crate::game::gameplay::GameplayPlugin;
use crate::game::inventory::plugin::InventoryPlugin;
use crate::game::world::WorldPlugin;

/// 客户端 Plugin 集合。
pub struct ClientPluginGroup;

impl PluginGroup for ClientPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            // Engine 层（最先注册，其他插件依赖）
            .add(AssetPlugin)
            .add(TaskPlugin)
            // Content 层（数据驱动，不依赖 Game/Client）
            .add(ItemContentPlugin)
            .add(VoxelPlugin)
            .add(LootPlugin)
            .add(TagContentPlugin)
            .add(RecipeContentPlugin)
            // Game 层（运行时逻辑，依赖 Content）
            .add(GameplayPlugin)
            .add(WorldPlugin)
            .add(InventoryPlugin)
            // App 层
            .add(CorePlugin)
            // Client 层（渲染/UI，依赖 Content+Game）
            .add(ClientRenderingPlugin)
            .add(ClientPlayerPlugin)
            .add(SkyPlugin)
            .add(UIPlugin)
            .add(ClientStartupPlugin)
    }
}
