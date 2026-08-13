//! 组装区块、物品和纹理图集等客户端渲染子系统。

use bevy::prelude::*;

use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::shared::states::app_state::AppState;

pub(crate) mod constants;
pub(crate) mod distant;
pub mod item;
pub mod lighting;
pub mod tex_atlas;
pub mod world;

/// 客户端渲染插件。
pub struct ClientRenderingPlugin;

impl Plugin for ClientRenderingPlugin {
    /// 注册客户端渲染资源和运行时系统。
    fn build(&self, app: &mut App) {
        world::register_mesh_lifecycle_resources(app);
        app.add_plugins(lighting::VoxelLightingPlugin)
            .init_resource::<item::cache::ItemModelCache>()
            .init_resource::<item::gui_icon_cache::GuiItemIconCache>()
            .init_resource::<world::MeshBuildChannel>()
            .init_resource::<world::CachedBlockInfo>()
            .init_resource::<distant::DistantTerrainConfig>()
            .init_resource::<distant::DistantTerrainRuntime>()
            .init_resource::<distant::DistantTerrainBuildChannel>()
            .add_systems(
                OnEnter(AppState::InGame),
                tex_atlas::init_block_render_assets_system
                    .in_set(ContentReloadSet::Consumers)
                    .run_if(content_reload_requested),
            )
            .add_systems(
                Update,
                (
                    item::renderer::prepare_gui_item_icons_system,
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
                    world::collect_priority_mesh_rebuilds,
                    world::spawn_mesh_build_tasks
                        .after(crate::game::world::generation::receive_structure_results),
                    world::receive_mesh_results,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    distant::sync_distant_terrain_plan_system.after(world::spawn_mesh_build_tasks),
                    distant::spawn_distant_terrain_tasks_system,
                    distant::receive_distant_terrain_results_system,
                    distant::sync_distant_terrain_camera_range_system,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                distant::initialize_distant_terrain_system
                    .after(tex_atlas::init_block_render_assets_system),
            )
            .add_systems(OnExit(AppState::InGame), world::clear_mesh_lifecycle)
            .add_systems(
                OnEnter(AppState::WorldLoading),
                distant::clear_distant_terrain_system,
            )
            .add_systems(
                OnEnter(AppState::MainMenu),
                distant::clear_distant_terrain_system,
            );
    }
}
