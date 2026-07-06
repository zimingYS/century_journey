use crate::engine::asset::cache::AssetCache;
use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::pipeline::AssetPipeline;
use crate::engine::asset::resolver::AssetResolver;
use crate::engine::asset::texture::{TextureAsset, TextureMetadata};
use bevy::asset::LoadState;
use bevy::prelude::*;
use std::collections::HashMap;

/// 资源管理器 —— Engine Asset 面向 `Handle<T>` 资源的唯一入口（Facade）。
///
/// 只负责：解析路径（`AssetResolver`）→ 按规则加载（`AssetPipeline` + Bevy `AssetServer`）
/// → 缓存 Handle（`AssetCache`）。状态查询直接问 Bevy，不自建状态机。
///
/// 同步读配置/JSON 用 [`AssetFiles`](super::files::AssetFiles)，不属于这里。
#[derive(Resource)]
pub struct AssetManager {
    resolver: AssetResolver,
    cache: AssetCache,
    textures: HashMap<String, TextureAsset>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            resolver: AssetResolver::default(),
            cache: AssetCache::default(),
            textures: HashMap::new(),
        }
    }
}

impl AssetManager {
    /// 加载纹理（像素风格）。重复调用同一个 `id` 直接返回缓存结果。
    /// 返回的 `TextureAsset.metadata` 在 Bevy 解码完成前是占位默认值，
    /// 真实宽高由 [`pipeline::sync_texture_metadata_system`](super::pipeline::sync_texture_metadata_system) 每帧回填。
    pub fn texture(&mut self, id: &AssetId, asset_server: &AssetServer) -> TextureAsset {
        let key = id.to_string();
        if let Some(existing) = self.textures.get(&key) {
            return existing.clone();
        }

        let location = self.resolver.resolve(id, "png");
        let handle = AssetPipeline::load_pixel_texture(asset_server, &location);
        self.cache.insert(&key, handle.clone());

        let asset = TextureAsset::new(handle, TextureMetadata::default(), id.clone());
        self.textures.insert(key, asset.clone());
        asset
    }

    /// 加载字体
    pub fn font(&mut self, id: &AssetId, asset_server: &AssetServer) -> Handle<Font> {
        let key = id.to_string();
        if let Some(handle) = self.cache.get::<Font>(&key) {
            return handle;
        }
        let location = self.resolver.resolve(id, "ttf");
        let handle = AssetPipeline::load::<Font>(asset_server, &location);
        self.cache.insert(&key, handle.clone());
        handle
    }

    /// 是否加载完成——直接委托给 Bevy 的 `LoadState`。
    pub fn is_loaded<T: Asset>(&self, handle: &Handle<T>, asset_server: &AssetServer) -> bool {
        matches!(asset_server.get_load_state(handle), Some(LoadState::Loaded))
    }

    pub fn resolver(&self) -> &AssetResolver {
        &self.resolver
    }

    /// 供 `AssetPipeline` 的同步 system 调用，业务代码不要直接用。
    pub(crate) fn sync_pending_texture_metadata(&mut self, images: &Assets<Image>) {
        for asset in self.textures.values_mut() {
            let is_placeholder = asset.metadata.width <= 1 && asset.metadata.height <= 1;
            if is_placeholder {
                if let Some(image) = images.get(&asset.handle) {
                    let size = image.size();
                    asset.metadata = TextureMetadata::from_size(size.x, size.y);
                }
            }
        }
    }
}
