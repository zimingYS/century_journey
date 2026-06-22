use bevy::asset::AssetServer;
use bevy::prelude::*;

/// 资源加载器 trait。
///
/// 每种资源类型对应一个Loader
/// 不负责缓存——缓存由 AssetCache管理。
pub trait AssetLoader: Send + Sync + 'static {
    type Output: Asset;

    /// 提交加载请求，立即返回 Handle。
    fn load(&self, path: &str, asset_server: &AssetServer) -> Handle<Self::Output>;
}

pub mod json;
pub mod texture;
