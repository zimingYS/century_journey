use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::manager::AssetManager;
use crate::engine::asset::registry::AssetRegistry;
use crate::engine::asset::runtime::context::RuntimeContext;
use crate::engine::asset::runtime::diagnostics::DiagnosticsService;
use crate::engine::asset::runtime::memory::{EvictionPolicy, MemoryService};
use crate::engine::asset::runtime::reload::ReloadService;
use crate::engine::asset::runtime::scheduler::RuntimeScheduler;
use crate::engine::asset::runtime::service::RuntimeService;
use crate::engine::asset::runtime::streaming::StreamingService;
use bevy::prelude::*;

/// Asset Runtime — 资源运行时核心
///
/// Runtime 自己不包含业务逻辑。
/// 只负责初始化 Scheduler、启动所有 Service、每帧调度。
#[derive(Resource, Default)]
pub struct AssetRuntime;

/// 每帧 Runtime 更新
pub fn asset_runtime_update_system(
    _runtime: Res<AssetRuntime>,
    mut manager: ResMut<AssetManager>,
    mut scheduler: ResMut<RuntimeScheduler>,
    mut ctx: ResMut<RuntimeContext>,
    mut cache: ResMut<AssetCache>,
    mut registry: ResMut<AssetRegistry>,
) {
    // 0. 处理业务层提交的待加载请求
    let pending = manager.drain_pending();
    for id in &pending {
        let key = id.to_string();
        ctx.retain(&key);
        scheduler.submit(crate::engine::asset::runtime::job::AssetJob::Streaming {
            key,
            priority: crate::engine::asset::runtime::streaming::StreamPriority::Normal,
            timestamp: now_secs(),
        });
    }

    // 1. Scheduler 处理 Job + 更新 Service
    scheduler.update(&mut ctx, &mut cache, &mut registry);
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// AssetRuntime Plugin
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
