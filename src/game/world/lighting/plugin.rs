//! 组装权威光照数据与固定步传播系统。

use bevy::prelude::*;

use crate::game::simulation::SimulationSet;
use crate::game::world::lighting::local;
use crate::game::world::lighting::resources::{
    CachedLightInfo, LightingBuildChannel, LightingRebuildTracker, WorldLighting,
};
use crate::game::world::lighting::systems::{
    clear_world_lighting, rebuild_light_info_snapshot, receive_lighting_results,
    schedule_lighting_rebuild,
};

/// 组装权威光照数据与固定步传播系统。
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        local::register_resources(app);
        app.init_resource::<WorldLighting>()
            .init_resource::<CachedLightInfo>()
            .init_resource::<LightingRebuildTracker>()
            .init_resource::<LightingBuildChannel>()
            .add_systems(
                FixedUpdate,
                (
                    rebuild_light_info_snapshot,
                    local::prune_unloaded_lighting,
                    local::receive_local_lighting_results,
                    receive_lighting_results,
                    local::queue_pending_chunk_lighting,
                    local::sync_changed_block_sources,
                    local::schedule_local_lighting_rebuild,
                    schedule_lighting_rebuild,
                )
                    .chain()
                    .after(SimulationSet::VoxelChange)
                    .before(SimulationSet::Survival),
            )
            .add_systems(
                OnExit(crate::shared::states::AppState::InGame),
                (clear_world_lighting, local::clear_local_lighting).chain(),
            );
    }
}
