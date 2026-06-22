use bevy::prelude::*;
use std::collections::HashMap;

/// 泛型资源缓存
///
/// 使用 Bevy `UntypedHandle` 统一存储不同类型资源的句柄。
/// 同一 `AssetId`（作为 String key）只加载一次，后续直接复用缓存的 Handle。
///
/// # 使用示例
///
/// ```ignore
/// let mut cache: AssetCache = AssetCache::default();
/// cache.insert::<Image>("grass_top", image_handle);
/// let handle: Handle<Image> = cache.get::<Image>("grass_top").unwrap();
/// ```
#[derive(Resource, Debug, Default)]
pub struct AssetCache {
    handles: HashMap<String, UntypedHandle>,
}

impl AssetCache {
    /// 查找缓存的 Handle 并转为指定类型。
    ///
    /// 如果 key 不存在，返回 `None`。
    /// 如果 key 存在但类型不匹配，静默返回 `None`。
    pub fn get<T: Asset>(&self, key: &str) -> Option<Handle<T>> {
        self.handles
            .get(key)
            .and_then(|u| Some(u.clone().typed::<T>()))
    }

    /// 存入指定类型的 Handle。
    pub fn insert<T: Asset>(&mut self, key: &str, handle: Handle<T>) {
        self.handles.insert(key.to_string(), handle.untyped());
    }

    /// 检查 key 是否存在。
    pub fn contains(&self, key: &str) -> bool {
        self.handles.contains_key(key)
    }

    /// 移除指定 key 的缓存条目。
    pub fn remove(&mut self, key: &str) {
        self.handles.remove(key);
    }

    /// 清空所有缓存。
    pub fn clear(&mut self) {
        self.handles.clear();
    }

    /// 返回缓存条目数量。
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// 缓存是否为空。
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// 获取所有缓存的 key。
    pub fn keys(&self) -> Vec<&String> {
        self.handles.keys().collect()
    }
}
