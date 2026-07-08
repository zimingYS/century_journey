use bevy::prelude::*;

use crate::shared::states::app_state::AppState;

pub mod held_renderer;
pub mod item_model;
pub mod mesh_cache;
pub mod tex_atlas;

/// 客户端渲染功能插件
pub struct ClientRenderingPlugin;

impl Plugin for ClientRenderingPlugin {
    // 初始化手持网格缓存全局资源
    fn build(&self, app: &mut App) {
        app.init_resource::<mesh_cache::HeldMeshCache>()
            .init_resource::<item_model::ItemModelRenderAssets>()
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    tex_atlas::init_block_render_assets_system,
                    item_model::prepare_item_model_render_assets_system,
                )
                    .chain(),
            );
    }
}
