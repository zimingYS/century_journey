use bevy::asset::AssetServer;
use bevy::prelude::*;

use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::loader::texture::TextureLoader;
use crate::engine::asset::loader::json::JsonLoader;
use crate::engine::asset::loader::AssetLoader as AssetLoaderTrait;
use crate::engine::asset::path::AssetPathResolver;
use crate::engine::asset::registry::AssetRegistry;
use crate::engine::asset::registry::AssetState;

/// 资源管理器 — Engine Asset 的唯一公开入口。
/// 业务代码只能通过 `AssetManager` 加载资源，
/// 禁止直接调用 `AssetServer::load()`。
/// 内部协调 PathResolver → Loader → Cache → Registry 全流程。

#[derive(Resource)]
pub struct AssetManager {
    /// 路径解析器
    resolver: AssetPathResolver,
    /// 纹理加载器
    texture_loader: TextureLoader,
    /// JSON 加载器
    json_loader: JsonLoader,
}

impl AssetManager {
    /// 使用默认解析器创建管理器。
    pub fn new() -> Self {
        Self {
            resolver: AssetPathResolver::default(),
            texture_loader: TextureLoader,
            json_loader: JsonLoader,
        }
    }

    /// 加载纹理资源。
    pub fn texture(
        &self, id: &AssetId,
        asset_server: &AssetServer,
        cache: &mut AssetCache,
        registry: &mut AssetRegistry,
    ) -> Handle<Image> {
        let key = id.to_string();

        if let Some(handle) = cache.get_texture(&key) {
            return handle.clone();
        }

        registry.set_state(&key, AssetState::Loading);
        let path = self.resolver.resolve(id);
        let handle = self.texture_loader.load(&path, asset_server);

        cache.set_texture(&key, handle.clone());
        registry.set_state(&key, AssetState::Loaded);
        handle
    }

    /// 加载 JSON 资源。
    pub fn json(
        &self, id: &AssetId,
        asset_server: &AssetServer,
        cache: &mut AssetCache,
        registry: &mut AssetRegistry,
    ) -> Handle<crate::engine::asset::loader::json::JsonAsset> {
        let key = id.to_string();

        if let Some(handle) = cache.get_json(&key) {
            return handle.clone();
        }

        registry.set_state(&key, AssetState::Loading);
        let path = self.resolver.resolve(id);
        let handle = self.json_loader.load(&path, asset_server);

        cache.set_json(&key, handle.clone());
        registry.set_state(&key, AssetState::Loaded);
        handle
    }

    /// 清除指定资源的缓存
    pub fn invalidate(&self, id: &AssetId, cache: &mut AssetCache, registry: &mut AssetRegistry) {
        let key = id.to_string();
        cache.invalidate(&key);
        registry.set_state(&key, AssetState::Unloaded);
    }

    /// 暴露路径解析器。
    pub fn resolver(&self) -> &AssetPathResolver {
        &self.resolver
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
