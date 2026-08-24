//! 组装生物群系定义加载与注册表刷新。

use crate::content::biome::registry::BiomeRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::validation::ContentCompilation;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

/// 生物群系内容加载与注册插件。
pub struct BiomeContentPlugin;

impl Plugin for BiomeContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BiomeRegistry>().add_systems(
            OnEnter(AppState::InGame),
            load_biomes_system
                .in_set(ContentReloadSet::Load)
                .run_if(content_reload_requested),
        );
    }
}

fn load_biomes_system(mut registry: ResMut<BiomeRegistry>, compilation: Res<ContentCompilation>) {
    let definitions = compilation.content.biomes.clone();
    match registry.replace_definitions(definitions) {
        Ok(()) => log::info!("[群系] 已加载 {} 个群系定义", registry.len()),
        Err(error) => log::error!("[群系] 注册表构建失败: {error}"),
    }
}
