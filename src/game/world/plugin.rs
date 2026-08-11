//! 组装世界状态、生成、流送、植被、交互、时间与实体等子插件。

use crate::game::simulation::SimulationSet;
use crate::game::world::state;
use crate::game::world::voxel_change::apply::apply_voxel_changes;
use crate::game::world::{entity, generation, interaction, lighting, streaming, time, vegetation};
use crate::shared::voxel_change::VoxelChangeBuffer;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{FixedUpdate, IntoScheduleConfigs};

/// 组装世界基础资源、时间、生成、流送、植被、交互和实体子领域插件。
///
/// 本插件只负责世界领域的顶层装配；具体运行逻辑将逐步下沉到各子领域插件。
pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelChangeBuffer>()
            .init_resource::<state::WorldState>()
            .init_resource::<state::ChunkRuntime>()
            .init_resource::<crate::game::block::BlockBehaviorRegistry>()
            .add_systems(
                FixedUpdate,
                apply_voxel_changes.in_set(SimulationSet::VoxelChange),
            )
            .add_systems(Startup, crate::game::block::init_behavior_registry_system)
            .add_plugins((
                time::WorldTimePlugin,
                streaming::WorldStreamingPlugin,
                generation::WorldGenerationPlugin,
                vegetation::VegetationPlugin,
                entity::EntityPlugin,
                interaction::WorldInteractionPlugin,
                lighting::LightingPlugin,
            ));
    }
}
