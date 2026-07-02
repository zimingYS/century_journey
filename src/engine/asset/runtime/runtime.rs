use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::manager::AssetManager;
use crate::engine::asset::registry::AssetRegistry;
use crate::engine::asset::runtime::context::RuntimeContext;
use crate::engine::asset::runtime::diagnostics::DiagnosticsService;
use crate::engine::asset::runtime::memory::{EvictionPolicy, MemoryService};
use crate::engine::asset::runtime::reload::ReloadService;
use crate::engine::asset::runtime::scheduler::RuntimeScheduler;
use crate::engine::asset::runtime::streaming::StreamingService;
use bevy::prelude::*;

/// Asset Runtime
#[derive(Resource, Default)]
pub struct AssetRuntime;

/// 每帧 Runtime 更新
pub fn asset_runtime_update_system(
    _runtime: Res<AssetRuntime>,
    _manager: ResMut<AssetManager>,
    mut scheduler: ResMut<RuntimeScheduler>,
    mut ctx: ResMut<RuntimeContext>,
    mut cache: ResMut<AssetCache>,
    mut registry: ResMut<AssetRegistry>,
) {
    scheduler.update(&mut ctx, &mut cache, &mut registry);
}

pub struct AssetRuntimePlugin;

impl Plugin for AssetRuntimePlugin {
    fn build(&self, app: &mut App) {
        let mut scheduler = RuntimeScheduler::new(50);
        scheduler.register_service(StreamingService::new(20));
        scheduler.register_service(ReloadService::new(2.0));
        scheduler.register_service(MemoryService::new(1024 * 1024 * 1024, EvictionPolicy::LRU));
        scheduler.register_service(DiagnosticsService::new());

        let mut ctx = RuntimeContext::new();
        scheduler.startup(&mut ctx);

        app.init_resource::<AssetRuntime>()
            .insert_resource(scheduler)
            .insert_resource(ctx)
            .add_systems(PostUpdate, asset_runtime_update_system);
    }
}
