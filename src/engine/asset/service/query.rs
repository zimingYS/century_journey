use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::registry::AssetRegistry;
use crate::engine::asset::state::AssetState;
use bevy::prelude::*;

/// QueryService — 资源查询服务
pub struct QueryService;

impl QueryService {
    pub fn is_ready(registry: &AssetRegistry, key: &str) -> bool {
        matches!(registry.state(key), AssetState::Ready)
    }

    pub fn is_cached(cache: &AssetCache, key: &str) -> bool {
        cache.contains(key)
    }

    pub fn state(registry: &AssetRegistry, key: &str) -> AssetState {
        registry.state(key)
    }
}

/// RegisterService — 资源注册服务
pub struct RegisterService;

impl RegisterService {
    pub fn register(registry: &mut AssetRegistry, id: AssetId, source: impl Into<String>) {
        registry.register(id, source);
    }
}

/// ReloadService — 资源重载服务
pub struct ReloadService;

impl ReloadService {
    pub fn reload(cache: &mut AssetCache, registry: &mut AssetRegistry, key: &str) {
        cache.remove(key);
        registry.set_state(key, AssetState::Reloading);
    }
}

/// 统一服务入口
pub struct AssetService;

impl AssetService {
    pub fn query() -> QueryService {
        QueryService
    }
    pub fn register() -> RegisterService {
        RegisterService
    }
    pub fn reload() -> ReloadService {
        ReloadService
    }
}
