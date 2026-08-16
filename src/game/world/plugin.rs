//! 组装世界状态、生成、流送、植被、交互、时间与实体等子插件。

use crate::game::block::BlockBehaviorPlugin;
use crate::game::world::state::WorldStatePlugin;
use crate::game::world::voxel_change::VoxelChangePlugin;
use crate::game::world::weather::WeatherPlugin;
use crate::game::world::{entity, generation, interaction, lighting, streaming, time, vegetation};
use bevy::prelude::*;

/// 组装世界领域的顶层装配插件。
///
/// 本插件只负责把世界各子领域插件组合起来；具体资源与系统由各子插件
/// 自行注册，保持单一职责与层次化装配。
pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WorldStatePlugin,
            VoxelChangePlugin,
            WeatherPlugin,
            BlockBehaviorPlugin,
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
