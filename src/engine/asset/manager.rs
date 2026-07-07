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

    /// 加载 glTF/GLB 模型。
    /// 返回 `Handle<Gltf>`——场景、网格、材质可通过 `Assets<Gltf>` 查询。
    /// Bevy 自动识别 `.glb`/`.gltf` 文件并解码。
    pub fn model(&mut self, id: &AssetId, asset_server: &AssetServer) -> Handle<Gltf> {
        use bevy::gltf::Gltf;
        let key = id.to_string();
        if let Some(handle) = self.cache.get::<Gltf>(&key) {
            return handle;
        }
        let location = self.resolver.resolve(id, "glb");
        let handle = AssetPipeline::load::<Gltf>(asset_server, &location);
        self.cache.insert(&key, handle.clone());
        handle
    }

    /// 聚合查询：所有已发出的 `texture()` / `font()` / `model()` 请求是否都已加载完成。
    ///
    /// 用于 Loading → InGame 的状态切换条件，避免进入 InGame 时 UI 上出现
    /// 空白占位纹理（物品图标的 Handle 可能还在后台解码）。
    pub fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        for (_, handle) in self.cache.iter() {
            if !matches!(
                asset_server.get_load_state(handle.id()),
                Some(LoadState::Loaded)
            ) {
                return false;
            }
        }
        true
    }

    /// 清空所有资源缓存（场景切换时调用）。
    pub fn clear(&mut self) {
        self.cache.clear();
        self.textures.clear();
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
