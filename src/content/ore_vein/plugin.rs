//! 在内容重载阶段重建矿脉运行时注册表。

use crate::content::block::registry::BlockRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::ore_vein::registry::OreVeinRegistry;
use crate::content::validation::ContentCompilation;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 组装矿脉内容注册表及其重载流程。
pub struct OreVeinContentPlugin;

impl Plugin for OreVeinContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OreVeinRegistry>().add_systems(
            OnEnter(AppState::InGame),
            load_ore_veins_system
                .in_set(ContentReloadSet::Load)
                .run_if(content_reload_requested),
        );
    }
}

fn load_ore_veins_system(
    mut registry: ResMut<OreVeinRegistry>,
    block_registry: Res<BlockRegistry>,
    compilation: Res<ContentCompilation>,
) {
    let definitions = compilation.content.ore_veins.clone();
    match registry.replace_definitions(definitions, &block_registry) {
        Ok(()) => log::info!("[矿脉内容] 已加载 {} 条矿脉定义", registry.len()),
        Err(error) => log::error!("[矿脉内容] 无法构建矿脉注册表: {error}"),
    }
}
