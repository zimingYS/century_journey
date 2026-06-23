use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use crate::app::plugin::CorePlugin;
use crate::client::player::ClientPlayerPlugin;
use crate::client::renderer::RenderingPlugin;
use crate::client::sky::SkyPlugin;
use crate::client::startup::plugin::ClientStartupPlugin;
use crate::client::ui::UIPlugin;
use crate::content::block::VoxelPlugin;
use crate::content::loot::LootPlugin;
use crate::engine::asset::AssetPlugin;
use crate::engine::task::TaskPlugin;
use crate::game::gameplay::GameplayPlugin;
use crate::game::inventory::plugin::InventoryPlugin;
use crate::game::world::WorldPlugin;
use crate::shared::tag::TagPlugin;

/// 客户端 Plugin 集合。
pub struct ClientPluginGroup;

impl PluginGroup for ClientPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            // Engine 层（最先注册，其他插件依赖）
            .add(AssetPlugin)
            .add(TaskPlugin)
            // 业务层
            .add(CorePlugin)
            .add(GameplayPlugin)
            .add(LootPlugin)
            .add(VoxelPlugin)
            .add(RenderingPlugin)
            .add(TagPlugin)
            .add(ClientPlayerPlugin)
            .add(WorldPlugin)
            .add(SkyPlugin)
            .add(UIPlugin)
            .add(InventoryPlugin)
            .add(ClientStartupPlugin)
    }
}
