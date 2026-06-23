use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::dependency::DependencyTracker;
use crate::engine::asset::manager::{AssetManager, asset_manager_bridge_system};
use crate::engine::asset::pipeline::AssetPipeline;
use crate::engine::asset::pipeline::stages::{CacheStage, ResolverStage, SourceStage};
use crate::engine::asset::registry::AssetRegistry;
use crate::engine::asset::resolver::{DefaultResolver, ResolverChain};
use crate::engine::asset::runtime::AssetRuntimePlugin;
use crate::engine::asset::source::{ResourcePackManager, SourceManager, SourceRegistry};
use bevy::prelude::*;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        let resolvers = ResolverChain::new(vec![Box::new(DefaultResolver::new("assets"))]);
        let sources = SourceManager::default();

        // 构建 Pipeline: Resolver → Source → Cache
        let pipeline = AssetPipeline::new()
            .add_stage(ResolverStage::new("assets"))
            .add_stage(SourceStage)
            .add_stage(CacheStage);

        app.init_resource::<AssetManager>()
            .init_resource::<AssetCache>()
            .init_resource::<AssetRegistry>()
            .insert_resource(pipeline)
            .insert_resource(resolvers)
            .insert_resource(sources)
            .init_resource::<SourceRegistry>()
            .init_resource::<ResourcePackManager>()
            .init_resource::<DependencyTracker>()
            .add_plugins(AssetRuntimePlugin)
            .add_systems(PreUpdate, asset_manager_bridge_system);
    }
}
