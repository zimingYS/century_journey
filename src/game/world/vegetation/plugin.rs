//! 组装植被候选索引和固定步生长规则。

use super::growth::grow_saplings_system;
use super::instance::track_tree_root_changes_system;
use super::runtime::{
    VegetationRuntime, index_loaded_growth_blocks_system, reset_vegetation_runtime_system,
    track_growth_block_changes_system,
};
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 组装运行时植被索引与权威生长系统。
pub(in crate::game::world) struct VegetationPlugin;

impl Plugin for VegetationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VegetationRuntime>()
            .add_systems(OnEnter(AppState::InGame), reset_vegetation_runtime_system)
            .add_systems(
                FixedUpdate,
                (
                    track_tree_root_changes_system,
                    track_growth_block_changes_system,
                    index_loaded_growth_blocks_system,
                    grow_saplings_system,
                )
                    .chain()
                    .in_set(SimulationSet::Environment)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
