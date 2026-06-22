use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::dependency::DependencyTracker;
use crate::engine::asset::manager::AssetManager;
use crate::engine::asset::pipeline::AssetPipeline;
use crate::engine::asset::registry::AssetRegistry;
use crate::engine::asset::resolver::{DefaultResolver, ResolverChain};
use crate::engine::asset::runtime::AssetRuntimePlugin;
use crate::engine::asset::source::{ResourcePackManager, SourceManager, SourceRegistry};
use bevy::prelude::*;

/// Engine Asset Plugin V4
///
/// 初始化完整 Asset Pipeline + Runtime + 多来源系统。
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        let resolvers = ResolverChain::new(vec![Box::new(DefaultResolver::new("assets"))]);
        let sources = SourceManager::default();
        let pipeline = AssetPipeline::new();

        app
            // Pipeline + Core
            .init_resource::<AssetManager>()
            .init_resource::<AssetCache>()
            .init_resource::<AssetRegistry>()
            .insert_resource(pipeline)
            .insert_resource(resolvers)
            .insert_resource(sources)
            // Source extras
            .init_resource::<SourceRegistry>()
            .init_resource::<ResourcePackManager>()
            .init_resource::<DependencyTracker>()
            // Runtime (Reference, Reload, Streaming, Memory, Diagnostics)
            .add_plugins(AssetRuntimePlugin);
    }
}
