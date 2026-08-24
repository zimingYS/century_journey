//! 组装云层定义加载与注册表刷新。

use crate::content::cloud::registry::CloudRegistry;
use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::validation::ContentCompilation;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

/// 云层内容加载与注册插件。
pub struct CloudContentPlugin;

impl Plugin for CloudContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CloudRegistry>().add_systems(
            OnEnter(AppState::InGame),
            load_clouds_system
                .in_set(ContentReloadSet::Load)
                .run_if(content_reload_requested),
        );
    }
}

fn load_clouds_system(mut registry: ResMut<CloudRegistry>, compilation: Res<ContentCompilation>) {
    let definitions = compilation.content.clouds.clone();
    registry.replace_definitions(definitions);
    log::info!("[云层] 已加载 {} 个云层定义", registry.len());
}
