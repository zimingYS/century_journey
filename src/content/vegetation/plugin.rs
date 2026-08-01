//! 在内容重载阶段重建树种运行时注册表。

use crate::content::block::registry::BlockRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::validation::ContentCompilation;
use crate::content::vegetation::registry::TreeSpeciesRegistry;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 组装树种内容注册表及其重载流程。
pub struct VegetationContentPlugin;

impl Plugin for VegetationContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TreeSpeciesRegistry>().add_systems(
            OnEnter(AppState::InGame),
            load_tree_species_system
                .in_set(ContentReloadSet::Load)
                .run_if(content_reload_requested),
        );
    }
}

fn load_tree_species_system(
    mut registry: ResMut<TreeSpeciesRegistry>,
    block_registry: Res<BlockRegistry>,
    compilation: Res<ContentCompilation>,
) {
    let definitions = compilation.content.tree_species.clone();
    match registry.replace_definitions(definitions, &block_registry) {
        Ok(()) => log::info!("[植被内容] 已加载 {} 个树种定义", registry.len()),
        Err(error) => log::error!("[植被内容] 无法构建树种注册表: {error}"),
    }
}
