//! 组装 Content 层各类定义与生命周期插件。

use bevy::prelude::*;

use crate::content::biome::plugin::BiomeContentPlugin;
use crate::content::block::VoxelPlugin;
use crate::content::cloud::plugin::CloudContentPlugin;
use crate::content::item::plugin::ItemContentPlugin;
use crate::content::lifecycle::ContentLifecyclePlugin;
use crate::content::loot::LootPlugin;
use crate::content::ore_vein::plugin::OreVeinContentPlugin;
use crate::content::recipe::plugin::RecipeContentPlugin;
use crate::content::tag::plugin::TagContentPlugin;
use crate::content::vegetation::VegetationContentPlugin;

/// Content 层插件聚合入口。
///
/// 叶子插件只负责自己的注册表和内容生命周期；本聚合器保持现有注册顺序。
pub struct ContentPluginGroup;

impl Plugin for ContentPluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ContentLifecyclePlugin,
            BiomeContentPlugin,
            ItemContentPlugin,
            VoxelPlugin,
            LootPlugin,
            TagContentPlugin,
            RecipeContentPlugin,
            VegetationContentPlugin,
            OreVeinContentPlugin,
            CloudContentPlugin,
        ));
    }
}
