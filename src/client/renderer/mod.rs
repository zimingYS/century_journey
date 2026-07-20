use bevy::prelude::*;

use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::shared::states::app_state::AppState;

pub mod item;
pub mod item_model;
pub mod mesh_cache;
pub mod tex_atlas;
pub mod world;

/// 客户端渲染插件。
pub struct ClientRenderingPlugin;

impl Plugin for ClientRenderingPlugin {
    /// 注册客户端渲染资源和运行时系统。
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::client::presentation::ClientPresentation>()
            .init_resource::<item::cache::ItemModelCache>()
            .init_resource::<item::gui_icon_cache::GuiItemIconCache>()
            .init_resource::<world::MeshBuildChannel>()
            .init_resource::<world::CachedBlockInfo>()
            .add_systems(
                OnEnter(AppState::InGame),
                tex_atlas::init_block_render_assets_system
                    .in_set(ContentReloadSet::Consumers)
                    .run_if(content_reload_requested),
            )
            .add_systems(
                Update,
                (
                    item::renderer::prepare_item_model_render_assets_system,
                    item::gui_icon_baker::retire_gui_item_icon_cameras_system,
                    world::dropped_item::dropped_item_visual_system,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                world::rebuild_block_info_snapshot
                    .before(world::spawn_mesh_build_tasks)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    world::spawn_mesh_build_tasks
                        .after(crate::game::world::systems::receive_structure_results),
                    world::receive_mesh_results,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
