use bevy::prelude::*;

use crate::shared::states::app_state::AppState;

pub mod held_renderer;
pub mod item;
pub mod item_model;
pub mod mesh_cache;
pub mod tex_atlas;

/// 客户端渲染插件。
pub struct ClientRenderingPlugin;

impl Plugin for ClientRenderingPlugin {
    /// 注册客户端渲染资源和运行时系统。
    fn build(&self, app: &mut App) {
        app.init_resource::<item::cache::ItemModelCache>()
            .init_resource::<item::gui_icon_cache::GuiItemIconCache>()
            .add_systems(
                OnEnter(AppState::InGame),
                tex_atlas::init_block_render_assets_system,
            )
            .add_systems(
                Update,
                (
                    item::renderer::prepare_item_model_render_assets_system,
                    item::gui_icon_baker::retire_gui_item_icon_cameras_system,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
