//! 为测试和服务端式运行组装不依赖客户端渲染的世界状态。

use crate::game::world::state::authoritative::WorldState;
use crate::game::world::state::chunk_runtime::ChunkRuntime;
use bevy::app::{App, Plugin};
use bevy::prelude::{Fixed, Time};

/// 组装不依赖窗口和渲染器的最小权威世界运行时。
pub struct HeadlessWorldPlugin;

impl Plugin for HeadlessWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldState>()
            .init_resource::<ChunkRuntime>()
            .init_resource::<crate::game::block::BlockBehaviorRegistry>()
            .insert_resource(Time::<Fixed>::from_hz(
                crate::game::world::time::SIMULATION_TICKS_PER_SECOND as f64,
            ))
            .add_plugins(crate::game::simulation::SimulationPlugin)
            .add_plugins(crate::game::gameplay::GameplayPlugin)
            .add_plugins(crate::game::inventory::InventoryPlugin)
            .add_plugins(crate::game::player::plugin::GamePlayerPlugin);
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/state/headless.rs"]
mod tests;
