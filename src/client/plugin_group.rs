use bevy::prelude::*;

use crate::client::effect::ClientEffectPlugin;
use crate::client::input::ClientInputPlugin;
use crate::client::interpolation::ClientInterpolationPlugin;
use crate::client::particle::ClientParticlePlugin;
use crate::client::player::ClientPlayerPlugin;
use crate::client::renderer::ClientRenderingPlugin;
use crate::client::sky::SkyPlugin;
use crate::client::sound::ClientSoundPlugin;
use crate::client::startup::plugin::ClientStartupPlugin;
use crate::client::ui::UIPlugin;

/// 客户端 Plugin 集合。
pub struct ClientPluginGroup;

impl Plugin for ClientPluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientInputPlugin)
            .add_plugins(ClientRenderingPlugin)
            .add_plugins(ClientPlayerPlugin)
            .add_plugins(ClientInterpolationPlugin)
            .add_plugins(SkyPlugin)
            .add_plugins(UIPlugin)
            .add_plugins(ClientSoundPlugin)
            .add_plugins(ClientParticlePlugin)
            .add_plugins(ClientEffectPlugin)
            .add_plugins(ClientStartupPlugin);
    }
    // fn build(self) -> PluginGroupBuilder {
    //     PluginGroupBuilder::start::<Self>()
    //         // Engine 层（最先注册，其他插件依赖）
    //         .add(AssetPlugin)
    //         .add(TaskPlugin)
    //         // Content 层（数据驱动，不依赖 Game/Client）
    //         .add(ContentLifecyclePlugin)
    //         .add(BiomeContentPlugin)
    //         .add(ItemContentPlugin)
    //         .add(VoxelPlugin)
    //         .add(LootPlugin)
    //         .add(TagContentPlugin)
    //         .add(RecipeContentPlugin)
    //         // Game 层（运行时逻辑，依赖 Content）
    //         .add(GameplayPlugin)
    //         .add(SimulationPlugin)
    //         .add(CraftingPlugin)
    //         .add(WorldPlugin)
    //         .add(InventoryPlugin)
    //         // App 层
    //         .add(CorePlugin)
    //         // Client 层（渲染/UI，依赖 Content+Game）
    //         .add(ClientInputPlugin)
    //         .add(ClientRenderingPlugin)
    //         .add(ClientPlayerPlugin)
    //         .add(ClientInterpolationPlugin)
    //         .add(SkyPlugin)
    //         .add(UIPlugin)
    //         .add(ClientSoundPlugin)
    //         .add(ClientParticlePlugin)
    //         .add(ClientEffectPlugin)
    //         .add(ClientStartupPlugin)
    // }
}
