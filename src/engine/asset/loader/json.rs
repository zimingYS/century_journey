use crate::engine::asset::loader::AssetLoader;
use bevy::asset::AssetServer;
use bevy::prelude::*;

/// JSON 资源类型
#[derive(Asset, TypePath, Debug, Clone)]
pub struct JsonAsset {
    /// 原始 JSON 字符串
    pub content: String,
}

/// JSON 加载器
pub struct JsonLoader;

impl AssetLoader for JsonLoader {
    type Output = JsonAsset;

    fn load(&self, path: &str, _asset_server: &AssetServer) -> Handle<JsonAsset> {
        Handle::default()
    }
}
