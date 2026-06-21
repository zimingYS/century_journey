use std::collections::HashMap;
use bevy::prelude::*;

/// 资源缓存 — 存储已加载资源的 Handle。
/// 同一 AssetId 只加载一次，后续请求复用缓存的 Handle。
#[derive(Resource, Debug, Default)]
pub struct AssetCache {
    /// 纹理句柄缓存
    textures: HashMap<String, Handle<Image>>,
    /// JSON 资源句柄缓存
    json_assets: HashMap<String, Handle<crate::engine::asset::loader::json::JsonAsset>>,
}

impl AssetCache {
    /// 获取缓存的纹理 Handle。
    pub fn get_texture(&self, id: &str) -> Option<&Handle<Image>> {
        self.textures.get(id)
    }

    /// 存入纹理 Handle。
    pub fn set_texture(&mut self, id: &str, handle: Handle<Image>) {
        self.textures.insert(id.to_string(), handle);
    }

    /// 获取缓存的 JSON Handle。
    pub fn get_json(&self, id: &str) -> Option<&Handle<crate::engine::asset::loader::json::JsonAsset>> {
        self.json_assets.get(id)
    }

    /// 存入 JSON Handle。
    pub fn set_json(&mut self, id: &str, handle: Handle<crate::engine::asset::loader::json::JsonAsset>) {
        self.json_assets.insert(id.to_string(), handle);
    }

    /// 清除指定资源的缓存（用于热重载）。
    pub fn invalidate(&mut self, id: &str) {
        self.textures.remove(id);
        self.json_assets.remove(id);
    }
}
