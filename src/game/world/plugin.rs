use crate::game::world::{entity, generation, interaction, state, streaming, time};
use bevy::app::{App, Plugin, Startup};

/// 组装世界基础资源、时间、生成、流送和实体子领域插件。
///
/// 本插件只负责世界领域的顶层装配；具体运行逻辑将逐步下沉到各子领域插件。
pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<state::WorldState>()
            .init_resource::<state::ChunkRuntime>()
            .init_resource::<crate::game::block::BlockBehaviorRegistry>()
            .add_systems(Startup, crate::game::block::init_behavior_registry_system)
            .add_plugins((
                time::WorldTimePlugin,
                streaming::WorldStreamingPlugin,
                generation::WorldGenerationPlugin,
                entity::EntityPlugin,
                interaction::WorldInteractionPlugin,
            ));
    }
}
